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
  page.on('pageerror', error => errors.push(error.message));
  let headingCount = 0;
  for (const entry of baseline.pages) {
    assert.equal((await page.goto(origin + entry.route)).status(), 200, entry.route);
    const result = await page.evaluate(ids => ids.map(id => ({ id, count: [...document.querySelectorAll('[id]')].filter(el => el.id === id).length })), entry.heading_ids);
    for (const { id, count } of result) assert.equal(count, 1, `${entry.route}#${id}: expected one target`);
    headingCount += entry.heading_ids.length;
  }
  for (const route of baseline.new_routes) assert.equal((await page.goto(origin + route)).status(), 200, route);
  for (const route of new Set(aliases.map(alias => alias.route))) {
    const entries = aliases.filter(alias => alias.route === route);
    await page.goto(origin + route + '#' + encodeURIComponent(entries[0].legacy_id));
    await page.waitForFunction(id => {
      const top = document.getElementById(id).getBoundingClientRect().top;
      return top >= 0 && top < innerHeight;
    }, entries[0].legacy_id);
    const margins = await page.evaluate(ids => ids.map(id => ({ id, margin: parseFloat(getComputedStyle(document.getElementById(id)).scrollMarginTop) })), entries.map(entry => entry.legacy_id));
    for (const { id, margin } of margins) assert(margin > 0, `${route}#${id}: sticky-header offset missing`);
  }
  await page.goto(origin);
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
  assert.deepEqual(errors, [], 'Browser runtime errors');
  console.log(`Preservation passed: ${baseline.pages.length} original routes, ${baseline.new_routes.length} added routes, ${headingCount} original heading IDs, ${aliases.length} compatibility mappings, ${baseline.authored_assets.length} byte-identical images and current repository links.`);
} finally {
  if (browser) await browser.close();
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
}
