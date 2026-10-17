// The native binary must produce a movable site that works without HTTP.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium, webkit, expect } from '@playwright/test';
import { checkConfiguration } from './portable-config.mjs';

const binary = path.resolve(process.argv[2] || `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const temp = await mkdtemp(path.join(tmpdir(), 'zellij-portable-'));
const source = path.join(temp, 'Markdown 日本');
const output = path.join(temp, 'export');
const moved = path.join(temp, 'Moved documentation café 日本');
const files = {
  'README.md': '# Portable handbook\n\nA complete, movable documentation site.\n\n[Install now](guide/Install%20Now.md#setup)\n\n[Japanese](guide/日本.md)\n\n![Local diagram](assets/local%20diagram.svg)\n\n## Start here\n\nRead the guide, or use search to find a topic.\n\n```rust\nfn main() { println!("Hello, portable docs!"); }\n```\n\n| Feature | Status |\n| --- | --- |\n| Offline | Ready |\n\n- [x] No server required\n',
  'guide/index.md': '# Guide\n\n[Install](Install%20Now.md)\n',
  'guide/Install Now.md': '# Install now\n\n## Setup\n\nEverything is local. [Home](../README.md?from=guide#start-here).\n\n## Finding nebula\n\nThe nebula example makes offline search easy to verify.\n\n## Setup\n\nA repeated heading receives a distinct anchor.\n\n[Second setup](#setup-1)\n',
  'guide/日本.md': '# 日本語のガイド\n\n## はじめに\n\nUnicode paths work after moving the whole folder. [Home](../README.md)\n',
  'assets/local diagram.svg': '<svg xmlns="http://www.w3.org/2000/svg" width="640" height="120" viewBox="0 0 640 120"><rect width="640" height="120" rx="12" fill="#194b3b"/><text x="32" y="70" fill="white" font-size="26">Markdown → portable documentation</text></svg>',
};
const env = { ...process.env, PATH: path.join(temp, 'no-external-tools') };
try {
  for (const [relative, body] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(source, relative)), { recursive: true });
    await writeFile(path.join(source, relative), body);
  }
  execFileSync(binary, ['build', source, '--out', output, '--name', 'Acme documentation'], { env, stdio: 'pipe' });
  for (const [relative, body] of Object.entries(files)) assert.equal(await readFile(path.join(source, relative), 'utf8'), body);
  const first = await readFile(path.join(output, 'index.html'));
  await writeFile(path.join(source, 'broken.md'), '# Broken\n\n[Missing](missing.md)\n');
  assert.throws(() => execFileSync(binary, ['build', source, '--out', output], { env, stdio: 'pipe' }));
  assert.deepEqual(await readFile(path.join(output, 'index.html')), first);
  await rm(path.join(source, 'broken.md'));
  // Explicit target and default detection must select the same native renderer.
  execFileSync(binary, ['build', source, '--target', 'portable', '--out', output, '--name', 'Acme documentation'], { env, stdio: 'pipe' });
  await rename(output, moved);
  console.log('Native build, failure preservation and relocation passed without external tools.');
  assert.equal((await readdir(path.join(moved, '_zellij/fonts/files'))).length, 75);
  const home = pathToFileURL(path.join(moved, 'index.html')).href;
  const nested = pathToFileURL(path.join(moved, 'guide/Install Now.html')).href;
  const selected = process.env.ZELLIJ_PORTABLE_BROWSER?.split(',') || ['chromium', 'webkit'];
  for (const engine of selected) {
    const browser = await ({ chromium, webkit })[engine].launch({ headless: true });
    try {
      // WebKit's emulated offline state rejects even a plain file:// heading
      // before loading. Block HTTP directly for it; never relax file security.
      const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, offline: engine !== 'webkit', serviceWorkers: 'block' });
      context.setDefaultTimeout(10000);
      const requests = [];
      const errors = [];
      context.on('request', request => requests.push(request.url()));
      await context.route(/^https?:/, route => route.abort());
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(home);
      await expect(page.getByRole('heading', { name: 'Portable handbook' })).toBeVisible();
      await expect(page.getByRole('button', { name: /Search/ })).toBeVisible();
      console.log(`${engine}: file navigation and scripts loaded.`);
      assert.equal(await page.locator('article img').evaluate(image => image.complete && image.naturalWidth > 0), true);
      assert(await page.locator('.code-block span[style]').count() > 0, 'Code has build-time syntax colors');
      await page.getByLabel('Theme', { exact: true }).selectOption('forest');
      await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'dark');
      assert.equal(await page.evaluate(async () => {
        await document.fonts.load('16px "Zellij Inter"');
        await document.fonts.ready;
        return document.fonts.check('16px "Zellij Inter"');
      }), true, 'Bundled WOFF2 decodes through file://');
      await page.keyboard.press('Control+k');
      await expect(page.getByRole('dialog')).toBeVisible();
      await page.getByRole('searchbox').fill('nebula');
      await expect(page.getByRole('status')).toHaveText('1 matching page.');
      await page.getByRole('searchbox').press('ArrowDown');
      await expect(page.locator('#_zel-results a')).toBeFocused();
      await page.keyboard.press('Enter');
      await expect(page.getByRole('heading', { name: 'Install now', exact: true })).toBeVisible();
      assert.equal(new URL(page.url()).hash, '#finding-nebula');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'dark');
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      assert.equal(new URL(page.url()).searchParams.get('from'), 'guide');
      assert.equal(new URL(page.url()).hash, '#start-here');
      await expect(page.getByRole('heading', { name: 'Portable handbook' })).toBeVisible();
      await page.locator('article').getByRole('link', { name: 'Japanese' }).click();
      await expect(page.getByRole('heading', { name: '日本語のガイド' })).toBeVisible();
      // A fresh deep link has no warmed router, data cache, or initial home load.
      const fresh = await context.newPage();
      await fresh.goto(nested);
      await expect(fresh.getByRole('heading', { name: 'Install now', exact: true })).toBeVisible();
      await fresh.locator('article').getByRole('link', { name: 'Second setup' }).click();
      assert.equal(new URL(fresh.url()).hash, '#setup-1');
      await fresh.close();
      await page.goto(home);
      await page.setViewportSize({ width: 390, height: 844 });
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'No mobile horizontal overflow');
      await page.getByRole('button', { name: 'Contents', exact: true }).click();
      await expect(page.getByRole('navigation', { name: 'Documentation', exact: true })).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(page.getByRole('button', { name: 'Contents', exact: true })).toBeFocused();
      await page.getByRole('button', { name: /Search/ }).click();
      await page.getByRole('searchbox').fill('nothing matches this');
      await expect(page.getByRole('status')).toHaveText('No matching pages.');
      await page.keyboard.press('Escape');
      await expect(page.getByRole('dialog')).not.toBeVisible();
      await expect(page.getByRole('button', { name: /Search/ })).toBeFocused();
      if (process.env.ZELLIJ_PORTABLE_ARTIFACTS) {
        await mkdir(process.env.ZELLIJ_PORTABLE_ARTIFACTS, { recursive: true });
        await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-mobile.png`), fullPage: true });
        await page.setViewportSize({ width: 1280, height: 900 });
        await page.getByLabel('Appearance', { exact: true }).selectOption('light');
        await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-desktop.png`), fullPage: true });
      }
      assert.deepEqual(errors, []);
      assert(requests.every(url => url.startsWith('file:') || url.startsWith('data:')), 'No network requests or HTTP server');
      await context.close();
      const noJS = await browser.newContext({ javaScriptEnabled: false, offline: engine !== 'webkit', serviceWorkers: 'block' });
      noJS.on('request', request => requests.push(request.url()));
      await noJS.route(/^https?:/, route => route.abort());
      const plain = await noJS.newPage();
      await plain.goto(nested);
      await expect(plain.getByRole('heading', { name: 'Install now', exact: true })).toBeVisible();
      await expect(plain.getByRole('navigation', { name: 'Documentation', exact: true })).toBeVisible();
      await plain.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      await expect(plain.getByRole('heading', { name: 'Portable handbook' })).toBeVisible();
      await expect(plain.locator('article table')).toBeVisible();
      assert(requests.every(url => url.startsWith('file:') || url.startsWith('data:')), 'No-JS content also needs no network');
      await noJS.close();
      await checkConfiguration({ binary, root: temp, env, engine, browser });
      console.log(`${engine}: moved file:// site, nested links, search, keyboard, themes, local fonts/media, mobile and no-JS content passed.`);
    } finally { await browser.close(); }
  }
} finally { await rm(temp, { recursive: true, force: true }); }
