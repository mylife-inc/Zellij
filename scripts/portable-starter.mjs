// Test the files users actually receive from `new portable`, not a substitute fixture.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { rename } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect } from '@playwright/test';

export async function checkStarter({ binary, root, env, engine, browser }) {
  for (const minimal of [false, true]) {
    const source = path.join(root, `Starter 日本 ${engine} ${minimal}`);
    const output = path.join(source, 'site');
    const moved = path.join(root, `Moved starter café ${engine} ${minimal}`);
    const name = 'Acme: [Docs] <reference> & café 日本';
    execFileSync(binary, ['new', 'portable', source, '--name', name, '--theme', 'forest', ...(minimal ? ['--minimal'] : [])], { env });
    const report = JSON.parse(execFileSync(binary, ['check', source, '--strict', '--json'], { env, encoding: 'utf8' }));
    assert.equal(report.ok, true);
    assert.deepEqual(report.diagnostics, []);
    assert.equal(report.pages, minimal ? 1 : 2);
    execFileSync(binary, ['build', source], { env });
    await rename(output, moved);
    for (const javaScriptEnabled of [true, false]) {
      const context = await browser.newContext({ javaScriptEnabled, offline: engine === 'chromium', serviceWorkers: 'block' });
      const requests = [];
      const errors = [];
      context.on('request', request => requests.push(request.url()));
      await context.route(/^https?:/, route => route.abort());
      try {
        const page = await context.newPage();
        page.on('pageerror', error => errors.push(error.message));
        const home = pathToFileURL(path.join(moved, 'index.html')).href;
        const guide = pathToFileURL(path.join(moved, 'guide/getting-started.html')).href;
        await page.goto(home);
        await expect(page.getByRole('heading', { name, exact: true })).toBeVisible();
        await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
        const nav = page.getByRole('navigation', { name: 'Documentation', exact: true });
        await expect(nav.getByRole('link')).toHaveText(minimal ? ['Home'] : ['Home', 'Getting started']);
        if (!minimal) {
          await page.locator('article').getByRole('link', { name: 'getting started guide' }).click();
          assert.equal(page.url(), `${guide}#write-your-first-page`);
          await expect(page.getByRole('heading', { name: 'Getting started', exact: true })).toBeVisible();
          await expect(page.locator('article img')).toBeVisible();
          assert(await page.locator('article img').evaluate(img => img.complete && img.naturalWidth > 0));
          await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
          assert.equal(page.url(), `${home}#build-and-share`);
          // Open a nested file without loading home first in that page.
          const fresh = await context.newPage();
          await fresh.goto(guide);
          await expect(fresh.getByRole('heading', { name: 'Getting started', exact: true })).toBeVisible();
          await fresh.close();
          if (javaScriptEnabled) {
            await page.getByRole('button', { name: /Search/ }).click();
            await page.getByRole('searchbox').fill('company');
            await expect(page.getByRole('status')).toHaveText('1 matching page.');
            await page.locator('#_zel-results a').click();
            await expect(page.getByRole('heading', { name: 'Getting started', exact: true })).toBeVisible();
          }
        }
        assert.deepEqual(errors, []);
        assert(requests.every(url => /^(file|data):/.test(url)), 'Starter must work without HTTP or a CDN');
      } finally { await context.close(); }
    }
  }
  console.log(`${engine}: generated full/minimal starters pass strict checks, moved file navigation, assets/search and no-JS acceptance.`);
}
