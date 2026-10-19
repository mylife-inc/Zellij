// Install and build a native-generated wrapper outside the workspace. Exercise
// real file: dependency installation, bundled CLI entry points and Next/browser
// rendering. No container, registry publication or provider account is used.
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { cp, lstat, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';

const repo = fileURLToPath(new URL('../', import.meta.url));
const eject = process.argv.includes('--eject');
const zel = path.resolve(process.argv.slice(2).find(arg => !arg.startsWith('--')) ?? 'cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-app-scaffold-'));
const engine = path.join(root, 'local engine #1%');
const app = path.join(root, 'editable app');
const starter = path.join(root, 'starter');
const content = path.join(root, 'docs/manual/content');
const env = { ...process.env, NEXT_TELEMETRY_DISABLED: '1', ZELLIJ_OUTPUT: '' };
delete env.ZELLIJ_BASE_PATH;
let server;
let serverDone;
const run = (file, args, cwd = app, extra = {}) => execFileSync(file, args, {
  cwd, env: { ...env, ...extra }, stdio: 'inherit', timeout: 600000,
});
// setup-node's Windows distribution ships npm alongside node.exe. Invoke its
// JS entry directly so a .cmd wrapper never needs shell quoting of arguments.
const npm = args => process.platform === 'win32'
  ? run(process.execPath, [path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js'), ...args])
  : run('npm', args);
async function put(file, text) {
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, text);
}
async function ready(origin) {
  for (let n = 0; n < 120; n++) {
    try { if ((await fetch(`${origin}/products/docs/`, { signal: AbortSignal.timeout(1000) })).ok) return; } catch { /* startup */ }
    if (server.exitCode !== null) throw new Error(`Next exited with ${server.exitCode}`);
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  throw new Error('Generated wrapper did not become ready');
}
try {
  if (!eject) {
    const packageRoot = path.join(repo, 'engine/packages/zellij');
    const manifest = JSON.parse(await readFile(path.join(packageRoot, 'package.json'), 'utf8'));
    for (const file of ['package.json', ...manifest.files]) {
      await mkdir(path.dirname(path.join(engine, file)), { recursive: true });
      await cp(path.join(packageRoot, file), path.join(engine, file), { recursive: true });
    }
  }
  await put(path.join(root, 'docs/manual/zel.yaml'), 'version: 1\nrenderer: hosted\n');
  await put(path.join(content, 'site.yaml'), 'name: External documentation\nhome: manual\nbasePath: /products/docs\n');
  await put(path.join(content, 'manual/_sidebar.yaml'), 'title: Manual\nnav: [index, next]\n');
  const original = '# External documentation\n\n[Next page](/manual/next)\n\n![Logo](/_zellij/assets/logo.svg)\n';
  await put(path.join(content, 'manual/index.mdx'), original);
  await put(path.join(content, 'manual/next.mdx'), '# More documentation\n\n## Searchable heading\n\nSaffron passage.\n');
  await put(path.join(content, 'assets/logo.svg'), '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="9"/></svg>');
  if (eject) {
    run(zel, ['eject', 'docs/manual', '--out', app], root);
    run(zel, ['eject', '--with-docs', '--out', starter], root);
    // Exercise selection after extraction, before installing the real engine.
    run(zel, ['features', 'all', '--site-root', app], root);
    run(zel, ['features', 'all', '--site-root', starter], root);
    assert.equal(JSON.parse(await readFile(path.join(app, 'package.json'), 'utf8')).dependencies['@shebka/zellij'], 'file:vendor/zellij');
  } else {
    run(zel, ['new', 'app', app, '--site-root', 'docs/manual', '--engine', engine, '--features', 'all'], root);
    run(zel, ['na', starter, '--with-docs', '--engine', engine, '--features', 'all'], root);
  }
  npm(['install', '--no-audit', '--no-fund']);
  assert(!(await lstat(path.join(app, 'node_modules/@shebka/zellij'))).isSymbolicLink(), 'install-links must copy the engine');
  npm(['run', 'check', '--', '--deep', '--strict']);
  // Same installed bundle checks the with-docs selector without a second install.
  run(process.execPath, [path.join(app, 'node_modules/@shebka/zellij/bin/zellij.mjs'), 'check', '--deep', '--strict'], starter);
  npm(['run', 'build']);
  npm(['run', 'typecheck']);
  const listener = net.createServer();
  await new Promise(resolve => listener.listen(0, '127.0.0.1', resolve));
  const port = listener.address().port;
  await new Promise(resolve => listener.close(resolve));
  server = spawn(process.execPath, [path.join(app, 'node_modules/next/dist/bin/next'), 'start', '--port', String(port), '--hostname', '127.0.0.1'], { cwd: app, env, stdio: 'inherit' });
  serverDone = new Promise((resolve, reject) => { server.once('exit', resolve); server.once('error', reject); });
  const origin = `http://127.0.0.1:${port}`;
  await ready(origin);
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('response', response => { if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`); });
    await page.goto(`${origin}/products/docs/`);
    await page.waitForURL(`${origin}/products/docs/manual`);
    await page.getByRole('heading', { name: 'External documentation', exact: true }).waitFor();
    const logo = page.locator('article img[alt="Logo"]');
    assert(await logo.evaluate(image => image.complete && image.naturalWidth > 0));
    await page.getByRole('link', { name: 'Next page', exact: true }).click();
    await page.waitForURL(`${origin}/products/docs/manual/next`);
    await page.reload();
    await page.getByRole('heading', { name: 'More documentation', exact: true }).waitFor();
    assert.deepEqual(errors, [], 'Generated wrapper browser failures');
  } finally { await browser.close(); }
  const search = await (await fetch(`${origin}/products/docs/_zellij/search-index.json`)).json();
  assert(search.some(entry => entry.route === '/manual/next#searchable-heading'));
  assert.equal((await fetch(`${origin}/manual`)).status, 404);
  assert.equal((await fetch(`${origin}/products/docs/missing`)).status, 404);
  assert.equal(await readFile(path.join(content, 'manual/index.mdx'), 'utf8'), original);
  console.log(`${eject ? 'Ejected' : 'Native'} app: installed local engine, starter/external checks, production build, navigation, reload, assets and search passed.`);
} finally {
  if (server && server.exitCode === null && server.signalCode === null) {
    server.kill('SIGTERM');
    await serverDone;
  }
  await rm(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
