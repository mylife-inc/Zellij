// Shared Node bootstrap for next.config and the TypeScript runtime. Keep this
// outside src/dist so both published module graphs resolve the same file.
const fs = require('node:fs');
const path = require('node:path');
const { parse } = require('yaml');

function resolveContentPaths(options = {}) {
  const appRoot = path.resolve(options.appRoot ?? process.cwd());
  return { appRoot, configFile: path.resolve(appRoot, options.configFile ?? 'zellij.config.yaml') };
}

function readConfig(file) {
  let text;
  try { text = fs.readFileSync(file, 'utf8'); }
  catch (error) {
    if (error.code === 'ENOENT') return undefined;
    throw error;
  }
  const value = parse(text) ?? {};
  if (typeof value !== 'object' || Array.isArray(value)) throw new Error(`${file}: expected a YAML mapping`);
  return value;
}

function normalizeDeploymentBasePath(value) {
  if (value === undefined) return '';
  if (typeof value !== 'string') throw new Error('basePath must be a string');
  const text = value.trim();
  if (text.startsWith('//') || /[\s?#\\%]/u.test(text)) throw new Error(`Invalid basePath: ${JSON.stringify(value)}`);
  const normalized = text.replace(/\/+$/, '');
  if (!normalized) return '';
  const result = normalized.startsWith('/') ? normalized : `/${normalized}`;
  if (result.slice(1).split('/').some((part) => !part || part === '.' || part === '..')) {
    throw new Error(`Invalid basePath: ${JSON.stringify(value)}`);
  }
  return result;
}

function resolveDeploymentBasePath(...values) {
  const explicit = values.filter((value) => value !== undefined).map(normalizeDeploymentBasePath);
  if (new Set(explicit).size > 1) {
    throw new Error(`basePath settings disagree: ${explicit.map((value) => JSON.stringify(value)).join(', ')}. Set it in site.yaml or the wrapper, or use matching values.`);
  }
  return explicit[0] ?? '';
}

// Private CLI-to-engine protocol, also retained in a generated Next image.
// Decoding data here keeps authored strings out of shell/Dockerfile syntax.
function readBuildOverrides() {
  const encoded = process.env.ZELLIJ_BUILD_OVERRIDES;
  if (encoded === undefined) return {};
  if (encoded.length > 32768 || !/^(?:[0-9a-f]{2})+$/i.test(encoded)) {
    throw new Error('Invalid Zellij build overrides encoding');
  }
  let value;
  try { value = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(Buffer.from(encoded, 'hex'))); }
  catch { throw new Error('Invalid Zellij build overrides JSON'); }
  if (!value || typeof value !== 'object' || Array.isArray(value) || value.version !== 1
      || Object.keys(value).some(key => !['version', 'basePath', 'siteUrl', 'noCheck'].includes(key))
      || typeof value.noCheck !== 'boolean') {
    throw new Error('Unsupported Zellij build overrides; use a matching CLI and engine');
  }
  const result = {};
  if (Object.hasOwn(value, 'basePath')) {
    if (typeof value.basePath !== 'string' || value.basePath.length > 2048 || /[\s\u0000-\u001f\u007f]/u.test(value.basePath)) throw new Error('Invalid build basePath');
    result.basePath = normalizeDeploymentBasePath(value.basePath);
  }
  if (Object.hasOwn(value, 'siteUrl')) {
    if (typeof value.siteUrl !== 'string' || value.siteUrl.length > 2048 || /[\s\\@]/u.test(value.siteUrl)
        || !/^https?:\/\/[^/?#]+\/?$/i.test(value.siteUrl)) throw new Error('Invalid build siteUrl');
    let url;
    try { url = new URL(value.siteUrl); } catch { throw new Error('Invalid build siteUrl'); }
    if (!['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password
        || url.pathname !== '/' || url.search || url.hash || value.siteUrl.includes('?') || value.siteUrl.includes('#')) {
      throw new Error('Build siteUrl must be an HTTP(S) origin; use basePath for a deployment prefix');
    }
    result.siteUrl = url.origin;
  }
  return result;
}

function resolveContentLocation(options = {}) {
  const { appRoot, configFile } = resolveContentPaths(options);
  let legacy = options.contentDir ? undefined : readConfig(configFile);
  const wrapperConfig = legacy && Object.hasOwn(legacy, 'version');
  if (wrapperConfig && (legacy.version !== 1 || Object.keys(legacy).some(key => !['version', 'content'].includes(key)) || typeof legacy.content !== 'string' || !legacy.content.trim())) {
    throw new Error(`${configFile}: versioned wrapper config requires version: 1 and a nonempty content path only; put presentation settings in content/site.yaml`);
  }
  const legacyConfigUsed = legacy !== undefined && !wrapperConfig;
  const configured = options.contentDir ?? legacy?.content ?? 'content';
  if (typeof configured !== 'string' || !configured.trim()) throw new Error('content directory must be a nonempty string');
  const contentDir = path.resolve(appRoot, configured);
  const layout = fs.existsSync(path.join(contentDir, 'marketing')) ? 'legacy' : 'folder';
  if (layout === 'legacy' && wrapperConfig) throw new Error('versioned wrapper config requires the current hosted folder layout, not marketing/');
  if (layout === 'legacy' && legacy === undefined) legacy = readConfig(configFile);
  const settings = layout === 'legacy' ? legacy : readConfig(path.join(contentDir, 'site.yaml'));
  const authoredBasePath = settings?.basePath === undefined ? undefined : normalizeDeploymentBasePath(settings.basePath);
  const basePath = readBuildOverrides().basePath ?? authoredBasePath;
  return { appRoot, configFile, contentDir, layout, basePath, legacyConfigUsed };
}

exports.resolveContentPaths = resolveContentPaths;
exports.resolveContentLocation = resolveContentLocation;
exports.normalizeDeploymentBasePath = normalizeDeploymentBasePath;
exports.resolveDeploymentBasePath = resolveDeploymentBasePath;
exports.readBuildOverrides = readBuildOverrides;
