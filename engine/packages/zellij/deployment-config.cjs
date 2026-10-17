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

function resolveContentLocation(options = {}) {
  const { appRoot, configFile } = resolveContentPaths(options);
  let legacy = options.contentDir ? undefined : readConfig(configFile);
  const legacyConfigUsed = legacy !== undefined;
  const configured = options.contentDir ?? legacy?.content ?? 'content';
  if (typeof configured !== 'string' || !configured.trim()) throw new Error('content directory must be a nonempty string');
  const contentDir = path.resolve(appRoot, configured);
  const layout = fs.existsSync(path.join(contentDir, 'marketing')) ? 'legacy' : 'folder';
  if (layout === 'legacy' && legacy === undefined) legacy = readConfig(configFile);
  const settings = layout === 'legacy' ? legacy : readConfig(path.join(contentDir, 'site.yaml'));
  const basePath = settings?.basePath === undefined ? undefined : normalizeDeploymentBasePath(settings.basePath);
  return { appRoot, configFile, contentDir, layout, basePath, legacyConfigUsed };
}

exports.resolveContentPaths = resolveContentPaths;
exports.resolveContentLocation = resolveContentLocation;
exports.normalizeDeploymentBasePath = normalizeDeploymentBasePath;
exports.resolveDeploymentBasePath = resolveDeploymentBasePath;
