// Real wrapper/browser acceptance, or the actual Rust `zel dev` and container.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { once } from 'node:events';
import { mkdir, mkdtemp, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import { chromium, webkit, expect } from '@playwright/test';
import { observeThemeFonts } from './assert-theme-fonts.mjs';

const repo = fileURLToPath(new URL('../', import.meta.url));
const binary = process.argv[2] === '--container' ? path.resolve(process.argv[3]) : undefined;
const app = await mkdtemp(path.join(repo, '.zellij-dev-'));
const cli = path.join(repo, 'engine/packages/zellij/bin/zellij.mjs');
const content = path.join(app, 'authored docs');
const prefix = '/docs';
const browserRecord = path.join(app, 'browser.txt');
const browserExe = path.join(app, 'browser-fixture');
const env = { ...process.env, NEXT_TELEMETRY_DISABLED: '1', ZELLIJ_DEV_BROWSER_LOG: browserRecord };
delete env.ZELLIJ_OUTPUT;
delete env.ZELLIJ_BASE_PATH;
let server;
let exited;
let serverPort;
let log = '';
let browser;
let containerName;
const put = async (name, text) => { await mkdir(path.dirname(path.join(content, name)), { recursive: true }); await writeFile(path.join(content, name), text); };
async function until(check, label, timeout = 30000, allowExited = false) {
  const start = Date.now();
  let last;
  while (Date.now() - start < timeout) {
    if (!allowExited && server?.exitCode !== null && server?.exitCode !== undefined) throw new Error(`Dev exited: ${log}`);
    try { if (await check()) return; } catch (error) { last = error; }
    await delay(250);
  }
  throw new Error(`${label}\n${last ?? ''}\n${log.slice(-6000)}`);
}
try {
  if (binary) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/dev-browser.rs', '-o', browserExe], { stdio: 'pipe' });
  execFileSync(process.execPath, [cli, 'init', app, '--name', 'watch-fixture'], { env, stdio: 'inherit' });
  if (!binary) execFileSync(process.execPath, [cli, 'features', 'all', app], { env, stdio: 'inherit' });
  await rm(path.join(app, 'content'), { recursive: true });
  // A custom content root must reach preparation, rendering and Next config.
  const files = {
    'next.config.js': "import { withZellij } from '@shebka/zellij/next-config';\nexport default withZellij({ typedRoutes: false }, { contentDir: './authored docs' });\n",
    'src/app/layout.tsx': "import '@shebka/zellij/styles.css';\nimport { createZellijRootLayout } from '@shebka/zellij';\nexport default createZellijRootLayout({ contentDir: './authored docs' });\n",
    'src/app/[[...slug]]/page.tsx': "import { createZellijPages } from '@shebka/zellij';\nconst zellij = createZellijPages({ contentDir: './authored docs' });\nexport const generateStaticParams = zellij.generateStaticParams;\nexport const generateMetadata = zellij.generateMetadata;\nexport default zellij.Page;\n",
  };
  for (const [name, text] of Object.entries(files)) await writeFile(path.join(app, name), text);
  await put('site.yaml', `name: Watch fixture\nbasePath: ${prefix}\nhome: 01-guide\nnav: { search: true }\nanalytics: { provider: plausible, domain: docs.example.com }\n`);
  await put('01-guide/_sidebar.yaml', 'route: /manual\nnav: [index]\n');
  const markdown = (token) => `# Watching\n\n## ${token}\n\n${token} passage.\n\n![Watched image](/_zellij/assets/diagram.svg)\n`
    + '$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n';
  const svg = (width) => `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="40"><rect width="${width}" height="40" fill="red"/></svg>`;
  await put('01-guide/index.mdx', markdown('Original'));
  await put('assets/diagram.svg', svg(80));
  const reservation = createServer();
  reservation.listen(0, '127.0.0.1');
  await once(reservation, 'listening');
  const port = reservation.address().port;
  serverPort = port;
  await new Promise((resolve, reject) => reservation.close((error) => error ? reject(error) : resolve()));
  const origin = `http://127.0.0.1:${port}${prefix}`;
  server = binary
    ? spawn(binary, ['dev', content, '--port', String(port), '--host', '127.0.0.1', '--browser', browserExe, '--features', 'all'], { cwd: app, env, stdio: ['ignore', 'pipe', 'pipe'] })
    : spawn(process.execPath, [cli, 'dev', '--content', './authored docs', '--', '--hostname', '0.0.0.0', '--port', String(port), ...(process.env.ZELLIJ_TEST_WEBPACK ? ['--webpack'] : [])], { cwd: app, env, stdio: ['ignore', 'pipe', 'pipe'] });
  exited = once(server, 'exit');
  server.stdout.on('data', (data) => { log += data; });
  server.stderr.on('data', (data) => { log += data; });
  const fetchFile = (name) => fetch(`${origin}/_zellij/${name}`, { cache: 'no-store', signal: AbortSignal.timeout(10000) });
  const index = async () => (await fetchFile('search-index.json')).json();
  await until(async () => (await index()).some((entry) => entry.route === '/manual#original'), 'initial dev preparation', binary ? 240000 : 60000);
  if (binary) {
    const marker = await (await fetchFile('dev-revision.json')).json();
    containerName = marker.session;
    assert.match(containerName, /^zel-dev-/);
    assert.equal(marker.basePath, prefix);
    await until(async () => (await readFile(browserRecord, 'utf8')) === `${origin}/\n`, 'browser opens once at the configured prefix');
    assert(log.includes(`Serving at ${origin}/`));
    const [container] = JSON.parse(execFileSync('docker', ['inspect', containerName], { encoding: 'utf8' }));
    assert(container.HostConfig.PortBindings['3000/tcp'].every(binding => binding.HostIp === '127.0.0.1'));
  }
  await until(async () => (await fetch(`${origin}/manual`, { signal: AbortSignal.timeout(10000) })).ok, 'first page compiles', 120000);
  const engines = binary ? { chromium } : process.env.CI ? { chromium, webkit } : { chromium };
  for (const [name, engine] of Object.entries(engines)) {
    const token = `Updated${name}`;
    let page;
    let navigations = 0;
    const browserErrors = [];
    if (engine) {
      browser = await engine.launch();
      page = await browser.newPage();
      page.on('pageerror', error => { browserErrors.push(error.message); log += `\n[browser] ${error.stack ?? error}\n`; });
      page.on('console', message => { if (message.type() === 'error') browserErrors.push(message.text()); });
      page.on('framenavigated', frame => { if (frame === page.mainFrame()) navigations++; });
      page.on('requestfailed', request => { log += `\n[browser] ${request.url()}: ${request.failure()?.errorText}\n`; });
      const checkFonts = observeThemeFonts(page, `http://127.0.0.1:${port}`, prefix, { development: true });
      await page.goto(`${origin}/manual`);
      await expect(page.getByRole('heading', { name: 'Watching', exact: true })).toBeVisible();
      await expect(page.locator('mjx-container svg')).toBeVisible({ timeout: 30000 });
      await expect(page.locator('.zellij-mermaid svg')).toBeVisible({ timeout: 30000 });
      // The wildcard bind must accept loopback clients without opening dev
      // assets to arbitrary sites (including hostnames that resemble an IP).
      const clientScript = new URL(await page.locator('script[src*="/_next/"]').first().getAttribute('src'), origin);
      for (const host of ['127.0.0.1', '[::1]', 'untrusted.example.com', '127.0.0.1.example.com']) {
        const response = await fetch(clientScript, { headers: { Origin: `http://${host}:${port}` }, signal: AbortSignal.timeout(10000) });
        assert.equal(response.status, host.endsWith('.com') ? 403 : 200, `Development origin ${host}`);
        await response.body?.cancel();
      }
      await checkFonts();
      await expect(async () => {
        await page.keyboard.press('ControlOrMeta+k');
        await expect(page.getByRole('dialog')).toBeVisible({ timeout: 1000 });
      }).toPass({ timeout: 15000 });
      await expect(page.locator('.zellij-search-empty')).toHaveText('Start typing to search the guide.');
      await page.keyboard.press('Escape');
      assert.equal(await page.evaluate(() => typeof window.plausible), 'undefined', 'Development must not initialize analytics');
    }
    await put('01-guide/index.mdx', markdown(token));
    await put('01-guide/extra.mdx', `# Added${name}\n\nAdded document.\n`);
    await put('01-guide/_sidebar.yaml', 'route: /manual\nnav: [index, extra]\n');
    await put('assets/diagram.svg', svg(120));
    await until(async () => (await index()).some((entry) => entry.route === `/manual#${token.toLowerCase()}`), 'updated search route');
    await until(async () => (await (await fetchFile('assets/diagram.svg')).text()).includes('width="120"'), 'updated asset bytes');
    if (page) {
      // No manual reload: successful preparation must update the open tab.
      const heading = page.locator(`h2#${token.toLowerCase()}`);
      await expect(heading).toBeVisible({ timeout: 30000 });
      await expect(heading).toHaveAccessibleName(token);
      await expect(page).toHaveURL(`${origin}/manual`);
      await expect(async () => {
        await page.keyboard.press('ControlOrMeta+k');
        await expect(page.getByRole('dialog')).toBeVisible({ timeout: 1000 });
      }).toPass({ timeout: 15000 });
      await page.locator('.zellij-search-input').fill(token);
      await expect(page.locator('.zellij-search-result').first()).toContainText(token);
      await page.keyboard.press('Escape');
      await expect(page.locator('mjx-container svg')).toBeVisible({ timeout: 30000 });
      await expect(page.locator('.zellij-mermaid svg')).toBeVisible({ timeout: 30000 });
      await expect.poll(() => page.getByAltText('Watched image').evaluate((img) => img.naturalWidth)).toBe(120);
      // Images alone must trigger a reload with a new fingerprint too.
      await put('assets/diagram.svg', svg(160));
      await expect.poll(() => page.getByAltText('Watched image').evaluate((img) => img.naturalWidth), { timeout: 30000 }).toBe(160);
      await page.getByRole('link', { name: `Added${name}`, exact: true }).first().click();
      await expect(page.getByRole('heading', { name: `Added${name}`, exact: true })).toBeVisible();
      await page.goBack();
      await expect(heading).toBeVisible();
    } else {
      await until(async () => (await (await fetch(`${origin}/manual`)).text()).includes(token), 'rendered content after edit');
      assert((await fetch(`${origin}/manual/extra`)).ok);
    }
    const good = await index();
    const goodRevision = await (await fetchFile('dev-revision.json')).json();
    const goodNavigations = navigations;
    const priorErrors = (log.match(/Preparation failed/g) ?? []).length;
    await put('01-guide/index.mdx', '# Invalid\n\n<Callout>\n');
    await put('assets/diagram.svg', svg(140));
    await until(() => (log.match(/Preparation failed/g) ?? []).length > priorErrors, 'invalid edit reported');
    assert.deepEqual(await index(), good, 'invalid edit preserves last prepared index');
    assert.deepEqual(await (await fetchFile('dev-revision.json')).json(), goodRevision, 'invalid edit must not announce a reload');
    if (page) {
      await delay(1500); // span the browser poll after the watcher reported failure
      assert.equal(navigations, goodNavigations, 'invalid edit must retain the open document');
      await expect(page.locator(`h2#${token.toLowerCase()}`)).toBeVisible();
      await expect.poll(() => page.getByAltText('Watched image').evaluate((img) => img.naturalWidth)).toBe(160);
    }
    await put('01-guide/index.mdx', markdown(`Recovered${name}`));
    await rm(path.join(content, '01-guide/extra.mdx'));
    await put('01-guide/_sidebar.yaml', 'route: /manual\nnav: [index]\n');
    await until(async () => { const entries = await index(); return entries.some((entry) => entry.text.includes(`Recovered${name}`)) && !entries.some((entry) => entry.route === '/manual/extra'); }, 'recovery and deleted document');
    if (page) {
      await expect(page.locator(`h2#recovered${name}`)).toBeVisible({ timeout: 30000 });
      await expect(page.getByRole('link', { name: `Added${name}`, exact: true })).toHaveCount(0);
      await expect.poll(() => page.getByAltText('Watched image').evaluate((img) => img.naturalWidth)).toBe(140);
      assert.deepEqual(browserErrors, [], `${name}: browser hydration and reload must not report errors`);
    }
    await browser?.close(); browser = undefined;
    await rename(path.join(content, 'assets'), path.join(app, 'saved-assets'));
    await until(async () => (await fetchFile('assets/diagram.svg')).status === 404, 'entire assets directory removed');
    await rename(path.join(app, 'saved-assets'), path.join(content, 'assets'));
    await until(async () => (await fetchFile('assets/diagram.svg')).ok, 'assets directory restored');
    console.log(`${name}: automatic dev reload, edits/additions/deletions, search, image-only updates and invalid-edit recovery passed.`);
  }
  if (binary) assert.equal(await readFile(browserRecord, 'utf8'), `${origin}/\n`, 'content reloads must not reopen the browser');
} catch (error) {
  console.error(log.slice(-12000));
  throw error;
} finally {
  await browser?.close();
  if (server && server.exitCode === null && server.signalCode === null) {
    // Signal only zel: it must clean up its own named container and runtime client.
    server.kill(binary ? 'SIGINT' : 'SIGTERM');
    const timer = setTimeout(() => server.kill('SIGKILL'), 10000);
    const [code, signal] = await exited;
    clearTimeout(timer);
    if (binary) { assert.equal(code, 0, `dev shutdown ${signal}: ${log}`); assert(log.includes('Stopped.')); }
  }
  if (binary && containerName) {
    await until(() => !execFileSync('docker', ['ps', '-aq', '--filter', `name=^/${containerName}$`], { encoding: 'utf8' }).trim(), 'dev removes its container', 30000, true);
  }
  if (process.env.ZELLIJ_TEST_DEV_LOG) await writeFile(process.env.ZELLIJ_TEST_DEV_LOG, log);
  await rm(app, { recursive: true, force: true });
  if (serverPort) {
    let released = false;
    for (let attempt = 0; attempt < 30 && !released; attempt++) {
      released = await new Promise((resolve) => {
        const probe = createServer();
        probe.once('error', () => resolve(false));
        probe.listen(serverPort, '127.0.0.1', () => probe.close(() => resolve(true)));
      });
      if (!released) await delay(100);
    }
    assert(released, 'Stopping dev must release its HTTP port; no orphaned server');
  }
}
