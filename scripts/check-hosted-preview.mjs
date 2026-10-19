// Native hosted-preview orchestration on every OS; --real uses the CI engine.
import assert from 'node:assert/strict';
import { spawn, execFileSync, spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import { chromium, expect } from '@playwright/test';
const real = process.argv.includes('--real');
const windows = process.platform === 'win32';
const zel = path.resolve(`cli/target/debug/zel${windows ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-hosted-preview-'));
const site = path.join(root, 'docs/zellij');
const content = path.join(site, 'content');
const output = path.join(site, 'site');
const bin = path.join(root, 'bin');
const consoleExe = path.join(bin, 'console.exe');
const browserExe = path.join(bin, `browser${windows ? '.exe' : ''}`);
const record = path.join(root, 'browser.txt');
const log = path.join(root, 'runtime.log');
const env = { ...process.env, ZELLIJ_PREVIEW_RECORD: record, ZELLIJ_PREVIEW_ENGINE_LOG: log, ZELLIJ_PREVIEW_EXPECT: '</html>' };
if (!real) {
  env.PATH = bin;
  for (const key of ['DOCKER_CONTEXT', 'DOCKER_HOST', 'CONTAINER_CONNECTION', 'CONTAINER_HOST', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE']) delete env[key];
}
const active = new Set();
let browser;
const until = async (predicate, label, limit = real ? 240000 : 15000) => {
  const end = Date.now() + limit;
  while (!await predicate()) { if (Date.now() > end) throw new Error(label); await delay(100); }
};
const exists = file => readFile(file).then(() => true, () => false);
const tree = async folder => {
  const result = {};
  for (const entry of await readdir(folder, { recursive: true, withFileTypes: true })) if (entry.isFile()) {
    const file = path.join(entry.parentPath, entry.name);
    result[path.relative(folder, file)] = (await readFile(file)).toString('base64');
  }
  return result;
};
async function edit(title, prefix = '/products/manual', mode = 'ok') {
  if (real) {
    await writeFile(path.join(content, 'site.yaml'), `name: Preview\nhome: guide\nbasePath: ${prefix}\nnav: {search: true}\n`);
    await writeFile(path.join(content, 'guide/index.mdx'), mode === 'fail' ? '---\ntitle: [\n---\n' : `# ${title}\n\n[Next](/guide/next)\n\n![Logo](/_zellij/assets/logo.svg)\n`);
  } else await writeFile(path.join(content, 'preview.txt'), `${title}\n${prefix}\n${mode}\n`);
}
function launch(args = ['view', '--watch', '--browser', browserExe]) {
  const child = spawn(windows ? consoleExe : zel, windows ? [zel, ...args] : args, { cwd: root, env, stdio: ['pipe', 'pipe', 'pipe'] });
  const state = { child, stdout: '', stderr: '', exit: undefined };
  child.stdout.on('data', b => state.stdout += b); child.stderr.on('data', b => state.stderr += b);
  state.closed = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => { state.exit = { code, signal }; resolve(state.exit); }); });
  active.add(state); return state;
}
async function ready(state) {
  await until(() => /Serving at (http:\/\/127\.0\.0\.1:\d+\/\S*)/.test(state.stdout) || state.exit, `readiness\n${state.stderr}`);
  assert(!state.exit, `${state.stdout}\n${state.stderr}`);
  state.url = state.stdout.match(/Serving at (http:\/\/127\.0\.0\.1:\d+\/\S*)/)[1];
  return state;
}
async function stop(state) {
  if (!state.exit) { if (windows) state.child.stdin.end('stop\n'); else state.child.kill('SIGINT'); }
  await until(() => state.exit, `shutdown\n${state.stdout}\n${state.stderr}`);
  assert.equal(state.exit.code, 0, state.stderr); assert(state.stdout.includes('Stopped.'));
  active.delete(state);
}
function fails(args, expected) {
  const result = spawnSync(zel, args, { cwd: root, env, encoding: 'utf8', timeout: real ? 240000 : 15000 });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, expected); assert(!result.stdout.includes('Serving at '));
}
try {
  await mkdir(bin); await mkdir(path.join(content, 'guide'), { recursive: true });
  await writeFile(path.join(site, 'zel.yaml'), 'version: 1\nrenderer: hosted\npreview: {port: 0}\n');
  await writeFile(path.join(content, 'site.yaml'), 'name: Fixture\n');
  if (windows) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleExe], { stdio: 'pipe' });
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-browser.rs', '-o', browserExe], { stdio: 'pipe' });
  if (!real) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-engine.rs', '-o', path.join(bin, windows ? 'docker.exe' : 'docker')], { stdio: 'pipe' });
  if (real) {
    await writeFile(path.join(content, 'guide/_sidebar.yaml'), 'title: Guide\nnav: [index, next]\n');
    await writeFile(path.join(content, 'guide/next.mdx'), '# Next page\n\n## Nebula\n\n[Home](/guide)\n');
    await mkdir(path.join(content, 'assets'));
    await writeFile(path.join(content, 'assets/logo.svg'), '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="green"/></svg>');
  }
  await edit('First preview');
  const server = await ready(launch());
  assert(server.url.endsWith('/products/manual/'));
  await until(() => exists(record), 'browser launch');
  assert.equal(await readFile(record, 'utf8'), server.url);
  const initial = await tree(output);
  assert(!Object.keys(initial).some(p => p.endsWith('.zellij-deployment.json')));
  assert(!(await readFile(path.join(output, 'index.html'), 'utf8')).includes('_zellij/preview.js'));
  const status = () => fetch(`${server.url}_zellij/preview.json`).then(r => r.json());
  let version = (await status()).version;
  browser = await chromium.launch();
  const page = await browser.newPage(); const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(server.url);
  if (real) await expect(page).toHaveURL(`${server.url}guide`);
  await expect(page.getByRole('heading', { name: 'First preview', exact: true })).toBeVisible();
  if (real) {
    assert.equal(await page.locator('article img').evaluate(img => img.complete && img.naturalWidth > 0), true);
    await page.locator('article').getByRole('link', { name: 'Next', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Next page', exact: true })).toBeVisible();
    await page.reload();
    // Static HTML is visible before client effects attach their search handler.
    const dialog = page.getByRole('dialog');
    await expect(page.locator('[data-zellij-search-trigger]')).toBeVisible();
    await expect.poll(async () => {
      if (await dialog.isVisible()) return true;
      await page.locator('[data-zellij-search-trigger]').click();
      return dialog.isVisible();
    }, { timeout: 15000 }).toBe(true);
    await page.getByRole('searchbox').fill('nebula');
    await page.getByRole('option', { name: /Nebula/ }).first().click();
    await expect(page).toHaveURL(`${server.url}guide/next#nebula`);
    await expect(page.getByRole('dialog')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: 'Nebula', exact: true })).toBeVisible();
    await page.goto(`${server.url}guide?keep=1#top`);
  }
  await edit('After rebuild');
  await until(async () => (await status()).version !== version, 'watched rebuild');
  await expect(page.getByRole('heading', { name: 'After rebuild', exact: true })).toBeVisible({ timeout: 30000 });
  if (real) { assert.equal(new URL(page.url()).search, '?keep=1'); assert.equal(new URL(page.url()).hash, '#top'); }
  const good = await tree(output); version = (await status()).version;
  await edit('Broken', '/products/manual', 'fail');
  await until(async () => (await status()).error, 'invalid content diagnostic');
  assert.deepEqual(await tree(output), good); assert.equal((await status()).version, version);
  await expect(page.locator('#_zel-preview-error')).toBeVisible();
  await edit('New prefix', '/changed');
  await until(async () => (await status()).error?.includes('basePath changed'), 'prefix stays pinned');
  assert.deepEqual(await tree(output), good);
  await edit('Recovered');
  await until(async () => (await status()).version !== version, 'recovery');
  await expect(page.getByRole('heading', { name: 'Recovered', exact: true })).toBeVisible({ timeout: 30000 });
  await expect(page.locator('#_zel-preview-error')).toHaveCount(0);
  if (!real) {
    // Invalid/missing config cannot change an active session's renderer/layout.
    const config = path.join(site, 'zel.yaml');
    const valid = await readFile(config, 'utf8');
    const previous = await tree(output); const revision = (await status()).version;
    for (const broken of ['version: [', valid.replace('hosted', 'portable'), null]) {
      if (broken === null) await rm(config); else await writeFile(config, broken);
      await until(async () => (await status()).error, 'config failure');
      assert.deepEqual(await tree(output), previous);
      await writeFile(config, valid);
      await until(async () => !(await status()).error, 'restored config');
    }
    // The source changes while a container is building: never publish that stage.
    const stable = await tree(output); version = (await status()).version;
    await edit('Stale input', '/products/manual', 'slow');
    await until(async () => (await readFile(log, 'utf8')).includes('Stale input'), 'slow build started');
    await edit('Newer input', '/products/manual', 'slow');
    await until(async () => (await status()).error?.includes('content changed during the build'), 'stale build rejected');
    assert.deepEqual(await tree(output), stable); assert.equal((await status()).version, version);
    await until(async () => (await status()).version !== version, 'newer input rebuilt');
    await expect(page.getByRole('heading', { name: 'Newer input', exact: true })).toBeVisible();
    assert((await status()).version !== revision);
  }
  assert.deepEqual(errors, [], `browser errors: ${errors}`);
  await stop(server); await browser.close(); browser = undefined;
  const saved = await tree(output);
  if (!real) {
    await rm(record);
    for (const [prefix, mode, message] of [['/bad/../prefix', 'ok', /base-path/], ['/ok', 'missing-index', /index.html/], ['/ok', 'missing-metadata', /error:/], ['/ok', 'empty', /not shared/]]) {
      await edit('Bad export', prefix, mode); fails(['view', '--browser', browserExe], message);
      assert.deepEqual(await tree(output), saved); assert.equal(await exists(record), false);
    }
    await edit('Valid');
    const socket = net.createServer(); await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve));
    try { fails(['view', '--port', String(socket.address().port)], /already in use/); } finally { socket.close(); }
    assert.deepEqual(await tree(output), saved);
    fails(['view', '--name', 'Portable only'], /only to portable/);
    fails(['view', '--json'], /supported only/);
    await edit('Cancelled initial', '/products/manual', 'slow');
    const cancelled = launch(['view', '--browser', browserExe]);
    await until(async () => (await readFile(log, 'utf8')).includes('Cancelled initial'), 'initial build started');
    await stop(cancelled); assert.deepEqual(await tree(output), saved);
    assert.equal(await exists(record), false); assert(!cancelled.stdout.includes('Serving at '));
    await edit('Once');
    const once = await ready(launch(['view', '--no-open']));
    assert(!(await (await fetch(once.url)).text()).includes('_zellij/preview.js'));
    await stop(once);
    // Cancel a started rebuild: wait for the owned worker, discard its stage.
    const watched = await ready(launch(['v', '--watch', '--no-open']));
    const before = await tree(output);
    await edit('Cancelled result', '/products/manual', 'slow');
    await until(async () => (await readFile(log, 'utf8')).includes('Cancelled result'), 'build started');
    await stop(watched); assert.deepEqual(await tree(output), before);
    assert(!(await readdir(site)).some(name => name.startsWith('.zel-build-')));
  }
  console.log(`Hosted preview (${real ? 'real engine' : process.platform}): prefix, browser, rebuild/recovery, publication safety and shutdown passed.`);
} catch (error) {
  for (const state of active) console.error(`Preview stdout:\n${state.stdout.slice(-12000)}\nPreview stderr:\n${state.stderr.slice(-12000)}`);
  throw error;
} finally {
  await browser?.close();
  for (const state of active) {
    try { await stop(state); } catch { state.child.kill(); }
  }
  await rm(root, { recursive: true, force: true, maxRetries: 5 });
}
