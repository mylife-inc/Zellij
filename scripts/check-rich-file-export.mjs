/** Build showcase with ZELLIJ_OUTPUT=export first. All browser requests below use file://. */
import { mkdtemp, cp, readFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';
import { chromium, webkit, expect } from '@playwright/test';
const repo = path.resolve(import.meta.dirname, '..');
const temp = await mkdtemp(path.join(os.tmpdir(), 'zellij file 日本 '));
const output = path.join(temp, 'site');
await cp(path.join(repo, 'examples/showcase/out'), output, { recursive: true });
const exporter = pathToFileURL(path.join(repo, 'engine/packages/zellij/dist/offline/export.js')).href;
const code = `import {exportOffline} from ${JSON.stringify(exporter)}; await exportOffline(process.cwd(), ${JSON.stringify(output)});`;
const result = spawnSync(process.execPath, ['--input-type=module', '-e', code], {
  cwd: path.join(repo, 'examples/showcase'), env: { ...process.env, NODE_ENV: 'production' }, stdio: 'inherit',
});
assert.equal(result.status, 0, 'file export failed');
const manifest = JSON.parse(await readFile(path.join(output, '_zellij/file/manifest.json'), 'utf8'));
const url = route => pathToFileURL(path.join(output, route.slice(1), 'index.html')).href;
const entries = JSON.parse(await readFile(path.join(output, '_zellij/search-index.json'), 'utf8'));
const results = [];
try {
  for (const [name, browserType, options] of [['Chromium', chromium, process.platform === 'darwin' ? { channel: 'chrome' } : {}], ['WebKit', webkit, {}]]) {
    const browser = await browserType.launch({ headless: true, ...options });
    try {
      const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
      const errors = [], network = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('requestfailed', request => errors.push(`${request.url()}: ${request.failure().errorText}`));
      await page.route(/^https?:/, async route => { network.push(route.request().url()); await route.abort(); });
      for (const route of manifest.routes) {
        await page.goto(url(route));
        await page.evaluate(() => document.fonts.ready);
        await page.waitForFunction(() => [...document.querySelectorAll('.zellij-mermaid')].every(node => !['pending', 'failed'].includes(node.dataset.status)));
        await page.evaluate(() => { for (const image of document.images) image.loading = 'eager'; });
        await page.waitForFunction(() => [...document.images].every(image => image.complete));
        assert.deepEqual(await page.evaluate(() => [...document.images].filter(image => !image.naturalWidth).map(image => image.src)), [], route);
        await expect(page.locator('h1')).toHaveCount(1);
        // A successful hydration is not enough: reveal state updates used to
        // duplicate unkeyed headings without raising a browser exception.
        const html = await readFile(fileURLToPath(page.url()), 'utf8');
        const expectedHeaders = await page.evaluate(html => Array.from(
          new DOMParser().parseFromString(html, 'text/html').querySelectorAll('.zellij-section-header'),
          header => header.textContent), html);
        const headers = page.locator('.zellij-section-header');
        await expect.poll(() => headers.allTextContents()).toEqual(expectedHeaders);
        for (let index = 0; index < await headers.count(); index++) {
          if (!await headers.nth(index).isVisible()) continue;
          await headers.nth(index).evaluate(header => header.scrollIntoView({ block: 'center', behavior: 'instant' }));
          await expect(headers.nth(index)).toHaveAttribute('data-zellij-reveal', 'in');
        }
        await expect.poll(() => headers.allTextContents()).toEqual(expectedHeaders);
      }
      await page.goto(url('/'));
      await page.locator('[data-zellij-search-trigger]').first().click();
      await page.getByRole('dialog').getByRole('searchbox').fill(entries[0].title);
      await expect(page.locator('.zellij-search-result').first()).toBeVisible();
      await page.locator('.zellij-search-result').first().click();
      await expect(page).toHaveURL(/file:.*index\.html/);
      await expect(page).not.toHaveURL(url('/'));
      await page.waitForLoadState('networkidle');
      await expect(page.locator('h1')).toHaveCount(1);
      assert.equal(new URL(page.url()).protocol, 'file:');
      assert.deepEqual(errors, [], name);
      assert.deepEqual(network, [], `${name}: offline browsing attempted HTTP`);
      results.push({ browser: name, pages: manifest.pages, errors, network });
    } finally { await browser.close(); }
  }
  console.log(JSON.stringify(results, null, 2));
} finally { await rm(temp, { recursive: true, force: true }); }
