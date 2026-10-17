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
    let context;
    let page;
    let polling = true;
    let probeError;
    const availability = [];
    const probe = (async () => {
      while (polling) {
        const response = await fetch(server.url);
        await response.arrayBuffer(); // Release the connection between probes.
        availability.push(response.status);
        await delay(60);
      }
    })().catch(error => { probeError = error; });
    try {
      context = await browser.newContext({ serviceWorkers: 'block' });
      await context.tracing.start({ screenshots: true, snapshots: true, sources: true });
      await context.route(/^https?:/, route => new URL(route.request().url()).origin === new URL(server.url).origin ? route.continue() : route.abort());
      // Delay only the first fetch initiation. Holding an intercepted request
      // can also hold browser lifecycle events (including WebKit's DOM ready).
      // Responses still come from the real server, after publication. Reloaded
      // documents use ordinary fetch, so the rest of this scenario is unpatched.
      await context.addInitScript(() => {
        if (!['http:', 'https:'].includes(location.protocol)) return;
        const key = '_zel_test_first_poll';
        if (sessionStorage.getItem(key)) return;
        const original = window.fetch.bind(window);
        window.fetch = (input, options) => {
          const url = new URL(input instanceof Request ? input.url : input, location.href);
          if (url.pathname.endsWith('/_zellij/preview.json') && !sessionStorage.getItem(key)) {
            sessionStorage.setItem(key, 'held');
            window.__zelPollHeld = true;
            return new Promise(resolve => { window.__zelReleasePoll = resolve; }).then(() => original(input, options));
          }
          return original(input, options);
        };
      });
      page = await context.newPage();
      const errors = []; page.on('pageerror', error => errors.push(error.message));
      // This race deliberately holds initial loading work. Require the actual
      // document, rendered heading and attempted poll, not a lifecycle event.
      await page.goto(`${server.url}?sample=keep#anchor`, { waitUntil: 'commit' });
      await expect(page.getByRole('heading', { name: 'Before watching', exact: true })).toBeVisible();
      await until(() => page.evaluate(() => window.__zelPollHeld), 'first browser status poll is held');
      const original = await status();
      assert((await page.content()).includes('_zellij/preview.js?version='));
      assert(!(await readFile(path.join(output, 'index.html'), 'utf8')).includes('preview.js'));

      // Atomic editor saves, new documents, search updates and retained URL.
      await writeFile(extra, `# Extra ${engine}\n\n## Orion\n\n[Home](../README.md#anchor)\n`);
      await writeFile(path.join(source, '.edit.tmp'), home(`Updated ${engine}`));
      await rename(path.join(source, '.edit.tmp'), readme);
      await until(async () => (await status()).version !== original.version, 'publication before the initial browser poll');
      await page.evaluate(() => { window.__zelReleasePoll(); });
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
      assert.ifError(probeError);
      assert(availability.length > 10 && availability.every(code => code === 200), 'Publication must not expose a missing root index');
      await stop(server);
      assert.deepEqual(await snapshot(source), authored, 'Watching never rewrites authored source');
      await writeFile(readme, home('Edited after stopping'));
      await delay(800);
      assert.deepEqual(await snapshot(output), exported, 'Ctrl-C also stops the watcher');
      assert(!(await readdir(temp)).some(name => name.startsWith('.zel-build-')), 'No staging directories remain');
      assert.deepEqual(errors, []);
      await context.tracing.stop();
      await context.close();
      context = undefined;

      // Watch output is still a movable direct-file artifact with no polling.
      const moved = `${output}-moved-${engine}`;
      await rename(output, moved);
      const offline = await browser.newContext({ serviceWorkers: 'block', offline: engine === 'chromium' });
      context = offline;
      await offline.tracing.start({ screenshots: true, snapshots: true, sources: true });
      const requests = []; offline.on('request', request => requests.push(request.url()));
      await offline.route(/^https?:/, route => route.abort());
      const filePage = await offline.newPage();
      page = filePage;
      await filePage.goto(pathToFileURL(path.join(moved, 'index.html')).href);
      await expect(filePage.getByRole('heading', { name: `Recovered ${engine}`, exact: true })).toBeVisible();
      await expect.poll(() => filePage.locator('article img').evaluate(img => img.naturalWidth)).toBe(177);
      assert(requests.every(url => ['file:', 'data:'].includes(new URL(url).protocol)));
      assert.equal(await filePage.locator('script[src*="preview.js"]').count(), 0);
      await offline.tracing.stop();
      await offline.close(); context = undefined;
      await rename(moved, output);
      console.log(`${engine}: watch edits/assets/search, failed-build preservation, deletion/root/link recovery, live reload, real Ctrl-C and offline artifact passed.`);
    } catch (error) {
      console.error(`${engine} watch failure. CLI output:\n${server.stdout}\n${server.stderr}`);
      if (page && !page.isClosed()) {
        console.error(await Promise.race([
          page.evaluate(() => ({ url: location.href, ready: document.readyState, pollHeld: window.__zelPollHeld, headings: [...document.querySelectorAll('h1')].map(h => h.textContent) })),
          delay(2000, 'Browser state unavailable', { ref: false }),
        ]).catch(e => e.message));
      }
      if (context) {
        await mkdir('test-results', { recursive: true });
        await context.tracing.stop({ path: `test-results/preview-watch-${process.platform}-${engine}.zip` }).catch(e => console.error(e.message));
      }
      throw error;
    } finally {
      polling = false; await probe.catch(() => {});
      await browser.close();
      if (!server.exit) await stop(server);
    }
  }
}
