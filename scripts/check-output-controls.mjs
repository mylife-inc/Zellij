// Quiet mode must still rebuild, recover, serve HTTP, and stop on Ctrl-C.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const windows = process.platform === 'win32';
const zel = path.resolve(`cli/target/debug/zel${windows ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-quiet-preview-'));
let child, exit, closed;
let stdout = '', stderr = '';
async function until(predicate, label) {
  for (let i = 0; i < 200; i++) {
    if (await predicate()) return;
    assert(!exit, `Preview exited: ${JSON.stringify(exit)}\n${stdout}\n${stderr}`);
    await delay(100);
  }
  throw new Error(`${label}\n${stdout}\n${stderr}`);
}
try {
  const runner = path.join(root, 'console.exe');
  if (windows) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', runner], { stdio: 'inherit' });
  const reservation = createServer();
  await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
  const port = reservation.address().port;
  await new Promise((resolve, reject) => reservation.close(error => error ? reject(error) : resolve()));
  await mkdir(path.join(root, 'content'));
  await writeFile(path.join(root, 'selected.yaml'), 'version: 1\nrenderer: portable\n');
  const source = path.join(root, 'content/index.md');
  const output = path.join(root, 'site/index.html');
  await writeFile(source, '# Original\n\nFirst quiet build.\n');
  const args = ['--quiet', '--no-color', 'dev', '--config', 'selected.yaml', '--port', String(port)];
  child = spawn(windows ? runner : zel, windows ? [zel, ...args] : args, {
    cwd: root, env: { ...process.env, PATH: path.join(root, 'no-tools') }, stdio: ['pipe', 'pipe', 'pipe'],
  });
  child.stdout.on('data', data => stdout += data);
  child.stderr.on('data', data => stderr += data);
  closed = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => { exit = { code, signal }; resolve(exit); }); });
  const contains = async text => (await readFile(output, 'utf8').catch(() => '')).includes(text);
  await until(() => contains('First quiet build.'), 'quiet initial build');
  const response = await fetch(`http://127.0.0.1:${port}/`);
  assert.equal(response.status, 200);
  assert((await response.text()).includes('First quiet build.'));
  await writeFile(source, '# Original\n\nSecond quiet build.\n');
  await until(() => contains('Second quiet build.'), 'quiet rebuild');
  assert.equal(stdout, ''); assert.equal(stderr, '');
  const previous = await readFile(output);
  await writeFile(source, '# Broken\n\n[Bad](missing.md)\n');
  await until(() => stderr.includes('Rebuild failed'), 'quiet failures must remain visible');
  assert.deepEqual(await readFile(output), previous);
  await writeFile(source, '# Recovered\n\nRecovered quiet build.\n');
  await until(() => contains('Recovered quiet build.'), 'quiet recovery');
  if (windows) child.stdin.end('stop\n'); else child.kill('SIGINT');
  await until(() => exit !== undefined, 'quiet Ctrl-C cleanup');
  assert.equal(exit.code, 0, stderr);
  assert.equal(stdout, ''); assert(!stderr.includes('\u001b'));
  console.log('Quiet preview: build, HTTP, rebuild, failure preservation, recovery and Ctrl-C passed.');
} finally {
  if (child && !exit) { if (windows) child.stdin.end('stop\n'); else child.kill('SIGTERM'); await closed; }
  await rm(root, { recursive: true, force: true, maxRetries: 5 });
}
