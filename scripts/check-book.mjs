// Hosted CI only: a complete book must remain usable after moving its export.
import assert from 'node:assert/strict';
import { checkEquations } from './check-equations.mjs';
import { checkStatements } from './check-statements.mjs';
import { checkTerms } from './check-terms.mjs';
import { checkBibliography } from './check-bibliography.mjs';
import { execFileSync } from 'node:child_process';
import { cp, mkdtemp, mkdir, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium, webkit, expect } from '@playwright/test';
const binary = path.resolve(process.argv[2] || `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const temp = await mkdtemp(path.join(tmpdir(), 'zellij-book-'));
const source = path.join(temp, 'manuscript');
const moved = path.join(temp, 'Moved book 日本');
const captures = path.resolve('test-results/book');
try {
  await cp('scripts/fixtures/book', source, { recursive: true });
  const env = { ...process.env, PATH: path.join(temp, 'no-tools') };
  execFileSync(binary, ['build', source], { env, stdio: 'pipe' });
  await rename(path.join(source, 'site'), moved);
  const record = JSON.parse(await readFile(path.join(moved, '_zellij/book.json'), 'utf8'));
  assert.equal(record.entries[3].title, 'Better questions');
  const settings = await readFile(path.join(source, 'zel.yaml'), 'utf8');
  await writeFile(path.join(source, 'zel.yaml'), settings.replace('direction: ltr', 'direction: rtl'));
  execFileSync(binary, ['build', source], { env, stdio: 'pipe' });
  const technical = path.join(temp, 'technical');
  await cp('scripts/fixtures/equations', technical, { recursive: true });
  execFileSync(binary, ['build', technical], { env, stdio: 'pipe' });
  const relocated = path.join(temp, 'Relocated equations 日本');
  await rename(path.join(technical, 'site'), relocated);
  const statements = path.join(temp, 'statements');
  await cp('scripts/fixtures/statements', statements, { recursive: true });
  execFileSync(binary, ['build', statements], { env, stdio: 'pipe' });
  const movedStatements = path.join(temp, 'Relocated statements 日本');
  await rename(path.join(statements, 'site'), movedStatements);
  const terms = path.join(temp, 'terms');
  await cp('scripts/fixtures/terms', terms, { recursive: true });
  execFileSync(binary, ['build', terms], { env, stdio: 'pipe' });
  const movedTerms = path.join(temp, 'Relocated terms 日本');
  await rename(path.join(terms, 'site'), movedTerms);
  const bibliography = path.join(temp, 'bibliography');
  await cp('scripts/fixtures/bibliography', bibliography, { recursive: true });
  execFileSync(binary, ['build', bibliography], { env, stdio: 'pipe' });
  const movedBibliography = path.join(temp, 'Relocated bibliography 日本');
  await rename(path.join(bibliography, 'site'), movedBibliography);
  const bibRecord = JSON.parse(await readFile(path.join(movedBibliography, '_zellij/book.json'), 'utf8'));
  assert.deepEqual(bibRecord.bibliography.references.map(r => r.id), ['vale2024', 'river2021', 'shebka2026', 'uncited']);
  await mkdir(captures, { recursive: true });
  for (const [engine, launcher] of Object.entries({ chromium, webkit })) {
    const browser = await launcher.launch();
    try {
      for (const width of [1280, 390]) {
        const context = await browser.newContext({ viewport: { width, height: 900 }, offline: engine !== 'webkit', serviceWorkers: 'block' });
        const errors = [], requests = [];
        context.on('request', request => { if (/^https?:/.test(request.url())) requests.push(request.url()); });
        await context.route(/^https?:/, route => route.abort());
        const page = await context.newPage();
        page.on('pageerror', error => errors.push(error.message));
        await page.goto(pathToFileURL(path.join(moved, 'index.html')).href);
        await expect(page.getByRole('heading', { name: 'An atlas of quiet ideas', exact: true })).toBeVisible();
        await expect(page.locator('.book-authors')).toHaveText('Mira Vale');
        assert(await page.locator('.book-cover').evaluate(img => img.complete && img.naturalWidth > 0));
        await page.evaluate(() => document.fonts.ready);
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), 'No horizontal overflow');
        await page.screenshot({ path: path.join(captures, `${engine}-${width}-title.png`), fullPage: true });
        await page.locator('a[rel=next]').click();
        await expect(page.getByRole('heading', { name: 'Before we begin', exact: true })).toBeVisible();
        await page.locator('a[rel=next]').click();
        await expect(page.locator('.book-number')).toHaveText('Chapter 1');
        await page.locator('a[rel=next]').click();
        await expect(page.getByRole('heading', { name: 'A question worth keeping', exact: true })).toBeVisible();
        await expect(page.locator('.book-number')).toHaveText('Chapter 2');
        await page.screenshot({ path: path.join(captures, `${engine}-${width}-chapter.png`), fullPage: true });
        await page.getByRole('button', { name: /Search/ }).click();
        await page.getByRole('searchbox').fill('observatory');
        await expect(page.getByRole('status')).toHaveText('1 matching page.');
        await page.getByRole('searchbox').press('ArrowDown');
        await expect(page.locator('#_zel-results a')).toBeFocused();
        await page.keyboard.press('Enter');
        await expect(page.locator('article')).toHaveAttribute('data-book-id', 'questions');
        await page.locator('a[rel=next]').click();
        await page.locator('a[rel=next]').click();
        await expect(page.locator('.book-number')).toHaveText('Appendix A');
        await page.goto(pathToFileURL(path.join(source, 'site/index.html')).href);
        await expect(page.locator('html')).toHaveAttribute('dir', 'rtl');
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
        if (width === 1280) {
          const aside = await page.locator('.sidebar').boundingBox();
          const main = await page.locator('main').boundingBox();
          assert(aside.x > main.x, 'RTL sidebar follows the inline start edge');
        }
        await page.screenshot({ path: path.join(captures, `${engine}-${width}-rtl.png`), fullPage: true });
        assert.deepEqual(errors, []); assert.deepEqual(requests, []);
        await context.close();
      }
      for (const javaScriptEnabled of [true, false]) {
        const equations = await browser.newContext({ javaScriptEnabled, offline: engine !== 'webkit' });
        await equations.route(/^https?:/, route => route.abort());
        const equationPage = await equations.newPage();
        await checkEquations(equationPage, pathToFileURL(relocated + path.sep).href, captures, `${engine}-equations-${javaScriptEnabled}`);
        await checkStatements(equationPage, pathToFileURL(movedStatements + path.sep).href, captures, `${engine}-statements-${javaScriptEnabled}`);
        await checkBibliography(equationPage, pathToFileURL(movedBibliography + path.sep).href, captures, `${engine}-bibliography-${javaScriptEnabled}`);
        await checkTerms(equationPage, pathToFileURL(movedTerms + path.sep).href, captures, `${engine}-terms-${javaScriptEnabled}`);
        await equations.close();
      }
      const context = await browser.newContext({ javaScriptEnabled: false });
      await context.route(/^https?:/, route => route.abort());
      const page = await context.newPage();
      await page.goto(pathToFileURL(path.join(moved, 'index.html')).href);
      await page.locator('article').getByRole('link', { name: '1. The art of noticing', exact: true }).click();
      await expect(page.locator('.book-number')).toHaveText('Chapter 1');
      await page.locator('a[rel=next]').click();
      await expect(page.locator('.book-number')).toHaveText('Chapter 2');
      await context.close();
    } finally { await browser.close(); }
  }
  console.log('Book: relocated offline navigation, search, numbering, covers, mobile, RTL and no-script reading passed.');
} finally { await rm(temp, { recursive: true, force: true }); }
