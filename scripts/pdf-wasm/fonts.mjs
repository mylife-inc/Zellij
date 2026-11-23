// Maintainer font database generation from original Lua sources and OTF bytes.
import assert from 'node:assert/strict';
import { readFileSync, lstatSync } from 'node:fs';
import { mkdir, mkdtemp, lstat, writeFile, rename, rm } from 'node:fs/promises';
import { gunzipSync } from 'node:zlib';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { loadEngine, verifiedBytes, sha256 } from './bundle.mjs';
import { runFontMetadataPass } from './probe.mjs';

const inventoryBytes = readFileSync(new URL('./font-sources.lock.json', import.meta.url));
export const fontInventory = JSON.parse(inventoryBytes);
export const fontRecipe = JSON.parse(readFileSync(new URL('./font-inputs.json', import.meta.url)));
assert.deepEqual(fontInventory.recipe, fontRecipe, 'Font source recipe is stale');
const fontNames = fontRecipe.resources.filter(n => n.endsWith('.otf')).map(n => n.slice(3)).sort();
export const fontMap = new TextEncoder().encode('% Zellij uses OpenType fonts.\n');

export function loadFontInputs(root) {
  const stat = lstatSync(root);
  if (!stat.isDirectory() || stat.isSymbolicLink()) throw new Error('Font inputs must be a real directory');
  return new Map(fontInventory.files.map(f => [f.resource, verifiedBytes(root, { ...f, name: f.resource })]));
}

export function normalizeMetadata(compressed) {
  let text = new TextDecoder('utf-8', { fatal: true }).decode(gunzipSync(compressed, { maxOutputLength: 2 * 1024 * 1024 }));
  const names = [...text.matchAll(/\["basename"\]="([^"]+)"/g)].map(m => m[1]).sort();
  assert.deepEqual(names, fontNames, 'Generated database font coverage differs');
  // Date labels use the guest timezone, and timestamps are cache freshness hints.
  // Normalize only these known fields; require the expected count before editing.
  const dates = /\["(created|modified)"\]="[^"]*"/g;
  assert.deepEqual([...text.matchAll(dates)].map(m => m[1]).sort(), ['created', 'modified']);
  text = text.replace(dates, '["$1"]="1970-01-01 00:00:00"');
  const times = /(\["timestamp"\]=)[0-9.]+/g;
  assert.equal([...text.matchAll(times)].length, fontNames.length, 'Font timestamp coverage differs');
  text = text.replace(times, (_, prefix) => prefix + '0');
  return new TextEncoder().encode(text);
}

export function loadFontMetadata(root) {
  const stat = lstatSync(root);
  assert(stat.isDirectory() && !stat.isSymbolicLink(), 'Font metadata must be a real directory');
  function read(name, expected) {
    const file = path.join(root, name);
    const info = lstatSync(file);
    assert(info.isFile() && !info.isSymbolicLink() && info.size === expected.bytes, 'Invalid font metadata file');
    const bytes = new Uint8Array(readFileSync(file));
    assert.equal(sha256(bytes), expected.sha256, 'Font metadata checksum differs');
    return bytes;
  }
  return new Map([
    ['51/luaotfload-names.lua', read('luaotfload-names.lua', fontRecipe.output)],
    ['11/pdftex.map', read('pdftex.map', { bytes: fontMap.length, sha256: sha256(fontMap) })],
  ]);
}

export async function generateFontMetadata(engineRoot, inputRoot, output) {
  const destination = path.resolve(output);
  try { await lstat(destination); throw new Error('Font metadata output already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  const bundle = { ...loadEngine(engineRoot), resources: loadFontInputs(inputRoot) };
  const first = await runFontMetadataPass(bundle, fontRecipe.sourceDateEpoch);
  const second = await runFontMetadataPass(bundle, fontRecipe.sourceDateEpoch);
  const bytes = normalizeMetadata(first.database);
  assert.deepEqual(bytes, normalizeMetadata(second.database), 'Repeated font metadata output differs');
  assert.deepEqual(first.used, [...fontRecipe.resources].sort(), 'Font input usage differs from the reviewed inventory');
  assert.deepEqual(second.used, first.used);
  assert.deepEqual(first.misses, [], 'Font generation requested an unprovided resource');
  assert.deepEqual(second.misses, first.misses);
  assert.equal(bytes.length, fontRecipe.output.bytes, 'Font metadata output size differs');
  assert.equal(sha256(bytes), fontRecipe.output.sha256, 'Font metadata output checksum differs');
  const report = { schemaVersion: 1, experimental: true, prebuiltFormatRead: false,
    generatedFontMetadataRead: false, fontSourceLockSha256: sha256(inventoryBytes),
    sourceDateEpoch: fontRecipe.sourceDateEpoch, repeatedBuilds: 2, fonts: fontNames.length,
    database: fontRecipe.output, map: { bytes: fontMap.length, sha256: sha256(fontMap) },
    used: first.used, misses: first.misses, heapBytes: first.heapBytes };
  await mkdir(path.dirname(destination), { recursive: true });
  const stage = await mkdtemp(path.join(path.dirname(destination), '.font-metadata-'));
  try {
    await writeFile(path.join(stage, 'luaotfload-names.lua'), bytes);
    await writeFile(path.join(stage, 'pdftex.map'), fontMap);
    await writeFile(path.join(stage, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
    await rename(stage, destination);
  } finally { await rm(stage, { recursive: true, force: true }); }
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (process.argv.length !== 5) throw new Error('Usage: node scripts/pdf-wasm/fonts.mjs ENGINE INPUTS NEW_OUTPUT');
  const result = await generateFontMetadata(...process.argv.slice(2));
  console.log(`Two identical font database builds: ${result.database.bytes} bytes, ${result.database.sha256}`);
}
