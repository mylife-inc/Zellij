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
    assert.equal(await outer.evaluate(list => getComputedStyle(list).listStyleType), 'disc');
    assert.equal(await outer.locator(':scope > li').first().evaluate(item => getComputedStyle(item).listStyleType), 'disc');
    assert.equal(await outer.locator(':scope > li').count(), 4);
    assert.equal((await outer.locator(':scope > li').nth(2).innerText()).trim(), '');
    assert.equal(await outer.locator(':scope > li').last().locator('table td').count(), 2);
    assert((await outer.locator(':scope > li').last().locator('pre').innerText()).includes('- literal block bullet\n9. literal block number'));
    const ordered = outer.locator(':scope > li').first().locator(':scope > ol');
    assert.equal(await ordered.getAttribute('start'), '9');
    assert.equal(await ordered.evaluate(list => getComputedStyle(list).listStyleType), 'decimal');
    assert.equal(await ordered.locator(':scope > li').first().evaluate(item => getComputedStyle(item).listStyleType), 'decimal');
    assert.equal(await ordered.locator(':scope > li').count(), 3);
    const inner = ordered.locator(':scope > li').nth(1).locator(':scope > ul');
    assert.equal(await inner.locator(':scope > li').count(), 2);
    assert((await inner.locator('pre').innerText()).includes(':header: literal option\n- literal bullet\n#. literal number'));
    assert((await inner.locator('blockquote').innerText()).includes('Keep original content.'));
    const nestedLogo = inner.getByRole('img', { name: 'Nested logo', exact: true });
    assert(await nestedLogo.evaluate(image => image.complete && image.naturalWidth > 0));
    for (const start of [7, 3, 4, 2]) assert.equal(await article.locator(`ol:not([type])[start="${start}"]`).count(), 1);
    assert.equal(await article.locator('table td ul ol').count(), 1);
    // HTML type attribute selectors match case-insensitively. Read exact DOM
    // values so a/A and i/I remain distinct in these assertions.
    const sequences = await article.locator('ol[type]').evaluateAll(lists => lists.map(list => ({
      type: list.getAttribute('type'),
      start: list.getAttribute('start'),
      count: [...list.children].filter(child => child.tagName === 'LI').length,
      style: getComputedStyle(list).listStyleType,
      itemStyle: getComputedStyle(list.querySelector(':scope > li')).listStyleType,
      itemDisplay: getComputedStyle(list.querySelector(':scope > li')).display,
      parentType: list.parentElement.tagName === 'LI' && list.parentElement.parentElement.tagName === 'OL'
        ? list.parentElement.parentElement.getAttribute('type') : null,
    })));
    assert.equal(sequences.length, 4);
    for (const [type, start, count, style] of [
      ['a', '8', 3, 'lower-alpha'], ['A', '8', 4, 'upper-alpha'],
      ['i', '4', 4, 'lower-roman'], ['I', '4', 3, 'upper-roman'],
    ]) {
      const matching = sequences.filter(sequence => sequence.type === type);
      assert.equal(matching.length, 1);
      assert.deepEqual(matching[0], { type, start, count, style, itemStyle: style, itemDisplay: 'list-item', parentType: type === 'i' ? 'A' : null });
    }
    assert.equal(await article.getByRole('link', { name: 'Roman reference', exact: true }).getAttribute('href'), `${prefix}/guide/details#install`);
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
    await page.goto(`${origin}${prefix}/guide/substitutions`);
    await page.getByRole('heading', { name: 'Reusable content', exact: true, level: 1 }).waitFor();
    await page.getByRole('button', { name: /^Switch to (light|dark) theme$/ }).waitFor();
    assert.equal(await article.locator('p').first().innerText(), 'Zellij is the Zellij documentation engine.');
    assert.equal(await article.locator('p').first().locator('strong').count(), 2);
    assert.equal(await article.locator('caption').innerText(), 'Zellij features');
    assert.equal(await article.locator('table td').first().innerText(), 'Zellij');
    const substitutionImages = article.getByRole('img', { name: 'Substitution logo', exact: true });
    assert.equal(await substitutionImages.count(), 2);
    assert(await substitutionImages.first().evaluate(image => image.complete && image.naturalWidth > 0));
    assert.equal(await substitutionImages.first().evaluate(image => getComputedStyle(image).display), 'inline');
    assert.equal(await substitutionImages.last().evaluate(image => getComputedStyle(image).display), 'block');
    const substitutionCode = await article.locator('pre').innerText();
    assert(substitutionCode.includes('|undefined-in-code|'));
    assert(substitutionCode.includes('.. |ignored| replace:: not a definition'));
    assert.equal(await article.locator('p').filter({ hasText: 'Inline code keeps' }).innerText(), 'Inline code keeps |Product| literal; an escaped |Product| stays literal too.');
    await article.getByRole('link', { name: 'Read the guide', exact: true }).first().click();
    await page.waitForURL(`${origin}${prefix}/guide/details#install`);
    await page.getByRole('heading', { name: 'Installation', exact: true, level: 1 }).waitFor();
    await page.goBack();
    await article.getByRole('heading', { name: 'Reusable content', exact: true }).waitFor();
    await substitutionImages.first().click();
    await page.waitForURL(`${origin}${prefix}/guide/details#install`);
    await page.goBack();
    await article.getByRole('heading', { name: 'Reusable content', exact: true }).waitFor();
    const [substitutionNotes] = await Promise.all([
      page.waitForResponse(response => response.url() === notesUrl && response.request().isNavigationRequest()),
      article.getByRole('link', { name: 'Substituted notes', exact: true }).click(),
    ]);
    assert.equal(substitutionNotes.status(), 200);
    assert.equal(await substitutionNotes.text(), 'Table download bytes.\n');
    await page.waitForURL(notesUrl);
    assert.deepEqual(errors, [], 'Imported tables, lists and navigation must not cause hydration or browser errors');
    console.log('Sphinx tables: browser hydration, semantic cells, nested lists, alphabetic/Roman numbering styles, text/image substitutions, local image, document navigation and native asset navigation passed.');
  } finally {
    try { if (browser) await browser.close(); }
    finally { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
  }
}
