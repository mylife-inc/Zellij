import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, rmSync, copyFileSync, mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { loadEngine, lock, sha256 } from './bundle.mjs';
import { runFormatPass } from './probe.mjs';
import { generateFormat, loadFormatInputs, formatInventory, recipe } from './format.mjs';

const original = process.env.ZEL_PDF_PROBE_BUNDLE;
const inputs = process.env.ZEL_PDF_FORMAT_INPUTS;
assert(original && inputs, 'Set ZEL_PDF_PROBE_BUNDLE and ZEL_PDF_FORMAT_INPUTS');

test('INITEX reproduces the pinned bytes without a prebuilt format or generated font metadata', { timeout: 120_000 }, async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-format-'));
  try {
    const engine = path.join(root, 'engine');
    mkdirSync(engine);
    for (const entry of lock.engine.filter(e => /\.(js|wasm)$/.test(e.name))) {
      copyFileSync(path.join(original, entry.sha256), path.join(engine, entry.sha256));
    }
    const report = await generateFormat(engine, inputs, path.join(root, 'built'));
    assert.equal(report.repeatedBuilds, 2);
    assert.equal(report.prebuiltFormatRead, false);
    assert.equal(report.generatedFontMetadataRead, false);
    const sourceReceipt = JSON.parse(readFileSync(path.join(inputs, 'verification.json'), 'utf8'));
    assert.equal(report.formatSourceLockSha256, sourceReceipt.formatSourceLockSha256,
      'Source preparation and format generation must identify the same manifest bytes');
    assert.equal(sha256(readFileSync(path.join(root, 'built/zellij-lualatex.fmt'))), recipe.output.sha256);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('corrupt source inputs and existing outputs are rejected without publication', async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-format-reject-'));
  try {
    for (const f of formatInventory.files) copyFileSync(path.join(inputs, f.sha256), path.join(root, f.sha256));
    const first = formatInventory.files[0];
    const data = readFileSync(path.join(root, first.sha256)); data[0] ^= 1;
    writeFileSync(path.join(root, first.sha256), data);
    assert.throws(() => loadFormatInputs(root), /integrity failed/);
    const output = path.join(root, 'new');
    await assert.rejects(generateFormat(original, root, output), /integrity failed/);
    assert(!existsSync(output));
    mkdirSync(output); writeFileSync(path.join(output, 'keep'), 'authored');
    await assert.rejects(generateFormat(original, inputs, output), /output already exists/);
    assert.equal(readFileSync(path.join(output, 'keep'), 'utf8'), 'authored');
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('a frozen guest clock never disables the real supervisor deadline', { timeout: 15_000 }, async () => {
  const bundle = { ...loadEngine(original), resources: new Map([
    ['26/lualatex.ini', new TextEncoder().encode('\\catcode123=1 \\catcode125=2 \\directlua0{while true do end}')],
  ]) };
  await assert.rejects(runFormatPass(bundle, recipe.sourceDateEpoch, 5000), /timed out/);
});
