// Configured sites must retain their branding and authored navigation offline.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect } from '@playwright/test';

export async function checkConfiguration({ binary, root, env, engine, browser }) {
  const source = path.join(root, `configured-${engine}`);
  const output = path.join(root, `configured-output-${engine}`);
  const moved = path.join(root, `Moved configured 日本 ${engine}`);
  const config = `name: Acme reference\ntheme: forest\nbrand:\n  logo:\n    src: brand/company & mark.svg\n    alt: Acme company\n    href: https://example.com/company\nnav:\n  - page: README.md\n    label: Overview\n  - label: Getting started\n    children:\n      - page: guide/Start Here.md\n        label: Installation\n      - label: Reference\n        children:\n          - page: guide/api.md\n            label: API reference\n  - label: Company website\n    href: https://example.com/company\n`;
  const files = {
    'zel.yaml': config,
    'README.md': '# Configured home\n',
    'guide/Start Here.md': '# Install the product\n',
    'guide/api.md': '# API details\n',
    'unlisted.md': '# Unlisted appendix\n\nSearchable amber reference.\n',
    'brand/company & mark.svg': '<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><rect width="80" height="40" rx="8" fill="#197052"/><text x="15" y="27" fill="white" font-size="20">ACME</text></svg>',
  };
  for (const [relative, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(source, relative)), { recursive: true });
    await writeFile(path.join(source, relative), text);
  }
  execFileSync(binary, ['build', source, '--out', output], { env, stdio: 'pipe' });
  await rename(output, moved);
  const requests = [];
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, offline: engine === 'chromium', serviceWorkers: 'block' });
  await context.route(/^https?:/, route => route.abort());
  context.on('request', request => requests.push(request.url()));
  // Earlier exports saved initial defaults too. Do not let that old state
  // masquerade as an explicit preference and override the new project theme.
  await context.addInitScript(() => {
    if (!location.pathname.endsWith('/index.html')) return;
    try {
      const key = `zellij-docs:${new URL('./', location.href).pathname}`;
      if (!localStorage.getItem(key)) localStorage.setItem(key, JSON.stringify({ theme: 'cupertino' }));
    } catch {}
  });
  const errors = [];
  const home = pathToFileURL(path.join(moved, 'index.html')).href;
  try {
    const page = await context.newPage();
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(home);
    await expect(page.locator('.brand')).toHaveText('Acme reference');
    await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
    await expect(page.locator('.brand-logo')).toHaveAttribute('href', 'https://example.com/company');
    await expect(page.locator('.brand-logo img')).toHaveAttribute('alt', 'Acme company');
    assert.equal(await page.locator('.brand-logo img').evaluate(img => img.complete && img.naturalWidth), 80);
    const nav = page.getByRole('navigation', { name: 'Documentation', exact: true });
    await expect(nav.getByRole('link')).toHaveText(['Overview', 'Installation', 'API reference', 'Company website']);
    await expect(nav.locator('.nav-label')).toHaveText(['Getting started', 'Reference']);
    await page.locator('.paging a[rel=next]').click();
    await expect(page.getByRole('heading', { name: 'Install the product' })).toBeVisible();
    await expect(nav.locator('[aria-current=page]')).toHaveText('Installation');
    await expect(page.locator('.paging a[rel=prev]')).toContainText('Overview');
    await expect(page.locator('.paging a[rel=next]')).toContainText('API reference');
    await expect(page.locator('.brand-logo img')).toHaveAttribute('src', '../brand/company%20%26%20mark.svg');
    await page.locator('.paging a[rel=next]').click();
    await expect(page.getByRole('heading', { name: 'API details' })).toBeVisible();
    await page.keyboard.press('Control+k');
    await page.getByRole('searchbox').fill('amber');
    await page.locator('#_zel-results a').click();
    await expect(page.getByRole('heading', { name: 'Unlisted appendix' })).toBeVisible();
    await expect(page.locator('.paging a')).toHaveCount(0);
    await page.locator('.brand').click();
    await expect(page.getByRole('heading', { name: 'Configured home' })).toBeVisible();
    if (process.env.ZELLIJ_PORTABLE_ARTIFACTS) {
      await mkdir(process.env.ZELLIJ_PORTABLE_ARTIFACTS, { recursive: true });
      await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-configured-desktop.png`), fullPage: true });
    }
    await page.setViewportSize({ width: 390, height: 844 });
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
    const mobileHeader = await page.locator('.header').evaluate(header => {
      const title = header.querySelector('.brand').getBoundingClientRect();
      const logo = header.querySelector('.brand-logo').getBoundingClientRect();
      const controls = header.querySelector('.controls').getBoundingClientRect();
      return { titleHeight: title.height, titleLeft: title.left, logoRight: logo.right,
        brandBottom: Math.max(title.bottom, logo.bottom), controlsTop: controls.top,
        height: header.getBoundingClientRect().height };
    });
    assert(mobileHeader.titleHeight < 30, 'The short company title fits on one line');
    assert(mobileHeader.titleLeft >= mobileHeader.logoRight, 'The title clears the logo');
    assert(mobileHeader.controlsTop >= mobileHeader.brandBottom, 'Mobile controls use a separate row');
    assert(mobileHeader.height < 140, 'The mobile header leaves room for documentation');
    await page.getByRole('button', { name: 'Contents', exact: true }).click();
    await expect(nav).toBeVisible();
    if (process.env.ZELLIJ_PORTABLE_ARTIFACTS) await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-configured-mobile.png`), fullPage: true });
    await nav.getByRole('link', { name: 'API reference', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'API details' })).toBeVisible();
    assert.deepEqual(errors, []);
  } finally { await context.close(); }
  const noJS = await browser.newContext({ javaScriptEnabled: false, offline: engine === 'chromium', serviceWorkers: 'block' });
  await noJS.route(/^https?:/, route => route.abort());
  noJS.on('request', request => requests.push(request.url()));
  try {
    const page = await noJS.newPage();
    await page.goto(home);
    await page.getByRole('navigation', { name: 'Documentation', exact: true }).getByRole('link', { name: 'Installation', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Install the product' })).toBeVisible();
    assert.equal(await page.locator('.brand-logo img').evaluate(img => img.complete && img.naturalWidth), 80);
  } finally { await noJS.close(); }
  assert(requests.every(url => ['file:', 'data:'].includes(new URL(url).protocol)), 'Configured output has no HTTP dependency');
  for (const [relative, text] of Object.entries(files)) assert.equal(await readFile(path.join(source, relative), 'utf8'), text);
  console.log(`${engine}: configured name/theme, local company logo, nested navigation, paging, unlisted search, mobile and no-JS offline output passed.`);
}
