// Configured roots must watch authored inputs without observing their own export.
import assert from 'node:assert/strict';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { chromium, webkit, expect } from '@playwright/test';

export async function checkSiteLayout({ temp, start, stop, snapshot, until }) {
  for (const engine of process.env.ZELLIJ_PORTABLE_BROWSER?.split(',') || ['chromium', 'webkit']) {
    const root = path.join(temp, `Configured 日本 ${engine}`);
    const content = path.join(root, 'content');
    const output = path.join(root, 'site');
    const config = path.join(root, 'zel.yaml');
    const settings = name => `version: 1\nrenderer: portable\nname: ${name}\npreview: {port: 0}\n`;
    await mkdir(content, { recursive: true });
    await writeFile(config, settings('Before config edit'));
    await writeFile(path.join(content, 'index.md'), '# Configured home\n');
    await writeFile(path.join(root, 'private.md'), '# Unpublished\n[Broken](absent.md)\n');
    const server = await start(engine === 'chromium' ? ['dev', root] : ['v', root, '--watch', '--no-open']);
    const status = () => fetch(`${server.url}_zellij/preview.json`).then(r => r.json());
    const browser = await ({ chromium, webkit })[engine].launch();
    try {
      const page = await browser.newPage();
      await page.goto(server.url);
      await expect(page.locator('.brand')).toHaveText('Before config edit');
      await writeFile(config, settings('After config edit'));
      await expect(page.locator('.brand')).toHaveText('After config edit', { timeout: 20000 });
      await writeFile(path.join(content, 'index.md'), '# Edited content\n');
      await expect(page.getByRole('heading', { name: 'Edited content', exact: true })).toBeVisible({ timeout: 20000 });
      const good = await snapshot(output);
      const version = (await status()).version;
      // Even an invalid Markdown file under output must not trigger rebuilding.
      await writeFile(path.join(output, 'ignored.md'), '[Broken](missing.md)');
      await writeFile(path.join(root, 'private.md'), '# Another unpublished edit');
      await delay(1600);
      assert.equal((await status()).version, version);
      await rm(path.join(output, 'ignored.md'));
      for (const value of [null, 'version: 1\nrenderer: portable\nnav: [', 'name: Legacy layout', 'version: 1\nrenderer: hosted']) {
        if (value === null) await rm(config); else await writeFile(config, value);
        await until(async () => (await status()).error, 'configuration error while watching');
        await expect(page.getByRole('alert')).toBeVisible({ timeout: 20000 });
        assert.equal((await status()).version, version);
        assert.deepEqual(await snapshot(output), good);
      }
      await writeFile(config, settings('Recovered config'));
      await expect(page.locator('.brand')).toHaveText('Recovered config', { timeout: 20000 });
      await expect(page.getByRole('alert')).toHaveCount(0);
      await stop(server);
      // Bare configured-root serving resolves site/ and preview.port without building.
      const previous = await readFile(path.join(output, 'index.html'));
      await writeFile(path.join(content, 'index.md'), '[Broken](missing.md)');
      const served = await start(['s', root]);
      assert((await fetch(served.url).then(r => r.text())).includes('Edited content'));
      await stop(served);
      assert.deepEqual(await readFile(path.join(output, 'index.html')), previous);
    } finally { await browser.close(); }
    console.log(`${engine}: configured preview watches content/config, ignores output, preserves/reloads after errors, and serves without rebuilding.`);
  }
}
