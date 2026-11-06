// Called only by check-conversion.mjs --export in GitHub CI. The local,
// server-render-only check never imports this module or launches a browser.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium } from '@playwright/test';

export async function checkImportedTables(output) {
  const prefix = '/products/docs';
  const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.woff': 'font/woff', '.txt': 'text/plain' };
  const server = createServer(async (req, res) => {
    try {
      const pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      if (pathname !== prefix && !pathname.startsWith(prefix + '/')) { res.writeHead(404).end(); return; }
      const base = path.resolve(output, '.' + pathname.slice(prefix.length));
      if (base !== output && !base.startsWith(output + path.sep)) { res.writeHead(403).end(); return; }
      for (const candidate of [base, `${base}.html`, path.join(base, 'index.html')]) {
        try {
          const body = await readFile(candidate);
          res.writeHead(200, { 'content-type': mime[path.extname(candidate)] ?? 'application/octet-stream' }).end(body);
          return;
        } catch (error) { if (!['ENOENT', 'EISDIR', 'ENOTDIR'].includes(error.code)) throw error; }
      }
      res.writeHead(404).end();
    } catch { res.writeHead(500).end('Fixture error'); }
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const origin = `http://127.0.0.1:${server.address().port}`;
  let browser;
  try {
    // Navigation may prefetch the prefix root without a trailing slash.
    assert.equal((await fetch(`${origin}${prefix}`)).status, 200);
    browser = await chromium.launch();
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    page.on('response', response => { if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`); });
    await page.route('**/*', route => {
      if (new URL(route.request().url()).origin === origin) return route.continue();
      errors.push(`Unexpected external request: ${route.request().url()}`);
      return route.abort();
    });
    assert.equal((await page.goto(`${origin}${prefix}/guide/tables`)).status(), 200);
    // A neutral SSR label changes after useEffect. Waiting for load alone
    // would also pass a page whose scripts never hydrate.
    try {
      await page.getByRole('button', { name: /^Switch to (light|dark) theme$/ }).waitFor();
    } finally { assert.deepEqual(errors, []); }
    const article = page.locator('article');
    assert.equal(await article.locator('table').count(), 5);
    assert.equal(await article.getByRole('columnheader').count(), 6);
    assert.equal(await article.locator('table').nth(2).locator('th').count(), 0);
    assert.equal(await article.locator('table').nth(3).locator('th').count(), 0);
    assert.equal(await article.locator('caption').innerText(), 'Feature matrix');
    const logo = article.getByRole('img', { name: 'Table logo' });
    assert(await logo.evaluate(image => image.complete && image.naturalWidth > 0));
    await article.locator('table').first().getByRole('link', { name: 'Details', exact: true }).click();
    await page.waitForURL(`${origin}${prefix}/guide/details`);
    await page.getByRole('heading', { name: 'Installation', exact: true, level: 1 }).waitFor();
    assert.deepEqual(errors, [], 'Imported tables and navigation must not cause hydration or browser errors');
    console.log('Sphinx tables: browser hydration, semantic cells, local image and document navigation passed.');
  } finally {
    try { if (browser) await browser.close(); }
    finally { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
  }
}
