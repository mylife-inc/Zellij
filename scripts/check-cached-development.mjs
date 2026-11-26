// Native acceptance: actual Next server, no user-installed toolchain on PATH.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { createServer } from 'node:http';
import { createServer as reserve } from 'node:net';
import { createReadStream } from 'node:fs';
import { mkdir, mkdtemp, readFile, readdir, rename, rm, stat, writeFile } from 'node:fs/promises';
import { once } from 'node:events';
import path from 'node:path';
import os from 'node:os';
import { setTimeout as delay } from 'node:timers/promises';
const repo = path.resolve(import.meta.dirname, '..');
const binary = path.resolve(process.argv[2] ?? `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const version = JSON.parse(execFileSync(binary, ['version', '--json'], { encoding: 'utf8' })).cli.version;
const platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`;
const root = await mkdtemp(path.join(os.tmpdir(), 'zel dev 日本 '));
const downloads = new Map();
for (const [kind, suffix] of [['renderer', `-${platform}`], ['sources', ''], ['development', `-${platform}`]]) {
  const name = `zel-${kind}-v${version}${suffix}.tar.gz`;
  const directory = kind === 'sources' ? 'source' : kind;
  for (const ending of ['', '.sha256']) downloads.set(name + ending, path.join(repo, `dist/${directory}-packages`, name + ending));
}
let requests = 0;
let stall, stalled = false;
const artifacts = createServer(async (req, res) => {
  requests++;
  const file = downloads.get(decodeURIComponent(req.url.slice(1)));
  if (!file) { res.writeHead(404); res.end(); return; }
  if (stall && req.url.includes('/zel-development-') &&
    (stall.startsWith('checksum') === req.url.endsWith('.sha256'))) {
    stalled = true;
    if (stall !== 'checksum-headers') { res.writeHead(200, { 'content-length': (await stat(file)).size }); res.write('partial'); }
    return;
  }
  try { res.writeHead(200, { 'content-length': (await stat(file)).size }); createReadStream(file).pipe(res); }
  catch (error) { res.writeHead(500); res.end(String(error)); }
});
await new Promise(resolve => artifacts.listen(0, '127.0.0.1', resolve));
const url = `http://127.0.0.1:${artifacts.address().port}`;
const publicBase = process.env.ZEL_TEST_DEV_BASE_URL;
const env = { ...process.env, PATH: path.join(root, 'empty'), ZELLIJ_HOME: path.join(root, 'resources café'),
  ZEL_ENGINE_BASE_URL: publicBase ?? url, ZEL_SOURCE_BASE_URL: publicBase ?? url, ZEL_DEV_BASE_URL: publicBase ?? url };
delete env.ZEL_ENGINE_CACHE;
delete env.ZEL_DEV_BACKEND;
await mkdir(env.PATH);
const content = path.join(root, 'content');
await mkdir(path.join(content, '01-guide'), { recursive: true });
await mkdir(path.join(content, 'assets'));
await writeFile(path.join(root, 'zel.yaml'), 'version: 1\nrenderer: hosted\n');
await writeFile(path.join(content, 'site.yaml'), 'name: Cached development\nbasePath: /docs\nhome: 01-guide\n');
await writeFile(path.join(content, '01-guide/_sidebar.yaml'), 'route: /manual\nnav: [index]\n');
const md = token => `# ${token}\n\nA searchable ${token} passage.\n\n![Asset](/_zellij/assets/test.svg)\n\n$x^2$\n`;
const doc = path.join(content, '01-guide/index.mdx');
await writeFile(doc, md('Original'));
const svg = width => `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="20"/>`;
await writeFile(path.join(content, 'assets/test.svg'), svg(40));
const windows = process.platform === 'win32';
const helper = path.join(root, 'console.exe');
if (windows) execFileSync('rustc', ['--edition=2024', path.join(repo, 'scripts/fixtures/preview-console.rs'), '-o', helper]);
let child, done, log = '', port;
async function command(args, overrides = {}) {
  const p = spawn(binary, args, { env: { ...env, ...overrides }, stdio: ['ignore', 'pipe', 'pipe'] });
  let output = ''; p.stdout.on('data', b => output += b); p.stderr.on('data', b => output += b);
  const timeout = setTimeout(() => p.kill('SIGKILL'), 180000);
  try { const [code, signal] = await once(p, 'close'); return { code, signal, output }; }
  finally { clearTimeout(timeout); }
}
async function until(check, label, timeout = 180000) {
  const start = Date.now(); let last;
  while (Date.now() - start < timeout) {
    if (child?.exitCode !== null && child?.exitCode !== undefined) throw new Error(`Dev exited: ${log}`);
    try { if (await check()) return; } catch (error) { last = error; }
    await delay(100);
  }
  throw new Error(`${label}: ${last ?? ''}\n${log}`);
}
async function launch(offline = false) {
  const reservation = reserve(); reservation.listen(0, '127.0.0.1'); await once(reservation, 'listening');
  port = reservation.address().port; await new Promise(resolve => reservation.close(resolve));
  log = '';
  const args = ['dev', root, '--host', '127.0.0.1', '--port', String(port), '--features', 'all', ...(offline ? ['--offline'] : [])];
  child = spawn(windows ? helper : binary, windows ? [binary, ...args] : args, { env, stdio: ['pipe', 'pipe', 'pipe'] });
  done = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => resolve({code, signal})); });
  child.stdout.on('data', data => log += data); child.stderr.on('data', data => log += data);
  await until(() => log.includes(`Serving at http://127.0.0.1:${port}/docs/`), 'session readiness');
}
const get = name => fetch(`http://127.0.0.1:${port}/docs/${name}`, { signal: AbortSignal.timeout(10000) });
const index = async () => (await get('_zellij/search-index.json')).json();
async function stop(assertions = true) {
  if (!child) return;
  const active = child; child = undefined;
  if (active.exitCode === null && active.signalCode === null) {
    if (windows) active.stdin.write('stop\n'); else active.kill('SIGINT');
  }
  let timer;
  const result = await Promise.race([done, new Promise((_, reject) => {
    timer = setTimeout(() => { active.kill('SIGKILL'); reject(new Error(`Shutdown timed out: ${log}`)); }, 10000);
  })]).finally(() => clearTimeout(timer));
  if (assertions) { assert.equal(result.code, 0, log); assert(log.includes('Stopped.'), log); }
  await until(() => new Promise(resolve => {
    const probe = reserve(); probe.once('error', () => resolve(false));
    probe.listen(port, '127.0.0.1', () => probe.close(() => resolve(true)));
  }), 'port released', 10000);
  assert.deepEqual(await readdir(path.join(env.ZELLIJ_HOME, 'cache/development-sessions')).catch(e => {
    if (e.code === 'ENOENT') return []; throw e;
  }), [], 'session cleanup');
}
try {
  const cold = await command(['dev', root, '--offline']);
  assert.equal(cold.code, 2, cold.output); assert.match(cold.output, /not cached/);
  assert.equal(requests, 0);
  await launch();
  assert.equal(requests, publicBase ? 0 : 6, 'three verified packages downloaded once');
  await until(async () => { const r = await get('manual'); return r.ok && (await r.text()).includes('Original'); }, 'initial Next page');
  const initial = await (await get('_zellij/dev-revision.json')).json();
  assert.match(initial.session, /^zel-dev-/); assert.equal(initial.basePath, '/docs');
  await writeFile(doc, md('Updated'));
  await writeFile(path.join(content, 'assets/test.svg'), svg(80));
  await until(async () => (await index()).some(e => e.text.includes('Updated')), 'search after edit');
  await until(async () => (await (await get('_zellij/assets/test.svg')).text()).includes('width="80"'), 'asset after edit');
  const good = await index(), revision = await (await get('_zellij/dev-revision.json')).json();
  await writeFile(doc, '# Invalid\n\n<Callout>\n');
  await until(() => log.includes('Preparation failed'), 'invalid edit diagnostic');
  assert.deepEqual(await index(), good); assert.deepEqual(await (await get('_zellij/dev-revision.json')).json(), revision);
  await writeFile(doc, md('Recovered'));
  await until(async () => (await index()).some(e => e.text.includes('Recovered')), 'recovery');
  await stop();
  console.log('Cached Next: cold startup, edits, invalid-edit recovery and shutdown passed.');
  if (!publicBase) {
    const dependencies = path.join(env.ZELLIJ_HOME, 'cache/development', `${version}-${platform}`);
    const saved = path.join(root, 'saved-development'); await rename(dependencies, saved);
    try {
      for (const mode of ['checksum-headers', 'checksum-body', 'archive-body']) {
        stall = mode; stalled = false; log = '';
        const args = ['dev', root, '--host', '127.0.0.1', '--port', String(port)];
        child = spawn(windows ? helper : binary, windows ? [binary, ...args] : args, { env, stdio: ['pipe', 'pipe', 'pipe'] });
        child.stdout.on('data', b => log += b); child.stderr.on('data', b => log += b);
        done = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => resolve({ code, signal })); });
        await until(() => stalled, `download reaches ${mode}`);
        await stop(false);
        assert.equal((await done).code, 0, `cancelled ${mode}: ${log}`);
        assert.deepEqual(await readdir(path.dirname(dependencies)), [], `cancelled ${mode} preserves no partial package`);
        assert.doesNotMatch(log, /Serving at/);
      }
    } finally { stall = undefined; await rename(saved, dependencies); }
    requests = 6; // Ignore the three deliberately stalled cancellation fixtures.
    console.log('Cached Next: checksum/header/archive interruption and partial-cache cleanup passed.');
  }
  await new Promise(resolve => artifacts.close(resolve));
  await launch(true);
  await until(async () => { const r = await get('manual'); return r.ok && (await r.text()).includes('Recovered'); }, 'offline cached Next page');
  assert.equal(requests, publicBase ? 0 : 6, 'offline restart makes no package requests');
  await stop();
  console.log('Cached Next: offline restart and shutdown passed.');
  assert.equal(await readFile(doc, 'utf8'), md('Recovered'), 'authored content unchanged by runtime');
  const collision = createServer((req, res) => { res.end('unrelated service'); });
  await new Promise(resolve => collision.listen(0, '127.0.0.1', resolve));
  try {
    const failed = await command(['dev', root, '--offline', '--host', '127.0.0.1', '--port', String(collision.address().port)]);
    assert.notEqual(failed.code, 0, failed.output); assert.equal(failed.signal, null, failed.output);
    assert.doesNotMatch(failed.output, /Serving at/);
    const response = await fetch(`http://127.0.0.1:${collision.address().port}/`);
    assert.equal(await response.text(), 'unrelated service');
  } finally { await new Promise(resolve => collision.close(resolve)); }
  await writeFile(doc, '# Invalid\n\n<Callout>\n');
  const invalid = await command(['dev', root, '--offline']);
  assert.notEqual(invalid.code, 0, invalid.output); assert.equal(invalid.signal, null, invalid.output);
  assert.doesNotMatch(invalid.output, /Serving at/);
  await writeFile(doc, md('Recovered'));
  const cachedRunner = path.join(env.ZELLIJ_HOME, 'cache/development', `${version}-${platform}`, 'run.mjs');
  const originalRunner = await readFile(cachedRunner);
  await writeFile(cachedRunner, 'throw new Error("must never execute");');
  const tampered = await command(['dev', root, '--offline']);
  assert.equal(tampered.code, 2, tampered.output); assert.match(tampered.output, /integrity verification/);
  await writeFile(cachedRunner, originalRunner);
  assert.deepEqual(await readdir(path.join(env.ZELLIJ_HOME, 'cache/development-sessions')), [], 'failed startup removes its session');
  if (process.env.ZEL_DEV_BROWSER === 'true') {
    const browser = spawn(process.execPath, [path.join(repo, 'scripts/check-dev-site.mjs'), '--cached', binary], {
      cwd: repo, env: { ...process.env, ZELLIJ_HOME: env.ZELLIJ_HOME }, stdio: 'inherit',
    });
    const [code] = await once(browser, 'exit'); assert.equal(code, 0, 'cached development browser acceptance');
  }
  console.log(JSON.stringify({ platform, cachedNextDev: true, systemTools: false, coldDownloads: requests, offlineRestart: true, editRecovery: true, portReleased: true, sessionsRemoved: true }));
} catch (error) { console.error(log); throw error; }
finally {
  if (child) await stop(false);
  artifacts.closeAllConnections(); artifacts.close();
  if (process.env.ZEL_DEV_TEST_KEEP) console.log(`Retained ${root}`);
  else await rm(root, { recursive: true, force: true });
}
