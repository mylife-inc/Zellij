// Browser coverage for the website collection. Called only by the hosted
// template driver, against the actual CLI export, without external services.
import assert from 'node:assert/strict';
import path from 'node:path';

const sites = {
  'product-launch': { secondary: 'design', heading: 'Sound, beautifully shaped.', action: 'Explore the design' },
  'software-platform': { secondary: 'workflow', heading: 'From first idea to shipped.', action: 'Explore the workflow' },
  'creative-studio': { secondary: 'work', heading: 'A point of view. A world of possibilities.', action: 'Discover the work' },
};

async function loadImages(page) {
  await page.evaluate(async () => {
    for (let y = 0; y < document.documentElement.scrollHeight; y += innerHeight) {
      scrollTo(0, y); await new Promise(resolve => requestAnimationFrame(resolve));
    }
  });
  await page.waitForFunction(() => [...document.images].every(img => img.complete && img.naturalWidth > 0));
  await page.evaluate(() => scrollTo(0, 0));
}

export async function checkWebsiteTemplate(browser, url, manifest, artifacts) {
  const site = sites[manifest.id];
  if (!site) return;
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Website browser checks run only in hosted CI');
  for (const width of [1440, 390]) {
    for (const mode of ['light', 'dark']) {
      const context = await browser.newContext({ viewport: { width, height: 1000 },
        colorScheme: mode, reducedMotion: 'reduce' });
      // Test both palettes even for templates with an authored initial mode.
      await context.addInitScript(mode => localStorage.setItem('zellij:mode', mode), mode);
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', e => errors.push(e.message));
      page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
      page.on('response', r => { if (r.status() >= 400) errors.push(`${r.status()} ${r.url()}`); });
      page.on('request', r => { if (new URL(r.url()).origin !== new URL(url).origin) errors.push(`Remote asset: ${r.url()}`); });
      try {
        for (const route of ['', site.secondary, 'guide', 'guide/customize']) {
          await page.goto(new URL(route, url).href, { waitUntil: 'networkidle' });
          assert.equal(await page.locator('h1').count(), 1, `${manifest.id}/${route}: one page heading`);
          if (!route) assert.equal(await page.locator('h1').innerText(), site.heading);
          assert.equal(await page.locator('html').getAttribute('data-zellij-mode'), mode);
          // Scroll the complete page to load lazy images and exercise section visibility.
          await loadImages(page);
          assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${manifest.id}/${route}: overflow`);
          await page.evaluate(() => scrollTo(0, 0));
          await page.screenshot({ path: path.join(artifacts, `${manifest.id}-${width}-${mode}-${route.replaceAll('/', '-') || 'home'}.png`), fullPage: true });
          if (route === 'design') {
            const accordion = page.getByRole('group', { name: 'Design details', exact: true });
            const touch = accordion.getByRole('button', { name: 'Touch', exact: true });
            await touch.focus(); await page.keyboard.press('Enter');
            assert.equal(await touch.getAttribute('aria-expanded'), 'true');
            await page.keyboard.press('ArrowDown');
            assert(await accordion.getByRole('button', { name: 'Place', exact: true }).evaluate(el => el === document.activeElement));
            await page.keyboard.press('Enter');
            assert.equal(await accordion.getByRole('button', { name: 'Place', exact: true }).getAttribute('aria-expanded'), 'true');
          }
        }
        await page.goto(url, { waitUntil: 'networkidle' });
        await page.getByRole('link', { name: site.action, exact: true }).first().click();
        await page.waitForURL(new RegExp(`/${site.secondary}/?$`));
        assert.deepEqual(errors, [], `${manifest.id}: browser/network errors`);
      } finally { await context.close(); }
    }
    // All key content must survive without hydration or motion JavaScript.
    const context = await browser.newContext({ viewport: { width, height: 1000 }, javaScriptEnabled: false });
    try {
      const page = await context.newPage();
      for (const route of ['', site.secondary]) {
        await page.goto(new URL(route, url).href, { waitUntil: 'networkidle' });
        assert.equal(await page.locator('h1').count(), 1);
        assert(await page.locator('h1').isVisible());
        await loadImages(page);
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
        if (route === 'design') {
          const accordion = page.getByRole('group', { name: 'Design details', exact: true });
          assert.equal(await accordion.locator('img').count(), 3);
          assert.equal(await accordion.getByRole('button').count(), 0);
        }
        await page.screenshot({ path: path.join(artifacts, `${manifest.id}-${width}-no-js-${route || 'home'}.png`), fullPage: true });
      }
    } finally { await context.close(); }
  }
}
