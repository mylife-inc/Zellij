// Real Windows deny-delete handles; no production changes or installed tools.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { build } from 'esbuild';

assert.equal(process.platform, 'win32', 'This acceptance requires a real Windows host');
const root = await mkdtemp(path.join(os.tmpdir(), 'zellij-windows-lock-'));
const file = path.join(root, 'marker café.json');
const locker = path.join(root, 'lock.ps1');
let child;
let closed;
let startupError;
async function until(check, label) {
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    if (startupError) throw startupError;
    if (await check()) return;
    await delay(10);
  }
  throw new Error(`Windows lock fixture timed out: ${label}`);
}
async function hold() {
  let output = '', errors = '';
  startupError = undefined;
  child = spawn('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', locker, file], { stdio: ['pipe', 'pipe', 'pipe'] });
  closed = new Promise(resolve => child.once('close', (code, signal) => resolve({ code, signal })));
  child.on('error', error => { startupError = error; });
  child.stdin.on('error', error => { startupError = error; });
  child.stdout.on('data', bytes => { output += bytes; });
  child.stderr.on('data', bytes => { errors += bytes; });
  await until(() => {
    assert.equal(child.exitCode, null, errors);
    return output.includes('LOCKED');
  }, 'deny-delete handle acquired');
}
async function release() {
  if (!child) return;
  if (child.exitCode === null && child.signalCode === null) child.stdin.end('release\n');
  let timer;
  const result = await Promise.race([closed, new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error('Windows lock handle release timed out')), 10000);
  })]).finally(() => clearTimeout(timer));
  assert.equal(result.code, 0);
  child = undefined;
}
try {
  await build({ entryPoints: ['engine/packages/zellij/src/cli/write-prepared-file.ts'], bundle: true,
    platform: 'node', format: 'esm', outfile: path.join(root, 'writer.mjs') });
  const { writePreparedFile } = await import(pathToFileURL(path.join(root, 'writer.mjs')).href);
  await writeFile(locker, `$ErrorActionPreference = 'Stop'\n$stream = [System.IO.FileStream]::new($args[0], [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)\ntry { [Console]::WriteLine('LOCKED'); [Console]::ReadLine() | Out-Null } finally { $stream.Dispose() }\n`);
  await writeFile(file, 'previous');
  await hold();
  const once = path.join(root, 'once.tmp');
  await writeFile(once, 'baseline');
  await assert.rejects(rename(once, file), error => ['EPERM', 'EACCES', 'EBUSY'].includes(error.code));
  assert.equal(await readFile(file, 'utf8'), 'previous');
  await rm(once);
  await Promise.all([writePreparedFile(file, 'replacement'), delay(100).then(release)]);
  assert.equal(await readFile(file, 'utf8'), 'replacement');
  await hold();
  await assert.rejects(writePreparedFile(file, 'must not replace'), error => ['EPERM', 'EACCES', 'EBUSY'].includes(error.code));
  assert.equal(await readFile(file, 'utf8'), 'replacement');
  assert(!(await readdir(root)).some(name => name.endsWith('.tmp')));
  await release();
  console.log('Windows prepared-file locks: one-shot rename fails; transient lock recovers; persistent lock preserves output and removes temporary files.');
} finally {
  if (child) {
    try { await release(); } catch { child?.kill(); await closed; }
  }
  await rm(root, { recursive: true, force: true });
}
