// Exercise actual native serve/view processes, HTTP clients, browser rendering,
// launch failures, and real Ctrl-C. Never launch the machine's default browser.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, open, readFile, readdir, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import { chromium, webkit, expect } from '@playwright/test';
import { checkWatch } from './preview-watch.mjs';
import { checkSiteLayout } from './preview-site-layout.mjs';

const windows = process.platform === 'win32';
const binary = path.resolve(process.argv[2] || `cli/target/debug/zel${windows ? '.exe' : ''}`);
const temp = await mkdtemp(path.join(tmpdir(), 'zellij-preview-'));
const source = path.join(temp, 'Markdown 日本');
const output = path.join(temp, 'Published docs');
const browserExe = path.join(temp, `Fake browser 日本${windows ? '.exe' : ''}`);
const consoleExe = path.join(temp, 'console-runner.exe');
const record = path.join(temp, 'browser-url.txt');
const env = { ...process.env, PATH: path.join(temp, 'no-external-tools'), ZELLIJ_PREVIEW_RECORD: record };
const active = new Set();
const until = async (condition, description, timeout = 15000) => {
  const deadline = Date.now() + timeout;
  while (!await condition()) { if (Date.now() > deadline) throw new Error(`Timed out: ${description}`); await delay(30); }
};
const exists = async file => readFile(file).then(() => true, () => false);
const snapshot = async root => {
  const items = {};
  for (const entry of await readdir(root, { recursive: true, withFileTypes: true })) {
    if (entry.isFile()) {
      const file = path.join(entry.parentPath, entry.name);
      items[path.relative(root, file)] = createHash('sha256').update(await readFile(file)).digest('hex');
    }
  }
  return items;
};
function failure(args, overrides = {}) {
  let error;
  try { execFileSync(binary, args, { env: { ...env, ...overrides }, encoding: 'utf8', stdio: 'pipe', timeout: 15000 }); }
  catch (caught) { error = caught; }
  assert(error && error.status !== 0, `Expected failure: ${args.join(' ')}`);
  assert(!String(error.stdout).includes('Serving at '));
  return String(error.stderr);
}
async function start(args, overrides = {}, cwd) {
  const child = spawn(windows ? consoleExe : binary, windows ? [binary, ...args] : args, {
    env: { ...env, ...overrides }, cwd, stdio: ['pipe', 'pipe', 'pipe'],
  });
  const state = { child, stdout: '', stderr: '', exit: undefined, stopping: false };
  active.add(state);
  child.stdout.on('data', bytes => state.stdout += bytes);
  child.stderr.on('data', bytes => state.stderr += bytes);
  child.on('error', error => state.stderr += error.message);
  state.closed = new Promise(resolve => child.once('close', (code, signal) => { state.exit = { code, signal }; resolve(state.exit); }));
  await until(() => /Serving at (http:\/\/127\.0\.0\.1:\d+\/\S*)/.test(state.stdout) || state.exit, `server readiness: ${args.join(' ')}`);
  assert(!state.exit, `Server exited before readiness: ${state.stderr}\n${state.stdout}`);
  state.url = state.stdout.match(/Serving at (http:\/\/127\.0\.0\.1:\d+\/\S*)/)[1];
  return state;
}
async function stop(state) {
  if (!state.stopping && !state.exit) {
    state.stopping = true;
    if (windows) state.child.stdin.end('stop\n'); else state.child.kill('SIGINT');
  }
  const exit = await Promise.race([state.closed, delay(12000, undefined, { ref: false }).then(() => { throw new Error(`Server did not stop: ${state.stderr}`); })]);
  active.delete(state);
  assert.equal(exit.code, 0, `Shutdown failed: ${state.stderr}\n${state.stdout}`);
  assert(state.stdout.includes('Stopped.'), 'The Ctrl-C handler completed, not a forced process kill');
  const { hostname, port } = new URL(state.url);
  await until(() => new Promise(resolve => {
    const socket = net.connect({ host: hostname, port: Number(port) });
    socket.once('error', () => resolve(true));
    socket.once('connect', () => { socket.destroy(); resolve(false); });
  }), 'listener released after Ctrl-C', 3000);
}
async function body(url) {
  const response = await fetch(url); assert.equal(response.status, 200, url); return response.text();
}
try {
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-browser.rs', '-o', browserExe], { stdio: 'pipe' });
  if (windows) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleExe], { stdio: 'pipe' });
  await mkdir(path.join(source, 'guide'), { recursive: true });
  await mkdir(path.join(source, 'assets'));
  await writeFile(path.join(source, 'README.md'), '# Preview handbook\n\n[Next](guide/next.md#nebula)\n\n![Logo](assets/logo.svg)\n');
  await writeFile(path.join(source, 'guide/next.md'), '# Next page\n\n## Nebula\n\nLocal search works over HTTP too. [Home](../README.md)\n');
  await writeFile(path.join(source, 'assets/logo.svg'), '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><rect width="100" height="40" fill="green"/></svg>');
  const sourceBefore = await snapshot(source);
  execFileSync(binary, ['build', source, '--out', output], { env, stdio: 'pipe' });
  const outputBefore = await snapshot(output);
  assert.match(failure(['serve', source]), /index.html/);
  assert.match(failure(['serve', output, '--base-path', '/bad/../path']), /base-path/);
  assert.match(failure(['view', source, '--no-open', '--browser', browserExe]), /cannot be used/);
  assert.match(failure(['view', source, '--json']), /supported only/);

  const server = await start(['serve', output, '--port', '0', '--base-path', '/products/docs']);
  assert(server.url.endsWith('/products/docs/'));
  assert((await body(server.url)).includes('Preview handbook'));
  assert((await body(`${server.url}guide/next`)).includes('Next page'));
  const image = await fetch(`${server.url}assets/logo.svg`);
  assert.equal(image.headers.get('content-type'), 'image/svg+xml; charset=utf-8');
  assert.deepEqual(Buffer.from(await image.arrayBuffer()), await readFile(path.join(output, 'assets/logo.svg')));
  const head = await fetch(`${server.url}_zellij/site.js`, { method: 'HEAD' });
  assert.equal(head.status, 200); assert.equal(await head.text(), '');
  assert.equal((await fetch(new URL('/', server.url))).status, 404);
  assert.equal((await fetch(server.url, { method: 'POST' })).status, 405);
  const occupied = new URL(server.url).port;
  assert.match(failure(['view', source, '--out', output, '--port', occupied, '--browser', browserExe]), /already in use/);
  assert.equal(await exists(record), false);
  assert.deepEqual(await snapshot(output), outputBefore, 'Port conflict must fail before rebuilding');

  const outside = path.join(temp, 'outside'); await mkdir(outside); await writeFile(path.join(outside, 'secret.txt'), 'must not leak');
  await symlink(outside, path.join(output, 'linked'), windows ? 'junction' : 'dir');
  assert.equal((await fetch(`${server.url}linked/secret.txt`)).status, 403);
  // Remove only the link, not its target.
  await rm(path.join(output, 'linked'));
  for (const engine of process.env.ZELLIJ_PORTABLE_BROWSER?.split(',') || ['chromium', 'webkit']) {
    const browser = await ({ chromium, webkit })[engine].launch();
    try {
      const context = await browser.newContext({ serviceWorkers: 'block' });
      const errors = []; const requests = [];
      await context.route(/^https?:/, route => new URL(route.request().url()).origin === new URL(server.url).origin ? route.continue() : route.abort());
      context.on('request', request => requests.push(request.url()));
      const page = await context.newPage(); page.on('pageerror', error => errors.push(error.message));
      await page.goto(server.url);
      await expect(page.getByRole('heading', { name: 'Preview handbook' })).toBeVisible();
      assert.equal(await page.locator('article img').evaluate(img => img.complete && img.naturalWidth > 0), true);
      await page.keyboard.press('Control+k'); await page.getByRole('searchbox').fill('nebula');
      await page.locator('#_zel-results a').click();
      await expect(page.getByRole('heading', { name: 'Next page', exact: true })).toBeVisible();
      assert.equal(new URL(page.url()).hash, '#nebula');
      assert(new URL(page.url()).pathname.startsWith('/products/docs/'));
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Preview handbook' })).toBeVisible();
      assert.deepEqual(errors, []);
      assert(requests.every(url => new URL(url).origin === new URL(server.url).origin));
      await context.close();
      console.log(`${engine}: HTTP preview under a prefix, routes, media, scripts and search passed.`);
    } finally { await browser.close(); }
  }
  // A slow reader and a client with an unfinished request must not trap Ctrl-C.
  const large = await open(path.join(output, 'large.bin'), 'w'); await large.truncate(64 * 1024 * 1024); await large.close();
  const slow = net.connect(Number(occupied), '127.0.0.1'); slow.on('error', () => {});
  await new Promise(resolve => slow.once('connect', resolve)); slow.pause();
  slow.write(`GET /products/docs/large.bin HTTP/1.1\r\nHost: 127.0.0.1:${occupied}\r\n\r\n`);
  const partial = net.connect(Number(occupied), '127.0.0.1'); partial.on('error', () => {});
  await new Promise(resolve => partial.once('connect', resolve)); partial.write('GET /');
  await stop(server); slow.destroy(); partial.destroy();
  await rm(path.join(output, 'large.bin'));
  assert.deepEqual(await snapshot(output), outputBefore, 'Serving must not alter exported bytes');

  const view = await start(['view', source, '--out', output, '--port', '0', '--no-open', '--name', 'Company docs', '--theme', 'forest']);
  const html = await body(view.url); assert(html.includes('Company docs')); assert(html.includes('data-zellij-theme="forest"'));
  assert(!html.includes('_zellij/preview.js'));
  assert.equal((await fetch(`${view.url}_zellij/preview.json`)).status, 404);
  assert.equal(await exists(record), false); await stop(view);
  const opened = await start(['view', source, '--port', '0', '--browser', browserExe]);
  await until(() => exists(record), 'browser executable successfully requested the ready site');
  assert.equal(await readFile(record, 'utf8'), opened.url); assert(await exists(`${source}-site/index.html`));
  await stop(opened); await rm(record);
  for (const [executable, overrides] of [[path.join(temp, 'missing browser'), {}], [browserExe, { ZELLIJ_PREVIEW_FAIL: '1' }]]) {
    const failedOpen = await start(['view', source, '--out', output, '--port', '0', '--browser', executable], overrides);
    await until(() => failedOpen.stderr.includes('server is still running'), 'browser-launch failure diagnostic');
    assert((await body(failedOpen.url)).includes('Preview handbook')); await stop(failedOpen);
  }
  await writeFile(path.join(source, 'bad.md'), '# Broken\n\n[Missing](missing.md)\n');
  const oldExport = await snapshot(output);
  assert.match(failure(['view', source, '--out', output, '--port', '0', '--browser', browserExe]), /no matching document/);
  assert.deepEqual(await snapshot(output), oldExport); assert.equal(await exists(record), false);
  await rm(path.join(source, 'bad.md'));
  assert.deepEqual(await snapshot(source), sourceBefore);
  console.log('Native serve/view: HTTP bytes/MIME/routes, prefix, port conflicts, containment, ready browser launch, no-open/failure recovery and real Ctrl-C passed.');
  await checkWatch({ temp, source, output, start, stop, snapshot, until });
  await checkSiteLayout({ temp, start, stop, snapshot, until });
  await checkSiteLayout({ temp, start, stop, snapshot, until, explicitConfig: true });
  const project = path.join(temp, 'Software project');
  const docs = path.join(project, 'docs/zellij');
  await mkdir(path.join(docs, 'content'), { recursive: true });
  await writeFile(path.join(docs, 'zel.yaml'), 'version: 1\nrenderer: portable\nbuild: {out: preview}\npreview: {port: 0}\n');
  await writeFile(path.join(docs, 'content/index.md'), '# Development start\n');
  const dev = await start(['dev'], { DOCKER_HOST: 'not-a-runtime', DOCKER_CONTEXT: 'also-invalid' }, project);
  assert((await body(dev.url)).includes('Development start'));
  assert.equal(await exists(record), false, 'dev never launches the default browser');
  await writeFile(path.join(docs, 'content/index.md'), '# Development edit\n');
  await until(async () => (await body(dev.url)).includes('Development edit'), 'dev rebuild');
  await stop(dev);
  assert((await readFile(path.join(docs, 'preview/index.html'), 'utf8')).includes('Development edit'));
  console.log('Portable dev: repository default, configured output/ephemeral port, no external tools/browser, rebuild and Ctrl-C passed.');
} finally {
  for (const state of active) {
    try { await stop(state); } catch { state.child.kill(); }
  }
  await rm(temp, { recursive: true, force: true });
}
