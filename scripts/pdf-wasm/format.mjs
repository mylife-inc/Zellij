// Maintainer INITEX build, independent of the mirror's precompiled format/data.
import assert from 'node:assert/strict';
import { readFileSync, lstatSync } from 'node:fs';
import { mkdir, mkdtemp, lstat, writeFile, rename, rm } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { loadEngine, verifiedBytes, sha256 } from './bundle.mjs';
import { runFormatPass } from './probe.mjs';

const inventoryBytes = readFileSync(new URL('./format-sources.lock.json', import.meta.url));
export const formatInventory = JSON.parse(inventoryBytes);
export const recipe = JSON.parse(readFileSync(new URL('./format-inputs.json', import.meta.url)));
assert.deepEqual(formatInventory.recipe, recipe, 'Format source recipe is stale');

export function loadFormatInputs(root) {
  const stat = lstatSync(root);
  if (!stat.isDirectory() || stat.isSymbolicLink()) throw new Error('Format inputs must be a real directory');
  return new Map(formatInventory.files.map(f => [f.resource, verifiedBytes(root, { ...f, name: f.resource })]));
}

export async function generateFormat(engineRoot, inputRoot, output) {
  const destination = path.resolve(output);
  try { await lstat(destination); throw new Error('Format output already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  // Deliberately load only the verified compiler and original source files.
  // Neither the old format nor the two mirror-generated metadata files is read.
  const bundle = { ...loadEngine(engineRoot), resources: loadFormatInputs(inputRoot) };
  const first = await runFormatPass(bundle, recipe.sourceDateEpoch);
  const second = await runFormatPass(bundle, recipe.sourceDateEpoch);
  assert.deepEqual(second.format, first.format, 'Repeated INITEX output differs');
  assert.deepEqual(first.used, [...recipe.resources].sort(), 'Format input usage differs from the reviewed inventory');
  assert.deepEqual(second.used, first.used);
  assert.deepEqual(second.misses, first.misses);
  assert.equal(first.format.length, recipe.output.bytes, 'Format output size differs');
  assert.equal(sha256(first.format), recipe.output.sha256, 'Format output checksum differs');
  const report = { schemaVersion: 1, experimental: true, prebuiltFormatRead: false,
    generatedFontMetadataRead: false, formatSourceLockSha256: sha256(inventoryBytes),
    sourceDateEpoch: recipe.sourceDateEpoch, repeatedBuilds: 2,
    format: recipe.output, used: first.used, misses: first.misses, heapBytes: first.heapBytes };
  await mkdir(path.dirname(destination), { recursive: true });
  const stage = await mkdtemp(path.join(path.dirname(destination), '.format-build-'));
  try {
    await writeFile(path.join(stage, 'zellij-lualatex.fmt'), first.format);
    await writeFile(path.join(stage, 'format.log'), first.log);
    await writeFile(path.join(stage, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
    await rename(stage, destination);
  } finally { await rm(stage, { recursive: true, force: true }); }
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (process.argv.length !== 5) throw new Error('Usage: node scripts/pdf-wasm/format.mjs ENGINE INPUTS NEW_OUTPUT');
  const result = await generateFormat(...process.argv.slice(2));
  console.log(`Two identical offline format builds: ${result.format.bytes} bytes, ${result.format.sha256}`);
}
