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
    assert.equal(await article.locator('table').count(), 8);
    assert.equal(await article.getByRole('columnheader').count(), 17);
    assert.equal(await article.locator('table').nth(2).locator('th').count(), 0);
    assert.equal(await article.locator('table').nth(3).locator('th').count(), 0);
    assert.equal(await article.locator('caption').first().innerText(), 'Feature matrix');
    assert.equal(await article.getByRole('rowheader').count(), 7);
    const list = article.locator('table').nth(5);
    const csv = article.locator('table').nth(6);
    assert.equal(await list.getByRole('rowheader').first().innerText(), 'Alpha');
    assert.equal(await csv.locator('thead tr').count(), 3);
    assert.equal((await csv.locator('tbody tr').last().locator('td').last().innerText()).trim(), '');
    assert((await csv.innerText()).includes('Quoted "word", comma and bold.'));
    const listLogo = article.getByRole('img', { name: 'List logo', exact: true });
    assert(await listLogo.evaluate(image => image.complete && image.naturalWidth > 0));
    const logo = article.getByRole('img', { name: 'Table logo' });
    assert(await logo.evaluate(image => image.complete && image.naturalWidth > 0));
    await article.locator('table').first().getByRole('link', { name: 'Details', exact: true }).click();
    await page.waitForURL(`${origin}${prefix}/guide/details`);
    await page.getByRole('heading', { name: 'Installation', exact: true, level: 1 }).waitFor();
    await page.goBack();
    await article.locator('table').first().waitFor();
    await page.goto(`${origin}${prefix}/guide/lists`);
    await page.getByRole('heading', { name: 'Nested instructions', exact: true, level: 1 }).waitFor();
    await page.getByRole('button', { name: /^Switch to (light|dark) theme$/ }).waitFor();
    const outer = article.locator('ul').first();
    assert.equal(await outer.locator(':scope > li').count(), 4);
    assert.equal((await outer.locator(':scope > li').nth(2).innerText()).trim(), '');
    assert.equal(await outer.locator(':scope > li').last().locator('table td').count(), 2);
    assert((await outer.locator(':scope > li').last().locator('pre').innerText()).includes('- literal block bullet\n9. literal block number'));
    const ordered = outer.locator(':scope > li').first().locator(':scope > ol');
    assert.equal(await ordered.getAttribute('start'), '9');
    assert.equal(await ordered.locator(':scope > li').count(), 3);
    const inner = ordered.locator(':scope > li').nth(1).locator(':scope > ul');
    assert.equal(await inner.locator(':scope > li').count(), 2);
    assert((await inner.locator('pre').innerText()).includes(':header: literal option\n- literal bullet\n#. literal number'));
    assert((await inner.locator('blockquote').innerText()).includes('Keep original content.'));
    const nestedLogo = inner.getByRole('img', { name: 'Nested logo', exact: true });
    assert(await nestedLogo.evaluate(image => image.complete && image.naturalWidth > 0));
    for (const start of [7, 3, 4, 2]) assert.equal(await article.locator(`ol[start="${start}"]`).count(), 1);
    assert.equal(await article.locator('table td ul ol').count(), 1);
    await article.getByRole('link', { name: 'Nested details', exact: true }).click();
    await page.waitForURL(`${origin}${prefix}/guide/details`);
    await page.getByRole('heading', { name: 'Installation', exact: true, level: 1 }).waitFor();
    await page.goBack();
    await article.getByRole('heading', { name: 'Nested instructions', exact: true }).waitFor();
    const notesUrl = `${origin}${prefix}/_zellij/assets/imported/docs/notes.txt`;
    const [notes] = await Promise.all([
      page.waitForResponse(response => response.url() === notesUrl && response.request().isNavigationRequest()),
      article.getByRole('link', { name: 'Nested notes', exact: true }).click(),
    ]);
    assert.equal(notes.status(), 200);
    assert.equal(await notes.text(), 'Table download bytes.\n');
    await page.waitForURL(notesUrl);
    assert.deepEqual(errors, [], 'Imported tables, lists and navigation must not cause hydration or browser errors');
    console.log('Sphinx tables: browser hydration, semantic cells, nested lists, numbering, local image, document navigation and native asset navigation passed.');
  } finally {
    try { if (browser) await browser.close(); }
    finally { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
  }
}
