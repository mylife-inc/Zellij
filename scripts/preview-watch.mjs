// Real process/browser watch acceptance, using the shared preview lifecycle
// harness (including a private console and real Ctrl-C on Windows).
import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, rename, rm, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { chromium, webkit, expect } from '@playwright/test';

export async function checkWatch({ temp, source, output, start, stop, snapshot, until }) {
  const home = title => `# ${title}\n\n## Anchor\n\n![Logo](assets/logo.svg)\n\n[Next](guide/next.md#nebula)\n`;
  const svg = width => `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="40"><rect width="${width}" height="40" fill="blue"/></svg>`;
  const readme = path.join(source, 'README.md');
  const asset = path.join(source, 'assets/logo.svg');
  const extra = path.join(source, 'guide/extra.md');
  const bad = path.join(source, 'bad.md');
  for (const engine of process.env.ZELLIJ_PORTABLE_BROWSER?.split(',') || ['chromium', 'webkit']) {
    await writeFile(readme, home('Before watching'));
    await writeFile(asset, svg(100));
    await rm(extra, { force: true });
    const server = await start(['view', source, '--out', output, '--port', '0', '--no-open', '--watch']);
    assert(server.stdout.includes('Watching Markdown and assets'));
    const status = async () => {
      const response = await fetch(`${server.url}_zellij/preview.json`);
      assert.equal(response.status, 200); return response.json();
    };
    const browser = await ({ chromium, webkit })[engine].launch();
    let releaseFirstPoll = () => {};
    let polling = true;
    const availability = [];
    const probe = (async () => {
      while (polling) {
        availability.push((await fetch(server.url)).status);
        await delay(60);
      }
    })();
    try {
      const context = await browser.newContext({ serviceWorkers: 'block' });
      await context.route(/^https?:/, route => new URL(route.request().url()).origin === new URL(server.url).origin ? route.continue() : route.abort());
      // Force publication between HTML loading and its first status response.
      // A client that treats the first poll as its baseline would miss this edit.
      const firstPoll = new Promise(resolve => { releaseFirstPoll = resolve; });
      let holdPoll = true;
      await context.route(`${server.url}_zellij/preview.json`, async route => {
        if (holdPoll) await firstPoll;
        await route.continue();
      });
      const page = await context.newPage();
      const errors = []; page.on('pageerror', error => errors.push(error.message));
      await page.goto(`${server.url}?sample=keep#anchor`);
      await expect(page.getByRole('heading', { name: 'Before watching', exact: true })).toBeVisible();
      const original = await status();
      assert((await page.content()).includes('_zellij/preview.js?version='));
      assert(!(await readFile(path.join(output, 'index.html'), 'utf8')).includes('preview.js'));

      // Atomic editor saves, new documents, search updates and retained URL.
      await writeFile(extra, `# Extra ${engine}\n\n## Orion\n\n[Home](../README.md#anchor)\n`);
      await writeFile(path.join(source, '.edit.tmp'), home(`Updated ${engine}`));
      await rename(path.join(source, '.edit.tmp'), readme);
      await until(async () => (await status()).version !== original.version, 'publication before the initial browser poll');
      holdPoll = false; releaseFirstPoll();
      await expect(page.getByRole('heading', { name: `Updated ${engine}`, exact: true })).toBeVisible({ timeout: 20000 });
      assert.equal(new URL(page.url()).searchParams.get('sample'), 'keep');
      assert.equal(new URL(page.url()).hash, '#anchor');
      assert.notEqual((await status()).version, original.version);
      await page.keyboard.press('Control+k');
      await page.getByRole('searchbox').fill('Orion');
      await page.locator('#_zel-results a').click();
      await expect(page.getByRole('heading', { name: `Extra ${engine}`, exact: true })).toBeVisible();
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();

      await writeFile(asset, svg(144));
      await expect.poll(() => page.locator('article img').evaluate(img => img.naturalWidth), { timeout: 20000 }).toBe(144);
      await page.waitForLoadState('load');
      const good = await status();
      const oldExport = await snapshot(output);
      await page.evaluate(() => { window.previewSentinel = 'same document'; });
      await writeFile(bad, '# Invalid\n\n[Broken](absent.md)\n');
      await expect(page.getByRole('alert')).toContainText('last successful build', { timeout: 20000 });
      assert.equal((await status()).version, good.version);
      assert.deepEqual(await snapshot(output), oldExport);
      assert.equal(await page.evaluate(() => window.previewSentinel), 'same document');
      assert(server.stderr.includes('Rebuild failed; keeping the last successful site.'));

      // Changes while broken are picked up after correction, not lost.
      await writeFile(readme, home(`Recovered ${engine}`));
      await delay(900);
      assert.deepEqual(await snapshot(output), oldExport);
      await rm(bad);
      await expect(page.getByRole('heading', { name: `Recovered ${engine}`, exact: true })).toBeVisible({ timeout: 20000 });
      await expect(page.getByRole('alert')).toHaveCount(0);
      assert.equal((await status()).error, null);

      // Missing referenced assets and a temporarily absent source root recover.
      await rm(asset);
      await expect(page.getByRole('alert')).toBeVisible({ timeout: 20000 });
      await writeFile(asset, svg(177));
      await expect.poll(() => page.locator('article img').evaluate(img => img.naturalWidth), { timeout: 20000 }).toBe(177);
      await rename(source, `${source}-temporarily-moved`);
      await expect(page.getByRole('alert')).toBeVisible({ timeout: 20000 });
      await rename(`${source}-temporarily-moved`, source);
      await expect(page.getByRole('alert')).toHaveCount(0, { timeout: 20000 });

      const outside = path.join(temp, `outside-watch-${engine}`);
      await mkdir(outside); await writeFile(path.join(outside, 'external.md'), '# Must not be read');
      const linked = path.join(source, 'linked');
      await symlink(outside, linked, process.platform === 'win32' ? 'junction' : 'dir');
      await expect(page.getByRole('alert')).toBeVisible({ timeout: 20000 });
      await rm(linked);
      await expect(page.getByRole('alert')).toHaveCount(0, { timeout: 20000 });

      // A deleted current page shows a 404 which can itself recover on restore.
      await page.goto(`${server.url}guide/extra.html`);
      await rm(extra);
      await expect(page.getByRole('heading', { name: 'Page not found', exact: true })).toBeVisible({ timeout: 20000 });
      assert.equal((await fetch(page.url())).status, 404);
      await writeFile(extra, `# Restored ${engine}\n`);
      await expect(page.getByRole('heading', { name: `Restored ${engine}`, exact: true })).toBeVisible({ timeout: 20000 });

      const stable = await status();
      await writeFile(path.join(source, '.draft.md'), 'ignored');
      await mkdir(path.join(source, 'node_modules'));
      await writeFile(path.join(source, 'node_modules/bad.mdx'), 'ignored');
      await delay(1000);
      assert.deepEqual(await status(), stable, 'Ignored trees must not trigger a rebuild');
      await rm(path.join(source, '.draft.md'));
      await rm(path.join(source, 'node_modules'), { recursive: true });
      const authored = await snapshot(source);
      const exported = await snapshot(output);
      polling = false; await probe;
      assert(availability.length > 10 && availability.every(code => code === 200), 'Publication must not expose a missing root index');
      await stop(server);
      assert.deepEqual(await snapshot(source), authored, 'Watching never rewrites authored source');
      await writeFile(readme, home('Edited after stopping'));
      await delay(800);
      assert.deepEqual(await snapshot(output), exported, 'Ctrl-C also stops the watcher');
      assert(!(await readdir(temp)).some(name => name.startsWith('.zel-build-')), 'No staging directories remain');
      assert.deepEqual(errors, []);
      await context.close();

      // Watch output is still a movable direct-file artifact with no polling.
      const moved = `${output}-moved-${engine}`;
      await rename(output, moved);
      const offline = await browser.newContext({ serviceWorkers: 'block', offline: engine === 'chromium' });
      const requests = []; offline.on('request', request => requests.push(request.url()));
      await offline.route(/^https?:/, route => route.abort());
      const filePage = await offline.newPage();
      await filePage.goto(pathToFileURL(path.join(moved, 'index.html')).href);
      await expect(filePage.getByRole('heading', { name: `Recovered ${engine}`, exact: true })).toBeVisible();
      await expect.poll(() => filePage.locator('article img').evaluate(img => img.naturalWidth)).toBe(177);
      assert(requests.every(url => ['file:', 'data:'].includes(new URL(url).protocol)));
      assert.equal(await filePage.locator('script[src*="preview.js"]').count(), 0);
      await offline.close(); await rename(moved, output);
      console.log(`${engine}: watch edits/assets/search, failed-build preservation, deletion/root/link recovery, live reload, real Ctrl-C and offline artifact passed.`);
    } finally {
      releaseFirstPoll();
      polling = false; await probe.catch(() => {});
      await browser.close();
      if (!server.exit) await stop(server);
    }
  }
}
