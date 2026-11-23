import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, rmSync, cpSync, writeFileSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { loadBundle, loadCompactBundle } from './bundle.mjs';
import { loadFontMetadata } from './fonts.mjs';

const compact = process.env.ZEL_PDF_COMPACT_PACKAGE;
const original = process.env.ZEL_PDF_PROBE_BUNDLE;
const format = process.env.ZEL_PDF_REBUILT_FORMAT;
const fonts = process.env.ZEL_PDF_FONT_METADATA;
assert(compact && original && format && fonts, 'Set compact, original, rebuilt format and font metadata paths');

test('compact package supplies exactly the reviewed compiler, format and resource bytes', () => {
  const expected = loadBundle(original, { format: new Uint8Array(readFileSync(format)), metadata: loadFontMetadata(fonts) });
  assert.deepEqual(loadBundle(compact), expected);
  assert.throws(() => loadBundle(compact, { format: expected.format }), /outside replacements/);
});

test('manifest changes and resource corruption cannot select different compiler bytes', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-compact-'));
  try {
    cpSync(compact, root, { recursive: true });
    const file = path.join(root, 'manifest.json');
    const originalManifest = readFileSync(file);
    const manifest = JSON.parse(originalManifest);
    manifest.engine[0].sha256 = '../unexpected';
    writeFileSync(file, JSON.stringify(manifest));
    assert.throws(() => loadCompactBundle(root), /reviewed inventory/);
    writeFileSync(file, originalManifest);
    const resource = JSON.parse(originalManifest).resources[0];
    const blob = path.join(root, resource.sha256);
    const bytes = readFileSync(blob); bytes[0] ^= 1; writeFileSync(blob, bytes);
    assert.throws(() => loadCompactBundle(root), /integrity failed/);
    writeFileSync(blob, bytes.subarray(1));
    assert.throws(() => loadCompactBundle(root), /Missing or invalid/);
    writeFileSync(file, Buffer.alloc(256 * 1024 + 1));
    assert.throws(() => loadCompactBundle(root), /Invalid compact manifest/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
