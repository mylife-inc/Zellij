import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, rmSync, copyFileSync, cpSync, mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { gzipSync } from 'node:zlib';
import path from 'node:path';
import { loadEngine, loadBundle, lock, sha256 } from './bundle.mjs';
import { runFontMetadataPass } from './probe.mjs';
import { generateFontMetadata, loadFontMetadata, loadFontInputs, normalizeMetadata, fontInventory, fontRecipe } from './fonts.mjs';

const original = process.env.ZEL_PDF_PROBE_BUNDLE;
const inputs = process.env.ZEL_PDF_FONT_INPUTS;
assert(original && inputs, 'Set ZEL_PDF_PROBE_BUNDLE and ZEL_PDF_FONT_INPUTS');

test('metadata builds without an old format/database and the compact loader never reads their files', { timeout: 120_000 }, async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-fonts-'));
  try {
    const engine = path.join(root, 'engine'); mkdirSync(engine);
    for (const entry of lock.engine.filter(e => /\.(js|wasm)$/.test(e.name))) {
      copyFileSync(path.join(original, entry.sha256), path.join(engine, entry.sha256));
    }
    const output = path.join(root, 'metadata');
    const report = await generateFontMetadata(engine, inputs, output);
    assert.equal(report.repeatedBuilds, 2);
    assert.equal(report.prebuiltFormatRead, false);
    assert.equal(report.generatedFontMetadataRead, false);
    const receipt = JSON.parse(readFileSync(path.join(inputs, 'verification.json'), 'utf8'));
    assert.equal(report.fontSourceLockSha256, receipt.fontSourceLockSha256);
    const metadata = loadFontMetadata(output);
    for (const entry of lock.resources.filter(e => !metadata.has(e.name))) {
      copyFileSync(path.join(original, entry.sha256), path.join(engine, entry.sha256));
    }
    const format = new Uint8Array([1, 2, 3]);
    const bundle = loadBundle(engine, { format, metadata });
    assert.equal(bundle.format, format);
    assert.equal(sha256(bundle.resources.get('51/luaotfload-names.lua')), fontRecipe.output.sha256);
    assert.equal(new TextDecoder().decode(bundle.resources.get('11/pdftex.map')), '% Zellij uses OpenType fonts.\n');
    assert.equal(report.used.length, fontRecipe.resources.length);
    writeFileSync(path.join(output, 'pdftex.map'), Buffer.alloc(report.map.bytes));
    assert.throws(() => loadFontMetadata(output), /checksum differs/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('corrupt inputs, metadata and existing outputs fail without overwriting data', async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-fonts-reject-'));
  try {
    const bad = path.join(root, 'inputs'); cpSync(inputs, bad, { recursive: true });
    const first = fontInventory.files[0];
    const bytes = readFileSync(path.join(bad, first.sha256)); bytes[0] ^= 1;
    writeFileSync(path.join(bad, first.sha256), bytes);
    assert.throws(() => loadFontInputs(bad), /integrity failed/);
    const output = path.join(root, 'output');
    await assert.rejects(generateFontMetadata(original, bad, output), /integrity failed/);
    assert(!existsSync(output));
    mkdirSync(output); writeFileSync(path.join(output, 'keep'), 'authored');
    await assert.rejects(generateFontMetadata(original, inputs, output), /output already exists/);
    assert.equal(readFileSync(path.join(output, 'keep'), 'utf8'), 'authored');
    writeFileSync(path.join(output, 'luaotfload-names.lua'), Buffer.alloc(fontRecipe.output.bytes));
    assert.throws(() => loadFontMetadata(output), /checksum differs/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('normalization rejects missing/extra fonts, unexpected timestamps and oversized output', () => {
  const bundle = { ...loadEngine(original), resources: loadFontInputs(inputs) };
  return runFontMetadataPass(bundle, fontRecipe.sourceDateEpoch).then(({ database }) => {
    const text = new TextDecoder().decode(normalizeMetadata(database));
    assert.equal(sha256(normalizeMetadata(gzipSync(text))), fontRecipe.output.sha256);
    assert.throws(() => normalizeMetadata(gzipSync(text.replace('["basename"]=', '["other"]='))), /font coverage/);
    assert.throws(() => normalizeMetadata(gzipSync(text + '\n["basename"]="extra.otf"')), /font coverage/);
    assert.throws(() => normalizeMetadata(gzipSync(text.replace('["timestamp"]=', '["other-time"]='))), /timestamp coverage/);
    assert.throws(() => normalizeMetadata(gzipSync(text.replace('["created"]=', '["other-date"]='))));
    assert.throws(() => normalizeMetadata(gzipSync(Buffer.alloc(2 * 1024 * 1024 + 1))));
  });
});

test('a looping Lua generator obeys the real deadline and an incomplete database is rejected', { timeout: 15_000 }, async () => {
  const engine = loadEngine(original);
  const bundle = code => ({ ...engine, resources: new Map([['51/luaotfload-tool.lua', new TextEncoder().encode(code)]]) });
  await assert.rejects(runFontMetadataPass(bundle('while true do end'), fontRecipe.sourceDateEpoch, 1000), /timed out/);
  await assert.rejects(runFontMetadataPass(bundle('return'), fontRecipe.sourceDateEpoch), /No such file|FS error/);
});
