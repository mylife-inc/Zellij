import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, rmSync, writeFileSync, readFileSync, existsSync, mkdirSync, copyFileSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { loadBundle, loadInputs, lock, verifiedBytes } from './bundle.mjs';
import { runPass, compile } from './probe.mjs';

const bundleRoot = process.env.ZEL_PDF_PROBE_BUNDLE;
assert(bundleRoot, 'Set ZEL_PDF_PROBE_BUNDLE to a downloaded feasibility bundle');
const bundle = loadBundle(bundleRoot);
const tex = body => new Map([['book.tex', new TextEncoder().encode(`\\documentclass{article}\\begin{document}${body}\\end{document}`)]]);

test('all locked resources reject corruption before executing the engine', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-probe-integrity-'));
  try {
    const entry = lock.engine.find(item => item.name.endsWith('.wasm'));
    const file = path.join(root, entry.sha256);
    copyFileSync(path.join(bundleRoot, entry.sha256), file);
    const bytes = readFileSync(file);
    bytes[128] ^= 1;
    writeFileSync(file, bytes);
    assert.throws(() => verifiedBytes(root, entry), /integrity failed/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('generated inputs reject filesystem links', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-probe-links-'));
  try {
    mkdirSync(path.join(root, 'inputs'));
    mkdirSync(path.join(root, 'outside'));
    writeFileSync(path.join(root, 'inputs/book.tex'), 'not executed');
    symlinkSync(path.join(root, 'outside'), path.join(root, 'inputs/assets'), process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => loadInputs(path.join(root, 'inputs')), /Unsafe generated PDF input/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('guest host files, environment and shell are inaccessible; socket modules are unavailable', { timeout: 60_000 }, async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-probe-host-'));
  const sentinel = path.join(root, 'sentinel.txt');
  const output = path.join(root, 'forbidden.txt');
  writeFileSync(sentinel, 'HOST-ONLY');
  process.env.ZEL_PDF_PROBE_SENTINEL = 'HOST-ONLY';
  try {
    const literal = file => JSON.stringify(file.replaceAll('\\', '/'));
    const result = await runPass(bundle, tex(`\\directlua{
      assert(io.open(${literal(sentinel)}, 'r') == nil)
      assert(io.open(${literal(output)}, 'w') == nil)
      assert(os.getenv('ZEL_PDF_PROBE_SENTINEL') == nil)
      assert(not os.execute('echo forbidden'))
      assert(not pcall(require, 'socket.core'))
      assert(not pcall(require, 'socket'))
      local f = assert(io.open('virtual-only.txt', 'w'))
      f:write('virtual file'); f:close()
    }Isolated guest.`));
    assert(Buffer.from(result.pdf).subarray(0, 5).equals(Buffer.from('%PDF-')));
    assert.equal(readFileSync(sentinel, 'utf8'), 'HOST-ONLY');
    assert(!existsSync(output));
  } finally {
    delete process.env.ZEL_PDF_PROBE_SENTINEL;
    rmSync(root, { recursive: true, force: true });
  }
});

test('virtual file growth is bounded before a large sparse allocation', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(`\\directlua{
    local f = assert(io.open('large.bin', 'w'))
    f:seek('set', 536870912); f:write('x'); f:close()
  }After the sparse write.`)), /virtual filesystem budget exceeded/);
});

test('first writes to many new files share the filesystem budget', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(`\\directlua{
    local chunk = string.rep('x', 8388608)
    for i = 1, 40 do
      local f = assert(io.open('newfile' .. i, 'w'))
      f:setvbuf('no'); f:write(chunk); f:close()
    end
  }After the first writes.`)), /virtual filesystem budget exceeded/);
});

test('console growth is bounded', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(`\\directlua{
    for i = 1, 1100 do texio.write_nl('term', string.rep('x', 8192)) end
  }`), 30_000, 32 * 1024), /log budget exceeded/);
});

test('an unterminated console line is bounded before buffering it', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(`\\directlua{
    io.stdout:setvbuf('no')
    io.stdout:write(string.rep('x', 65536)); io.stdout:flush()
  }After console output.`), 30_000, 32 * 1024), /log budget exceeded \(raw console\)/);
});

test('guest heap exhaustion fails instead of growing without limit', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(`\\directlua{
    local chunks = {}
    for i = 1, 1024 do chunks[i] = string.rep('x', 1048576) end
  }`)), /not enough memory|out of memory/);
});

test('an endless guest is terminated, and a later pass still works', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex('\\directlua{while true do end}'), 15_000), /timed out/);
  const result = await runPass(bundle, tex('Recovered after cancellation.'));
  assert(result.pdf.length > 1000);
});

test('a typesetting error never becomes a successful partial PDF or replaces output', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex('Before the error.\\undefinedZellijCommand')), /LuaHBTeX exited/);
  const root = mkdtempSync(path.join(tmpdir(), 'zel-probe-output-'));
  try {
    const input = path.join(root, 'input');
    mkdirSync(input);
    writeFileSync(path.join(input, 'book.tex'), tex('A page.').get('book.tex'));
    const output = path.join(root, 'output');
    mkdirSync(output);
    writeFileSync(path.join(output, 'keep.txt'), 'unchanged');
    await assert.rejects(compile(bundleRoot, input, output), /output already exists/);
    assert.equal(readFileSync(path.join(output, 'keep.txt'), 'utf8'), 'unchanged');
    writeFileSync(path.join(input, 'book.tex'), tex('\\undefinedZellijCommand').get('book.tex'));
    await assert.rejects(compile(bundleRoot, input, path.join(root, 'new-output')), /LuaHBTeX exited/);
    assert(!existsSync(path.join(root, 'new-output')));
  } finally { rmSync(root, { recursive: true, force: true }); }
});
