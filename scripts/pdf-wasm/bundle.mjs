// Maintainer feasibility package. Not a released CLI toolchain or download path.
import { createHash } from 'node:crypto';
import { readFileSync, lstatSync, readdirSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';

export const lock = JSON.parse(readFileSync(new URL('./lock.json', import.meta.url), 'utf8'));
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export const entries = [...lock.engine, ...lock.resources];

export function verifiedBytes(root, entry) {
  // Paths in the checked-in lock are flat hash names, never guest-selected paths.
  if (!/^[a-f0-9]{64}$/.test(entry.sha256) || entry.bytes > 64 * 1024 * 1024) {
    throw new Error('Invalid PDF probe lock entry');
  }
  const file = path.join(root, entry.sha256);
  const stat = lstatSync(file);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size !== entry.bytes) {
    throw new Error(`Missing or invalid PDF probe resource: ${entry.name}`);
  }
  const bytes = readFileSync(file);
  if (sha256(bytes) !== entry.sha256) throw new Error(`PDF probe integrity failed: ${entry.name}`);
  return new Uint8Array(bytes);
}

export function loadEngine(root) {
  if (!lstatSync(root).isDirectory() || lstatSync(root).isSymbolicLink()) {
    throw new Error('PDF probe bundle must be a real directory');
  }
  const engine = new Map(lock.engine.filter(entry => /\.(js|wasm)$/.test(entry.name))
    .map(entry => [entry.name, verifiedBytes(root, entry)]));
  return { code: new TextDecoder().decode(engine.get('wasmtex-luatex.js')), wasm: engine.get('wasmtex-luatex.wasm') };
}

export function loadBundle(root, { format, metadata } = {}) {
  let compact = false;
  try { lstatSync(path.join(root, 'manifest.json')); compact = true; }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (compact) {
    if (format !== undefined || metadata !== undefined) throw new Error('Compact package cannot use outside replacements');
    return loadCompactBundle(root);
  }
  const core = loadEngine(root);
  const formatEntry = lock.engine.find(entry => entry.name === 'wasmtex-luatex.fmt.gz');
  const resources = new Map(lock.resources.map(entry => [entry.name,
    metadata?.get(entry.name) ?? verifiedBytes(root, entry)]));
  return {
    ...core,
    format: format ?? new Uint8Array(gunzipSync(verifiedBytes(root, formatEntry), { maxOutputLength: 64 * 1024 * 1024 })),
    resources,
  };
}

export function loadCompactBundle(root) {
  const rootInfo = lstatSync(root);
  if (!rootInfo.isDirectory() || rootInfo.isSymbolicLink()) throw new Error('Compact bundle must be a real directory');
  const file = path.join(root, 'manifest.json');
  const info = lstatSync(file);
  if (!info.isFile() || info.isSymbolicLink() || info.size > 256 * 1024) throw new Error('Invalid compact manifest');
  const manifest = JSON.parse(readFileSync(file, 'utf8'));
  const formatRecipe = JSON.parse(readFileSync(new URL('./format-inputs.json', import.meta.url)));
  const fontRecipe = JSON.parse(readFileSync(new URL('./font-inputs.json', import.meta.url)));
  const map = new TextEncoder().encode('% Zellij uses OpenType fonts.\n');
  const replacements = {
    '51/luaotfload-names.lua': fontRecipe.output,
    '11/pdftex.map': { bytes: map.length, sha256: sha256(map) },
  };
  const clean = ({ name, bytes, sha256 }) => ({ name, bytes, sha256 });
  const expected = { schemaVersion: 1, kind: 'experimental-pdf-resources', redistributionReviewComplete: false,
    engine: lock.engine.filter(e => /\.(js|wasm)$/.test(e.name)).map(clean),
    format: { name: 'zellij-lualatex.fmt', ...formatRecipe.output },
    resources: lock.resources.map(e => clean({ ...e, ...replacements[e.name] })) };
  // Match the known inventory before resolving any supplied hash or resource.
  // Manifest key formatting/order is irrelevant; every entry and value matters.
  if (!isDeepStrictEqual(manifest, expected)) throw new Error('Compact manifest differs from reviewed inventory');
  const engine = new Map(expected.engine.map(e => [e.name, verifiedBytes(root, e)]));
  return { code: new TextDecoder().decode(engine.get('wasmtex-luatex.js')), wasm: engine.get('wasmtex-luatex.wasm'),
    format: verifiedBytes(root, expected.format),
    resources: new Map(expected.resources.map(e => [e.name, verifiedBytes(root, e)])) };
}

export function loadInputs(root) {
  if (!lstatSync(root).isDirectory() || lstatSync(root).isSymbolicLink()) {
    throw new Error('PDF probe inputs must be a real directory');
  }
  const result = new Map();
  let total = 0;
  function visit(directory, prefix = '') {
    for (const item of readdirSync(directory, { withFileTypes: true })) {
      if (!/^[A-Za-z0-9_.-]+$/.test(item.name) || item.isSymbolicLink()) {
        throw new Error('Unsafe generated PDF input');
      }
      const name = prefix + item.name;
      const file = path.join(directory, item.name);
      if (item.isDirectory()) visit(file, name + '/');
      else if (item.isFile()) {
        const size = lstatSync(file).size;
        total += size;
        if (size > 32 * 1024 * 1024 || total > 64 * 1024 * 1024 || result.size >= 4096) {
          throw new Error('PDF probe inputs exceed budget');
        }
        result.set(name, new Uint8Array(readFileSync(file)));
      } else throw new Error('Nonregular generated PDF input');
    }
  }
  visit(root);
  if (!result.has('book.tex')) throw new Error('Missing generated book.tex');
  return result;
}
