// Build an isolated wrapper with a content-owned prefix, then exercise its real
// production HTTP routes. Never rewrite an example's authored configuration.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:net';
import { copyFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, webkit, expect } from '@playwright/test';
import { observeThemeFonts } from './assert-theme-fonts.mjs';
import { themePreferenceFiles, checkThemePreferences } from './check-theme-preferences.mjs';
import { pageConfigFiles, checkPageThemes, checkAnalytics } from './check-page-config.mjs';

const repo = fileURLToPath(new URL('../', import.meta.url));
const require = createRequire(import.meta.url);
const next = require.resolve('next/dist/bin/next');
const cli = path.join(repo, 'engine/packages/zellij/bin/zellij.mjs');
const app = await mkdtemp(path.join(repo, '.zellij-prefix-'));
const prefix = '/products/acme/docs';
const env = { ...process.env, NEXT_TELEMETRY_DISABLED: '1' };
delete env.ZELLIJ_OUTPUT;
delete env.ZELLIJ_BASE_PATH;
let server;
try {
  execFileSync(process.execPath, [cli, 'init', app, '--name', 'prefix-fixture'], { env, stdio: 'inherit' });
  await rm(path.join(app, 'content'), { recursive: true, force: true });
  await rm(path.join(app, 'src/app'), { recursive: true, force: true });
  const files = {
    ...themePreferenceFiles(prefix),
    ...pageConfigFiles(prefix),
    'next.config.js': "import { withZellij } from '@shebka/zellij/next-config';\nexport default withZellij({ typedRoutes: false }, { contentDir: './docs' });\n",
    'src/app/(content)/layout.tsx': "import '@shebka/zellij/styles.css';\nimport { createZellijRootLayout } from '@shebka/zellij';\nexport default createZellijRootLayout({ contentDir: './docs' });\n",
    'src/app/(content)/[[...slug]]/page.tsx': "import { createZellijPages } from '@shebka/zellij';\nconst zellij = createZellijPages({ contentDir: './docs' });\nexport const generateStaticParams = zellij.generateStaticParams;\nexport const generateMetadata = zellij.generateMetadata;\nexport default zellij.Page;\n",
    'src/app/sitemap.ts': "import { createZellijSitemap } from '@shebka/zellij';\nexport const dynamic = 'force-static';\nexport default createZellijSitemap({ contentDir: './docs' });\n",
    'src/app/robots.ts': "import { createZellijRobots } from '@shebka/zellij';\nexport const dynamic = 'force-static';\nexport default createZellijRobots({ contentDir: './docs' });\n",
    'src/app/icon.svg': '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7"/></svg>',
    'docs/site.yaml': `name: Prefix fixture\nbasePath: ${prefix}/\nhome: 01-guide\nsiteUrl: https://docs.example.com\nogImage: assets/og.png\nnav: { search: true, themeToggle: true }\n`,
    'docs/pages.yaml': '- name: media\n  title: Local video\n  sections: [local-video]\n',
    'docs/sections/local-video.yaml': 'type: video-embed\nheading: Local demo\nsrc: assets/demo.mp4\nposter: assets/og.png\n',
    'docs/01-guide/_sidebar.yaml': 'route: /manual\nnav: [intro, nested/detail]\n',
    'docs/01-guide/intro.mdx': '# Introduction\n\nWelcome.\n\n## Find this section\n\nThe nebula fixture proves search navigation.\n\n### **API** `options`\n\nFormatted heading content.\n\n#### Nested *details*\n\nMore detail.\n\n![Local asset](/_zellij/assets/og.png)\n\n<Screenshot src="assets/og.png" alt="Optimized asset" width={1200} height={630} />\n\n[Read the details](/manual/nested/detail)\n',
    'docs/01-guide/nested/detail.mdx': '# Details\n\nThe nested route works.\n\n[Back to introduction](/manual/intro)\n',
  };
  for (const [name, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(app, name)), { recursive: true });
    await writeFile(path.join(app, name), text);
  }
  await mkdir(path.join(app, 'docs/assets'));
  await copyFile(path.join(repo, 'examples/showcase/content/assets/og.png'), path.join(app, 'docs/assets/og.png'));
  await copyFile(path.join(repo, 'scripts/fixtures/template.mp4'), path.join(app, 'docs/assets/demo.mp4'));
  execFileSync(process.execPath, [cli, 'prepare', '--content', './docs'], { cwd: app, env, stdio: 'inherit' });
  execFileSync(process.execPath, [next, 'build', ...(process.env.ZELLIJ_TEST_WEBPACK ? ['--webpack'] : [])], { cwd: app, env, stdio: 'inherit' });

  const reservation = createServer();
  reservation.listen(0, '127.0.0.1');
  await once(reservation, 'listening');
  const port = reservation.address().port;
  await new Promise((resolve, reject) => reservation.close((error) => error ? reject(error) : resolve()));
  const origin = `http://127.0.0.1:${port}`;
  server = spawn(process.execPath, [next, 'start', '--hostname', '127.0.0.1', '--port', String(port)], { cwd: app, env, stdio: 'inherit' });
  let ready = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (server.exitCode !== null) throw new Error(`Next server exited: ${server.exitCode}`);
    try {
      const response = await fetch(`${origin}${prefix}/manual/intro`, { signal: AbortSignal.timeout(1000) });
      if (response.ok) { ready = true; break; }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  assert(ready, 'Prefixed production server did not become ready');
  assert.equal((await fetch(`${origin}/manual/intro`)).status, 404, 'Unprefixed routes must not silently serve the fixture');
  const index = await (await fetch(`${origin}${prefix}/_zellij/search-index.json`)).json();
  assert(index.some((entry) => entry.route === '/manual/intro#find-this-section'));
  assert(index.every((entry) => !entry.route.startsWith(prefix)));
  const sitemap = await (await fetch(`${origin}${prefix}/sitemap.xml`)).text();
  assert(sitemap.includes(`https://docs.example.com${prefix}/manual/nested/detail`));
  assert((await (await fetch(`${origin}${prefix}/robots.txt`)).text()).includes(`${prefix}/sitemap.xml`));

  const browsers = process.env.CI ? { chromium, webkit } : { chromium };
  for (const [name, engine] of Object.entries(browsers)) {
    const browser = await engine.launch();
    let context;
    try {
      await checkThemePreferences(browser, origin, prefix);
      console.log(`${name}: independent theme/mode restoration before hydration, navigation, stale preferences and unavailable storage passed.`);
      await checkPageThemes(browser, origin, prefix);
      console.log(`${name}: page theme tokens/presentation, modes, shared chrome, overrides, navigation and JavaScript-disabled rendering passed.`);
      await checkAnalytics(browser, origin, prefix);
      console.log(`${name}: real Plausible pageviews, client navigation/history, disabled/ignored/local visits and blocked requests passed without transmitting analytics.`);
      context = await browser.newContext();
      await context.tracing.start({ screenshots: true, snapshots: true });
      const page = await context.newPage();
      const checkFonts = observeThemeFonts(page, origin, prefix);
      const failures = [];
      page.on('pageerror', (error) => failures.push(error.message));
      page.on('response', (response) => {
        if (response.url().startsWith(origin) && response.status() >= 400) failures.push(`${response.status()} ${response.url()}`);
      });
      // Home and the guide root both redirect, without losing/doubling prefix.
      await page.goto(`${origin}${prefix}/`);
      await expect(page).toHaveURL(`${origin}${prefix}/manual/intro`);
      await expect(page.locator('h1')).toHaveText('Introduction');
      await checkFonts();
      for (const [level, title, id] of [[2, 'Find this section', 'find-this-section'], [3, 'API options', 'api-options'], [4, 'Nested details', 'nested-details']]) {
        const heading = page.getByRole('heading', { level, name: title, exact: true });
        await expect(heading).toBeVisible();
        const permalink = heading.getByRole('link', { name: title, exact: true });
        await expect(permalink).toHaveAttribute('href', `#${id}`);
        await permalink.focus();
        await expect(permalink).toBeFocused();
        await permalink.press('Enter');
        await expect(page).toHaveURL(`${origin}${prefix}/manual/intro#${id}`);
      }

      const internal = await page.locator('a[href^="/"]').evaluateAll((links) => links.map((link) => link.getAttribute('href')));
      assert(internal.length > 0 && internal.every((href) => href === prefix || href.startsWith(`${prefix}/`)), `Every internal link must retain the prefix: ${JSON.stringify(internal)}`);
      const resources = await page.locator('link[rel="stylesheet"], link[as="font"], script[src]').evaluateAll((nodes) => nodes.map((node) => node.getAttribute('href') ?? node.getAttribute('src')));
      assert(resources.length > 0 && resources.every((url) => new URL(url, origin).pathname.startsWith(`${prefix}/`)), 'Next resources must carry the prefix');
      await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', `https://docs.example.com${prefix}/manual/intro`);
      for (const selector of ['meta[property="og:image"]', 'meta[name="twitter:image"]']) {
        const url = new URL(await page.locator(selector).first().getAttribute('content'));
        assert.equal(url.pathname, `${prefix}/_zellij/assets/og.png`);
      }
      for (const alt of ['Local asset', 'Optimized asset']) {
        const img = page.getByAltText(alt, { exact: true });
        await img.scrollIntoViewIfNeeded();
        await expect.poll(() => img.evaluate((node) => node.complete && node.naturalWidth > 0)).toBe(true);
      }
      await page.getByRole('link', { name: 'Read the details', exact: true }).click();
      await expect(page).toHaveURL(`${origin}${prefix}/manual/nested/detail`);
      await expect(async () => {
        await page.keyboard.press('ControlOrMeta+k');
        await expect(page.getByRole('dialog', { name: 'Search the guide' })).toBeVisible({ timeout: 1000 });
      }).toPass({ timeout: 15000 });
      await expect(page.locator('.zellij-search-empty')).toContainText('Start typing');
      await page.locator('.zellij-search-input').fill('nebula');
      await page.locator('.zellij-search-result').filter({ hasText: 'Find this section' }).click();
      await expect(page).toHaveURL(`${origin}${prefix}/manual/intro#find-this-section`);
      await page.goto(`${origin}${prefix}/media`);
      const play = page.getByRole('button', { name: 'Play video: Local demo', exact: true });
      await expect(play.locator('img')).toHaveJSProperty('complete', true);
      await expect.poll(() => play.locator('img').evaluate(img => img.naturalWidth)).toBeGreaterThan(0);
      await play.click();
      const video = page.locator('video');
      await expect(video).toHaveAttribute('src', new RegExp(`^${prefix}/_zellij/assets/demo\\.mp4(?:\\?|$)`));
      await expect(video).toHaveAttribute('poster', new RegExp(`^${prefix}/_zellij/assets/og\\.png(?:\\?|$)`));
      await expect(video).toHaveJSProperty('videoWidth', 32, { timeout: 15000 });
      const videoUrl = await video.getAttribute('src');
      assert(videoUrl);
      assert.equal((await fetch(new URL(videoUrl, origin))).status, 200);
      assert.deepEqual(failures, [], `${name}: browser resources and hydration must succeed`);
      console.log(`${name}: prefixed redirects, accessible heading permalinks, decoded local video/poster, links, images, Next resources, metadata, sitemap and search passed.`);
      await context.tracing.stop();
    } catch (error) {
      if (context) {
        const artifacts = path.join(repo, 'examples/showcase/test-results');
        await mkdir(artifacts, { recursive: true });
        await context.tracing.stop({ path: path.join(artifacts, `basepath-${name}.zip`) });
      }
      throw error;
    } finally { await browser.close(); }
  }
} finally {
  if (server && server.exitCode === null) {
    const exited = once(server, 'exit');
    server.kill('SIGTERM');
    const timer = setTimeout(() => server.kill('SIGKILL'), 5000);
    await exited;
    clearTimeout(timer);
  }
  await rm(app, { recursive: true, force: true });
}
