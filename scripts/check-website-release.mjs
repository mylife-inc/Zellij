// Source/public CLI and engine acceptance. Actual browsers and containers run in CI.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { chromium, expect } from '@playwright/test';
import YAML from 'yaml';
import { checkWebsiteTemplate } from './check-website-templates.mjs';

assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Website runtime checks run only in hosted CI');
const zel = path.resolve(process.argv[2] || 'cli/target/debug/zel');
const captures = path.resolve('test-results/websites');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-website-release-'));
const run = args => execFileSync(zel, args, { encoding: 'utf8', timeout: 600000, stdio: ['ignore', 'pipe', 'pipe'] });
const records = [];
let browser;
async function serve(output, check) {
  const server = createServer(async (req, res) => {
    try {
      let file = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      if (file.endsWith('/')) file += 'index.html'; else if (!path.extname(file)) file += '.html';
      const selected = path.resolve(output, '.' + file);
      assert(selected.startsWith(output + path.sep));
      const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.json': 'application/json', '.txt': 'text/plain' };
      res.setHeader('Content-Type', types[path.extname(selected)] || 'application/octet-stream');
      res.end(await readFile(selected));
    } catch { res.writeHead(404); res.end(); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try { await check(`http://127.0.0.1:${server.address().port}/`); }
  finally { await new Promise(resolve => server.close(resolve)); }
}
try {
  await mkdir(captures, { recursive: true });
  const catalog = JSON.parse(run(['templates', '--json', '--offline']));
  browser = await chromium.launch();
  for (const id of ['product-launch', 'software-platform', 'creative-studio', 'api-platform']) {
    const manifest = catalog.templates.find(template => template.id === id);
    assert(manifest, `Released catalog must include ${id}`);
    const site = path.join(root, id);
    run(['create', site, '-t', `${id}@${manifest.version}`, '--offline']);
    run(['build', site, '--target', 'static']);
    await serve(path.join(site, 'site'), url => checkWebsiteTemplate(browser, url, manifest, captures));
    records.push({ id, version: manifest.version, pages: ['', id === 'product-launch' ? 'design' : id === 'software-platform' ? 'workflow' : id === 'api-platform' ? 'models' : 'work', 'guide', 'guide/customize'] });
  }

  // A separate authored fixture exercises section navigation and actual company
  // configuration without changing any bundled/published template archive.
  const site = path.join(root, 'navigation');
  run(['create', site, '-t', 'product-launch', '--offline']);
  const configPath = path.join(site, 'content/site.yaml');
  const config = YAML.parse(await readFile(configPath, 'utf8'));
  config.brand = { company: { src: 'assets/owner.svg', alt: 'Site owner', href: 'https://company.example/', newTab: true } };
  await writeFile(configPath, YAML.stringify(config));
  await writeFile(path.join(site, 'content/assets/owner.svg'), '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><path fill="#007aff" d="M2 2h28v28H2z"/></svg>');
  await writeFile(path.join(site, 'content/menu.yaml'), YAML.stringify([
    { name: 'Overview', page: 'home' },
    { name: 'Documentation', items: [{ name: 'Read', items: [
      { name: 'Guide', page: '01-guide', description: 'Start with the guide.', icon: 'book-open' },
      { name: 'Source', url: 'https://source.example/', newTab: true },
    ] }] },
    { name: 'Design', page: 'design' },
  ]));
  run(['build', site, '--target', 'static']);
  await serve(path.join(site, 'site'), async url => {
    for (const mode of ['light', 'dark']) {
      const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: mode, reducedMotion: 'reduce' });
      await context.addInitScript(mode => localStorage.setItem('zellij:mode', mode), mode);
      await context.route('https://*.example/**', route => route.fulfill({ contentType: 'text/html', body: '<title>External destination</title>' }));
      try {
        const page = await context.newPage(), errors = [];
        page.on('pageerror', e => errors.push(e.message));
        await page.goto(url, { waitUntil: 'networkidle' });
        await expect(page.locator('.zellij-company')).toHaveAttribute('href', 'https://company.example/');
        await expect(page.locator('.zellij-company')).toHaveAttribute('target', '_blank');
        for (const width of [1440, 390, 320]) {
          await page.setViewportSize({ width, height: 900 });
          await expect.poll(() => page.locator('.zellij-nav-inner').evaluate(bar => {
            const company = bar.querySelector('.zellij-company').getBoundingClientRect();
            const product = bar.querySelector('.zellij-brand').getBoundingClientRect();
            const group = bar.querySelector('.zellij-nav-content').getBoundingClientRect();
            const actions = bar.querySelector('.zellij-nav-actions').getBoundingClientRect();
            const width = document.documentElement.clientWidth;
            return company.left >= 15 && company.left <= 33 && company.right <= product.left && product.right <= actions.left &&
              (width <= 900 || Math.abs((group.left + group.right) / 2 - width / 2) < 2) && document.documentElement.scrollWidth <= width;
          })).toBe(true);
        }
        await page.setViewportSize({ width: 390, height: 844 });
        const trigger = page.getByRole('button', { name: 'Open navigation', exact: true });
        await trigger.focus(); await page.keyboard.press('Enter');
        const drawer = page.getByRole('dialog', { name: 'Site navigation', exact: true });
        await expect(drawer).toBeFocused();
        await drawer.screenshot({ path: path.join(captures, `navigation-${mode}-root.png`) });
        await drawer.getByRole('button', { name: 'Documentation', exact: true }).click();
        await expect(drawer.getByRole('heading', { name: 'Documentation', exact: true })).toBeFocused();
        await expect(drawer.getByRole('heading', { name: 'Read', exact: true })).toBeVisible();
        await expect(drawer.getByText('Start with the guide.', { exact: true })).toBeVisible();
        await drawer.screenshot({ path: path.join(captures, `navigation-${mode}-section.png`) });
        await drawer.getByRole('button', { name: 'Back to main menu' }).click();
        await expect(drawer.getByRole('button', { name: 'Documentation', exact: true })).toBeFocused();
        await expect(drawer.locator('.zellij-drawer-view')).toHaveCSS('animation-name', 'none');
        await drawer.getByRole('button', { name: 'Documentation', exact: true }).click();
        const source = drawer.getByRole('link', { name: 'Source', exact: true });
        await expect(source).toHaveAttribute('target', '_blank');
        const [popup] = await Promise.all([context.waitForEvent('page'), source.click()]);
        await expect(popup).toHaveURL('https://source.example/'); await popup.close();
        await expect(drawer).toHaveCount(0);
        await trigger.click();
        await expect(drawer.getByRole('heading', { name: 'Menu', exact: true })).toBeVisible();
        await drawer.getByRole('button', { name: 'Close navigation' }).click();
        await expect(drawer).toHaveCount(0);
        assert.deepEqual(errors, []);
      } finally { await context.close(); }
    }
  });
  await writeFile(path.join(captures, 'websites.json'), JSON.stringify({ templates: records, navigation: ['company alignment', 'root/section/back/close', 'external new tab', 'reduced motion'] }, null, 2) + '\n');
  console.log('Four website templates and company/mobile navigation passed with the selected CLI and engine.');
} finally {
  if (browser) await browser.close();
  await rm(root, { recursive: true, force: true });
}
