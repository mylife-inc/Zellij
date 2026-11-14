// Compare a built documentation site with the preserved public production inventory.
// Local only: no production writes, credentials, Docker or external browser requests.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';

const repo = fileURLToPath(new URL('../', import.meta.url));
const output = path.resolve(process.argv[2] ?? path.join(repo, 'examples/zellij-docs/out'));
const baseline = JSON.parse(await readFile(new URL('../specs/evidence/zellij-production-comparison-2026-11-02.json', import.meta.url)));
const addedRoutes = [...new Set([...baseline.new_routes, '/sections/compose', '/sections/media-accordion', '/cli/books'])];
const aliases = JSON.parse(await readFile(new URL('../specs/evidence/zellij-legacy-heading-aliases-2026-11-02.json', import.meta.url)));
const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.txt': 'text/plain' };
const server = createServer(async (req, res) => {
  try {
    const pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    const base = path.resolve(output, `.${pathname}`);
    if (base !== output && !base.startsWith(output + path.sep)) { res.writeHead(403).end(); return; }
    for (const candidate of [base, `${base}.html`, path.join(base, 'index.html')]) {
      try {
        const body = await readFile(candidate);
        res.writeHead(200, { 'content-type': mime[path.extname(candidate)] ?? 'application/octet-stream' }).end(body);
        return;
      } catch (error) { if (!['ENOENT', 'EISDIR', 'ENOTDIR'].includes(error.code)) throw error; }
    }
    res.writeHead(404).end('Not found');
  } catch { res.writeHead(500).end('Local fixture error'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
  for (const asset of baseline.authored_assets) {
    const bytes = await readFile(path.join(output, asset.path));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), asset.sha256, `Published asset changed: ${asset.path}`);
  }
  browser = await chromium.launch();
  const context = await browser.newContext();
  await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push({ kind: 'pageerror', url: page.url(), message: error.message }));
  // React reports recoverable hydration failures through console.error, not
  // necessarily pageerror. A page can recover and still fail acceptance.
  page.on('console', message => {
    if (message.type() === 'error') errors.push({ kind: 'console.error', url: page.url(), message: message.text() });
  });
  const assertNoErrors = () => assert.deepEqual(errors, [], `Browser runtime errors at ${page.url()}`);
  const visit = async route => {
    assert.equal((await page.goto(origin + route)).status(), 200, route);
    // This docs fixture enables the theme toggle on every route. Its neutral
    // SSR label changes only in useEffect, so load alone is not our readiness
    // signal. This also detects a page whose client scripts never hydrate.
    try {
      await page.getByRole('button', { name: /^Switch to (light|dark) theme$/ }).waitFor({ timeout: 15_000 });
    } catch (error) {
      throw new Error(`Documentation hydration readiness failed at ${page.url()}`, { cause: error });
    } finally {
      // Keep the causal browser error if it prevented mounting altogether.
      assertNoErrors();
    }
  };
  let headingCount = 0;
  for (const entry of baseline.pages) {
    await visit(entry.route);
    const result = await page.evaluate(ids => ids.map(id => ({ id, count: [...document.querySelectorAll('[id]')].filter(el => el.id === id).length })), entry.heading_ids);
    for (const { id, count } of result) assert.equal(count, 1, `${entry.route}#${id}: expected one target`);
    headingCount += entry.heading_ids.length;
  }
  for (const route of addedRoutes) await visit(route);
  await visit('/sections/compose');
  const composition = page.locator('[data-example="compose"]');
  await composition.getByRole('heading', { name: 'Clarity, composed.' }).waitFor();
  assert.equal(await composition.locator('.zellij-gradient-reveal-paint').first().evaluate(element => getComputedStyle(element).opacity), '1');
  await composition.getByRole('tab', { name: 'Source', exact: true }).click();
  assert.match(await composition.getByRole('tabpanel').filter({ visible: true }).innerText(), /type: compose/);
  await composition.getByRole('tab', { name: 'Preview', exact: true }).click();
  await composition.getByRole('heading', { name: 'Clarity, composed.' }).waitFor();
  assertNoErrors();
  await visit('/sections/media-accordion');
  const accordion = page.locator('[data-example="media-accordion"]');
  await accordion.getByRole('button', { name: 'Make room for ideas.', exact: true }).click();
  await accordion.getByRole('img', { name: 'An open notebook with text on the left and a green illustration on the right', exact: true }).waitFor();
  await accordion.getByRole('tab', { name: 'Source', exact: true }).click();
  assert.match(await accordion.getByRole('tabpanel').filter({ visible: true }).innerText(), /type: media-accordion/);
  await accordion.getByRole('tab', { name: 'Preview', exact: true }).click();
  await accordion.getByRole('button', { name: 'Find your focus.', exact: true }).waitFor();
  assertNoErrors();
  for (const route of new Set(aliases.map(alias => alias.route))) {
    const entries = aliases.filter(alias => alias.route === route);
    await visit(route + '#' + encodeURIComponent(entries[0].legacy_id));
    await page.waitForFunction(id => {
      const top = document.getElementById(id).getBoundingClientRect().top;
      return top >= 0 && top < innerHeight;
    }, entries[0].legacy_id);
    const margins = await page.evaluate(ids => ids.map(id => ({ id, margin: parseFloat(getComputedStyle(document.getElementById(id)).scrollMarginTop) })), entries.map(entry => entry.legacy_id));
    for (const { id, margin } of margins) assert(margin > 0, `${route}#${id}: sticky-header offset missing`);
  }
  await visit('/');
  assert(await page.locator('a[href="https://github.com/mylife-inc/Zellij"]').count() > 0);
  assert.equal(await page.locator('a[href="https://github.com/shebka/zellij"]').count(), 0);
  await page.locator('[data-zellij-search-trigger]').first().click();
  await page.getByRole('searchbox', { name: 'Search the guide' }).fill('runtime');
  await page.getByRole('option').first().waitFor();
  await Promise.all([
    page.waitForURL(url => url.pathname !== '/'),
    page.getByRole('option').first().click(),
  ]);
  const fragment = new URL(page.url()).hash.slice(1);
  if (fragment) assert(await page.evaluate(id => Boolean(document.getElementById(id)), decodeURIComponent(fragment)), 'Search result fragment is missing');
  assertNoErrors();
  console.log(`Preservation passed: ${baseline.pages.length} original routes, ${addedRoutes.length} added routes, ${headingCount} original heading IDs, ${aliases.length} compatibility mappings, ${baseline.authored_assets.length} byte-identical images, current repository links and no captured browser errors after hydration.`);
} finally {
  if (browser) await browser.close();
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
}
