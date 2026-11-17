// Browser coverage for the website collection. Called only by the hosted
// template driver, against the actual CLI export, without external services.
import assert from 'node:assert/strict';
import path from 'node:path';
import { expect } from '@playwright/test';

const sites = {
  'api-platform': { secondary: 'models', heading: 'One API for images, video, music and new possibilities.', action: 'Explore the models' },
  'product-launch': { secondary: 'design', heading: 'Sound, beautifully shaped.', action: 'Explore the design' },
  'software-platform': { secondary: 'workflow', heading: 'From first idea to shipped.', action: 'Explore the workflow' },
  'creative-studio': { secondary: 'work', heading: 'A point of view. Made visible.', action: 'Discover the work' },
};

async function loadImages(page) {
  // Drive scrolling/polling from Playwright: page-side animation-frame and
  // timer callbacks cannot settle when JavaScript is deliberately disabled.
  const height = await page.evaluate(() => document.documentElement.scrollHeight);
  for (let y = 0; y < height; y += page.viewportSize().height) {
    await page.evaluate(y => scrollTo(0, y), y);
    await page.waitForTimeout(50);
  }
  // Lazy carousel images can be outside the horizontal viewport even after
  // scrolling the whole document. Visit each card as a reader would.
  for(const picture of await page.locator('.zellij-carousel-track img').all()) {
    await picture.evaluate(el=>el.scrollIntoView({block:'center',inline:'center',behavior:'instant'}));
    await expect.poll(()=>picture.evaluate(el=>el.complete&&el.naturalWidth>0)).toBe(true);
  }
  await page.locator('.zellij-carousel-track').evaluateAll(tracks=>tracks.forEach(track=>track.scrollTo({left:0,behavior:'instant'})));
  let pending = [];
  for (let attempt = 0; attempt < 100; attempt++) {
    pending = await page.evaluate(() => [...document.images]
      .filter(img => !img.complete || !img.naturalWidth).map(img => img.currentSrc || img.src));
    if (!pending.length) break;
    await page.waitForTimeout(100);
  }
  assert.deepEqual(pending, [], 'Every local image must load');
  await page.evaluate(() => scrollTo(0, 0));
}

async function checkHeaderAlignment(page, width) {
  const layout = await page.locator('.zellij-nav-inner').evaluate(bar => {
    const group = bar.querySelector('.zellij-nav-content').getBoundingClientRect();
    const actions = bar.querySelector('.zellij-nav-actions').getBoundingClientRect();
    const brand = bar.querySelector('.zellij-brand').getBoundingClientRect();
    return { center: (group.left + group.right) / 2, viewport: document.documentElement.clientWidth,
      rightInset: document.documentElement.clientWidth - actions.right, brandEnd: brand.right, actionsStart: actions.left };
  });
  if (width > 900) assert(Math.abs(layout.center - layout.viewport / 2) < 2, `Product/menu group must be centered: ${JSON.stringify(layout)}`);
  assert(layout.rightInset >= 15 && layout.rightInset <= 33, 'Controls stay at the right edge');
  assert(layout.brandEnd <= layout.actionsStart, 'Product identity does not overlap controls');
}

async function checkBentoSurfaces(page) {
  const tile = page.locator('.zellij-bento-tile').first();
  if (!await tile.count()) return;
  const results = await tile.evaluate(tile => {
    const section = tile.closest('.zellij-section');
    const result = [];
    // Verify the real authored state before constructing independent variants.
    result.push({ outer: 'authored', inner: tile.getAttribute('data-background'),
      title: getComputedStyle(tile.querySelector('h3')).color,
      body: getComputedStyle(tile.querySelector('p')).color,
      expected: getComputedStyle(tile).color });
    for (const outer of ['default', 'inverted', 'accent']) {
      for (const inner of ['default', 'tinted', 'inverted', 'accent']) {
        const frame = section.cloneNode(false);
        const sample = tile.cloneNode(true);
        frame.setAttribute('data-zellij-background', outer);
        sample.setAttribute('data-background', inner);
        frame.append(sample); section.after(frame);
        try {
          const token = inner === 'inverted' ? '--zellij-inverted-fg' : inner === 'accent' ? '--zellij-accent-fg' : '--zellij-fg';
          const probe = document.createElement('span'); probe.style.color = `var(${token})`;
          sample.append(probe);
          result.push({ outer, inner, title: getComputedStyle(sample.querySelector('h3')).color,
            body: getComputedStyle(sample.querySelector('p')).color, expected: getComputedStyle(probe).color,
            tileColor: getComputedStyle(sample).color,
            tileHtml: sample.outerHTML.slice(0, 400) });
        } finally { frame.remove(); }
      }
    }
    return result;
  });
  if (results.some(r => r.title !== r.expected || r.body !== r.expected)) console.error(JSON.stringify(results, null, 2));
  for (const { outer, inner, title, body, expected } of results) {
    assert.equal(title, expected, `${outer} section / ${inner} tile: heading foreground`);
    assert.equal(body, expected, `${outer} section / ${inner} tile: body foreground`);
  }
}

async function checkStudioTextContrast(page) {
  const samples = await page.locator('.zellij-compose-text[data-tone="muted"]').evaluateAll(elements => {
    const rgba = value => value.match(/[\d.]+/g).map(Number);
    const luminance = rgb => rgb.slice(0, 3).map(v => {
      const s = v / 255; return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    }).reduce((sum, value, i) => sum + value * [0.2126, 0.7152, 0.0722][i], 0);
    return elements.map(el => {
      let parent = el, background;
      while (parent) {
        const candidate = rgba(getComputedStyle(parent).backgroundColor);
        if (candidate.length === 3 || candidate[3] === 1) { background = candidate; break; }
        parent = parent.parentElement;
      }
      if (!background) throw new Error('Muted text needs an explicit opaque background');
      const foreground = luminance(rgba(getComputedStyle(el).color)), back = luminance(background);
      return { text: el.textContent, ratio: (Math.max(foreground, back) + 0.05) / (Math.min(foreground, back) + 0.05) };
    });
  });
  for (const sample of samples) assert(sample.ratio >= 4.5, `Muted studio text contrast ${sample.ratio}: ${sample.text}`);
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
          await checkHeaderAlignment(page, width);
          if (manifest.id === 'creative-studio') await checkStudioTextContrast(page);
          assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${manifest.id}/${route}: overflow`);
          await page.evaluate(() => scrollTo(0, 0));
          await page.screenshot({ path: path.join(artifacts, `${manifest.id}-${width}-${mode}-${route.replaceAll('/', '-') || 'home'}.png`), fullPage: true });
          if (!route) {
            if (manifest.id === 'api-platform') {
              const design = await page.locator('h1').evaluate(el => {
                const css = getComputedStyle(el), root = getComputedStyle(document.documentElement);
                return { font: css.fontFamily, size: css.fontSize, weight: css.fontWeight,
                  align: css.textAlign, accent: root.getPropertyValue('--zellij-accent').trim() };
              });
              assert.match(design.font, /Space Grotesk/);
              assert.equal(design.size, width === 1440 ? '72px' : '38px');
              assert.equal(design.weight, '600');
              assert(['start', 'left'].includes(design.align));
              assert.equal(design.accent, mode === 'dark' ? '#4d86ff' : '#2d6cff');
              assert(await page.evaluate(() => [...document.fonts].some(f => f.family.replaceAll("\"", "").replaceAll("\'", "") === "Zellij Space Grotesk" && f.status === "loaded")));
              const faq = page.locator('.zellij-faq-question').first();
              await faq.focus(); await page.keyboard.press('Enter');
              assert(await faq.evaluate(el => el.parentElement.open));
              await page.evaluate(() => scrollTo(0, 0));
            }
            await page.screenshot({ path: path.join(artifacts, `${manifest.id}-${width}-${mode}-hero.png`) });
            await checkBentoSurfaces(page);
            if (width <= 900) {
              await page.getByRole('button', { name: 'Open navigation', exact: true }).click();
              const drawer = page.getByRole('dialog', { name: 'Site navigation', exact: true });
              await drawer.waitFor({ state: 'visible' });
              assert.equal(await drawer.getByRole('heading', { name: 'Menu', exact: true }).count(), 1);
              assert.equal(await drawer.getByRole('button', { name: 'Back to main menu' }).count(), 0);
              await drawer.screenshot({ path: path.join(artifacts, `${manifest.id}-${width}-${mode}-drawer.png`) });
              await drawer.getByRole('link').first().click();
              await page.waitForSelector('.zellij-drawer-root', { state: 'detached' });
            }
          }
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
        await checkHeaderAlignment(page, width);
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
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'no-preference' });
  try {
    const page = await context.newPage();
    await page.goto(url, { waitUntil: 'networkidle' });
    await loadImages(page);
    await page.waitForFunction(() => [...document.querySelectorAll('[data-zellij-reveal]')]
      .every(el => getComputedStyle(el).opacity === '1'));
    await page.screenshot({ path: path.join(artifacts, `${manifest.id}-1440-motion.png`), fullPage: true });
  } finally { await context.close(); }
}
