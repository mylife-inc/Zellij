// Native process/HTTP acceptance on Linux, macOS and Windows. No GUI or daemon.
import assert from 'node:assert/strict';
import { spawn, spawnSync, execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const windows = process.platform === 'win32';
const zel = path.resolve(`cli/target/debug/zel${windows ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-dev-options-'));
const bin = path.join(root, 'bin');
const browserExe = path.join(bin, `browser with spaces${windows ? '.exe' : ''}`);
const consoleExe = path.join(bin, 'console.exe');
const active = new Set();
const text = file => readFile(file, 'utf8').catch(() => '');
const until = async (check, label, limit = 15000) => {
  const end = Date.now() + limit;
  while (!await check()) { if (Date.now() > end) throw new Error(typeof label === 'function' ? label() : label); await delay(50); }
};
const reserve = async (host = '127.0.0.1', port = 0) => {
  const server = net.createServer();
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, host, resolve); });
  const selected = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return selected;
};
async function fixture(name, renderer = 'hosted', mode = '') {
  const folder = path.join(root, name);
  await mkdir(path.join(folder, 'content'), { recursive: true });
  await writeFile(path.join(folder, 'zel.yaml'), `version: 1\nrenderer: ${renderer}\n`);
  await writeFile(path.join(folder, 'content', renderer === 'hosted' ? 'site.yaml' : 'index.md'), renderer === 'hosted' ? 'name: Fixture\n' : '# Portable development\n');
  const record = path.join(folder, 'browser.txt');
  const env = { ...process.env, ZELLIJ_DEV_FIXTURE: folder, ZELLIJ_DEV_MODE: mode, ZELLIJ_DEV_BROWSER_LOG: record };
  const replaced = new Set(['PATH', 'DOCKER_CONTEXT', 'DOCKER_HOST', 'CONTAINER_CONNECTION', 'CONTAINER_HOST', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE']);
  for (const key of Object.keys(env)) if (replaced.has(key.toUpperCase())) delete env[key];
  Object.assign(env, { PATH: bin, CONTAINER_CONNECTION: 'zellij-dev-fixture' });
  const start = args => {
    const invocation = ['dev', ...args];
    const child = spawn(windows ? consoleExe : zel, windows ? [zel, ...invocation] : invocation, { cwd: folder, env, stdio: ['pipe', 'pipe', 'pipe'] });
    const state = { child, folder, record, stdout: '', stderr: '', exit: undefined };
    child.stdout.on('data', b => state.stdout += b); child.stderr.on('data', b => state.stderr += b);
    child.once('error', error => { state.exit = { error }; });
    child.once('close', (code, signal) => { state.exit = { code, signal }; });
    active.add(state); return state;
  };
  return { folder, record, env, start };
}
const detail = s => `${s.stdout}\n${s.stderr}\n${JSON.stringify(s.exit)}`;
async function response(url, label) {
  try { return await fetch(url, { signal: AbortSignal.timeout(5000) }); }
  catch (error) { throw new Error(`${label}: HTTP request failed at ${url}`, { cause: error }); }
}
async function fragmentedRequest(url) {
  const parsed = new URL(url);
  const socket = net.createConnection({ host: parsed.hostname, port: Number(parsed.port) });
  let received = '';
  socket.on('data', data => received += data);
  const complete = new Promise((resolve, reject) => {
    socket.once('error', reject);
    socket.once('end', resolve);
    socket.setTimeout(5000, () => socket.destroy(new Error('Fragmented fixture request timed out')));
  });
  // Attach rejection handling while deliberately pausing between TCP chunks.
  void complete.catch(() => {});
  try {
    socket.write(`GET ${parsed.pathname} HTTP/1.1\r\nHost: ${parsed.host}\r\nConnection: close\r\n`);
    await delay(100);
    assert.equal(received, '', 'Fixture must wait for complete request headers');
    socket.write('\r\n');
    await complete;
    assert.match(received, /^HTTP\/1.1 200 /);
    assert.match(received, /"revision":"first"/);
  } finally { socket.destroy(); }
}
async function stop(s) {
  if (!s.exit) { if (windows) s.child.stdin.end('stop\n'); else s.child.kill('SIGINT'); }
  await until(() => s.exit, () => `Shutdown timed out: ${detail(s)}`);
  assert.equal(s.exit.code, 0, detail(s));
  active.delete(s);
}
try {
  await mkdir(bin);
  for (const [source, output] of [['dev-engine', path.join(bin, windows ? 'podman.exe' : 'podman')], ['dev-browser', browserExe], ...(windows ? [['preview-console', consoleExe]] : [])]) {
    execFileSync('rustc', ['--edition=2024', `scripts/fixtures/${source}.rs`, '-o', output], { stdio: 'pipe' });
  }
  for (const host of ['127.0.0.1', '0.0.0.0', '::1', '::']) {
    const f = await fixture(`portable-${host.replaceAll(':', '_')}`, 'portable');
    const s = f.start(['--port', '0', '--host', host, '--browser', browserExe]);
    await until(async () => await text(f.record) || s.exit, () => detail(s));
    assert(!s.exit, detail(s));
    const url = (await text(f.record)).trim();
    assert.equal(new URL(url).hostname, host.includes(':') ? '[::1]' : '127.0.0.1');
    assert.match(await (await response(url, `portable ${host}`)).text(), /Portable development/);
    await stop(s);
    await reserve(host.includes(':') ? '::1' : '127.0.0.1', Number(new URL(url).port));
  }
  for (const [name, host, mode, opening] of [
    ['hosted-default', undefined, '', false],
    ['hosted-localhost', 'localhost', '', true],
    ['hosted-v6', '::1', '', true],
    ['hosted-wildcard', '::', 'stop-fails', true],
    ['hosted-cancel-startup', '127.0.0.1', 'slow', true],
    ['hosted-wrong-session', '127.0.0.1', 'wrong-session', true],
    ['hosted-browser-failure', '127.0.0.1', 'browser-failure', true],
  ]) {
    const f = await fixture(name, 'hosted', mode);
    const local = host?.includes(':') ? '::1' : '127.0.0.1';
    const port = await reserve(local);
    const s = f.start(['--port', String(port), ...(host ? ['--host', host] : []), ...(host === 'localhost' ? ['--open'] : []), ...(opening ? ['--browser', mode === 'browser-failure' ? path.join(bin, 'missing-browser') : browserExe] : [])]);
    await until(async () => await text(path.join(f.folder, 'listening')) || s.exit, () => detail(s));
    assert(!s.exit, detail(s));
    assert.equal(await text(f.record), '', 'Do not open before preparation');
    const args = await text(path.join(f.folder, 'args'));
    assert(args.includes(`${host?.includes(':') ? `[${host}]` : host === 'localhost' ? '127.0.0.1' : host ?? '0.0.0.0'}:${port}:3000`), args);
    if (mode !== 'slow') await writeFile(path.join(f.folder, 'allow-ready'), 'yes');
    if (mode === 'wrong-session' || mode === 'slow') {
      await delay(1800);
      assert.equal(await text(f.record), '', 'Do not open an unready or unrelated service');
      assert(!s.stdout.includes('Serving at '), detail(s));
    } else {
      await until(() => s.stdout.includes('Serving at ') || s.exit, () => detail(s));
      assert(!s.exit, detail(s));
      const url = `http://${local === '::1' ? '[::1]' : local}:${port}/products/manual/`;
      assert(s.stdout.includes(`Serving at ${url}`), detail(s));
      if (name === 'hosted-default') await fragmentedRequest(`${url}_zellij/dev-revision.json`);
      if (opening && mode !== 'browser-failure') {
        await until(async () => await text(f.record), () => detail(s));
        assert.equal(await text(f.record), `${url}\n`);
        await delay(1100);
        assert.equal(await text(f.record), `${url}\n`, 'Open once per session');
      } else if (mode === 'browser-failure') {
        await until(() => s.stderr.includes('could not open the browser'), () => detail(s));
        assert.equal((await response(`${url}_zellij/dev-revision.json`, name)).status, 200);
      } else assert.equal(await text(f.record), '');
    }
    await stop(s);
    assert.equal(await text(path.join(f.folder, 'stopped')), mode === 'stop-fails' ? 'rm' : 'stop');
    await reserve(local, port);
  }
  const f = await fixture('invalid');
  const quiet = await fixture('quiet');
  await writeFile(path.join(quiet.folder, 'allow-ready'), 'yes');
  const q = quiet.start(['--quiet', '--port', String(await reserve()), '--browser', browserExe]);
  await until(async () => await text(quiet.record) || q.exit, () => detail(q));
  assert(!q.exit, detail(q));
  await stop(q);
  assert.equal(q.stdout, ''); assert.equal(q.stderr, '');
  for (const host of ['http://127.0.0.1', 'localhost:3000', 'example.com', '127.0.0.1/path', '[::1]']) {
    const result = spawnSync(zel, ['dev', '--host', host], { cwd: f.folder, env: f.env, encoding: 'utf8' });
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /IPv4\/IPv6/);
    assert.equal(await text(path.join(f.folder, 'args')), '');
  }
  for (const [mode, code] of [['fail', 3], ['empty-mount', 4]]) {
    const failing = await fixture(`initial-${mode}`, 'hosted', mode);
    const temporary = path.join(failing.folder, 'tmp');
    await mkdir(temporary);
    for (const key of Object.keys(failing.env)) if (['TMPDIR', 'TMP', 'TEMP'].includes(key.toUpperCase())) delete failing.env[key];
    for (const key of ['TMPDIR', 'TMP', 'TEMP']) failing.env[key] = temporary;
    const failed = failing.start(['--port', String(await reserve()), '--browser', browserExe]);
    await until(() => failed.exit, () => detail(failed));
    assert.equal(failed.exit.code, code, detail(failed));
    assert.equal(await text(failing.record), '');
    const name = await text(path.join(failing.folder, 'name'));
    assert.match(name, /^zel-dev-/);
    await assert.rejects(stat(path.join(temporary, name)), { code: 'ENOENT' }, 'failure releases the temporary session identity');
    active.delete(failed);
  }
  const occupied = net.createServer();
  await new Promise(resolve => occupied.listen(0, '127.0.0.1', resolve));
  try {
    const conflict = await fixture('occupied-port');
    const s = conflict.start(['--host', '127.0.0.1', '--port', String(occupied.address().port), '--browser', browserExe]);
    await until(() => s.exit, () => detail(s));
    assert.notEqual(s.exit.code, 0, detail(s));
    assert.equal(await text(conflict.record), ''); active.delete(s);
  } finally { await new Promise(resolve => occupied.close(resolve)); }
  console.log('Native dev host/browser readiness, IPv4/IPv6, failures and Ctrl-C cleanup passed.');
} finally {
  for (const s of active) { try { await stop(s); } catch { s.child.kill('SIGKILL'); } }
  await rm(root, { recursive: true, force: true });
}
