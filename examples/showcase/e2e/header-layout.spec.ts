import { test, expect, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { browserFailureState } from '../../../scripts/browser-failure-state.mjs';

test.afterEach(async ({ page }, info) => {
  if (info.status === info.expectedStatus) return;
  try {
    const output = info.outputPath('header-failure-state.json');
    await writeFile(output, JSON.stringify(await browserFailureState(page), null, 2));
    await info.attach('header-failure-state', { path: output, contentType: 'application/json' });
  } catch (error) {
    console.error(`Header diagnostic capture failed: ${error}`);
  }
});

// Delay application work without stopping native resize/media delivery. A single
// batch can commit the new menu while its focus-restoration frame stays queued.
async function holdApplicationFrames(page: Page) {
  await page.evaluate(() => {
    const request = window.requestAnimationFrame.bind(window);
    const cancel = window.cancelAnimationFrame.bind(window);
    const callbacks = new Map<number, FrameRequestCallback>();
    let held = true;
    const render = () => { if (held) request(render); };
    request(render);
    let id = 0;
    window.requestAnimationFrame = callback => {
      if (!held) return request(callback);
      callbacks.set(--id, callback);
      return id;
    };
    window.cancelAnimationFrame = handle => {
      if (handle < 0) callbacks.delete(handle);
      else cancel(handle);
    };
    const advance = () => new Promise<void>(resolve => request(() => request(time => {
      for (const handle of [...callbacks.keys()]) {
        const callback = callbacks.get(handle);
        callbacks.delete(handle);
        callback?.(time);
      }
      request(() => request(() => resolve()));
    })));
    const state = {
      events: 0,
      advance,
      release: () => { held = false; return advance(); },
    };
    (window as unknown as { delayedFocusFrames: typeof state }).delayedFocusFrames = state;
    matchMedia('(max-width: 900px)').addEventListener('change', () => state.events++);
  });
}

test('product navigation stays centered while controls stay at the viewport edge', async ({ page }) => {
  await page.goto('/');
  for (const width of [1600, 1280, 1024, 901, 900, 390, 360, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await expect.poll(async () => page.locator('.zellij-nav-inner').evaluate((bar) => {
      const brand = bar.querySelector('.zellij-brand')!.getBoundingClientRect();
      const actions = bar.querySelector('.zellij-nav-actions')!.getBoundingClientRect();
      const width = document.documentElement.clientWidth;
      const links = [...bar.querySelectorAll('.zellij-nav-list:not(.zellij-nav-measure) > li > .zellij-nav-link')]
        .filter((link) => link.getClientRects().length).map((link) => link.getBoundingClientRect());
      const content = bar.querySelector('.zellij-nav-content')!.getBoundingClientRect();
      const aligned = width > 900 ? Math.abs((content.left + content.right) / 2 - width / 2) < 2 : brand.left >= 15 && brand.left <= 33;
      return aligned && links.every((link) => link.left >= brand.right && link.right <= actions.left) && width - actions.right >= 15 &&
        width - actions.right <= 33 && brand.right <= actions.left && document.documentElement.scrollWidth <= width;
    })).toBe(true);
  }
});

test('company stays far left without shifting the product/menu center', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/');
  await expect(page.locator('.zellij-nav-desktop')).toHaveAttribute('data-measured', 'true');
  await page.locator('.zellij-nav-company').evaluate(slot => {
    const link = document.createElement('a');
    link.className = 'zellij-company';
    link.href = 'https://company.example';
    link.setAttribute('aria-label', 'Company');
    // Exercise the largest permitted company image, not an unsupported text mark.
    const image = document.createElement('img');
    image.className = 'zellij-company-image';
    image.alt = 'Company';
    image.src = '/_zellij/assets/logo.svg';
    image.style.width = '48px';
    link.append(image);
    slot.append(link);
  });
  for (const width of [1440, 1024, 901, 390, 320, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await expect.poll(() => page.locator('.zellij-nav-inner').evaluate(bar => {
      const company = bar.querySelector('.zellij-company')!.getBoundingClientRect();
      const brand = bar.querySelector('.zellij-brand')!.getBoundingClientRect();
      const content = bar.querySelector('.zellij-nav-content')!.getBoundingClientRect();
      const actions = bar.querySelector('.zellij-nav-actions')!.getBoundingClientRect();
      const width = document.documentElement.clientWidth;
      return company.left >= 15 && company.left <= 33 && company.right <= brand.left && brand.right <= actions.left &&
        (width <= 900 || Math.abs((content.left + content.right) / 2 - width / 2) < 2) &&
        document.documentElement.scrollWidth <= width;
    })).toBe(true);
  }
  await page.setViewportSize({ width: 320, height: 900 });
  await page.getByRole('button', { name: 'Open navigation' }).click();
  const drawer = page.getByRole('dialog', { name: 'Site navigation' });
  await expect(drawer.getByRole('link', { name: 'GitHub repository' })).toHaveAttribute('href', 'https://github.com/shebka/zellij');
  await page.keyboard.press('Escape');
  await page.setViewportSize({ width: 1440, height: 900 });
  // Reserve extra right-hand space without letting it pull the menu off center.
  await page.locator('.zellij-nav-actions').evaluate(actions => { actions.style.paddingLeft = '80px'; });
  await expect.poll(() => page.locator('.zellij-nav-content').evaluate(content => {
    const box = content.getBoundingClientRect();
    return Math.abs((box.left + box.right) / 2 - document.documentElement.clientWidth / 2);
  })).toBeLessThan(2);
});

test.describe('measured overflow', () => {
  test.skip(({ isMobile }) => Boolean(isMobile), 'desktop keyboard and resize behavior');

  test('moves trailing links into More and restores them when space returns', async ({ page }) => {
    await page.goto('/', { waitUntil: 'networkidle' });
    await page.evaluate(() => document.fonts.ready);
    const nav = page.getByRole('navigation', { name: 'Main', exact: true });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    await page.setViewportSize({ width: 901, height: 900 });
    const more = nav.locator('[data-nav-index="more"] > button');
    await expect(more).toBeVisible();
    // At this narrow width every item belongs in More. Wait for the final
    // measurement, not an intermediate count while text/resize events settle.
    await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
    await more.focus();
    await page.keyboard.press('ArrowDown');
    await expect(nav.locator('[data-nav-index="more"] a').first()).toBeFocused();
    await expect(nav.locator('[data-nav-index="more"] a').filter({ hasText: 'Guide' }).last()).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(more).toBeFocused();
    await page.setViewportSize({ width: 1600, height: 900 });
    await expect(more).toHaveCount(0);
    await expect(nav.getByRole('link', { name: 'Guide', exact: true })).toBeVisible();
    await expect(nav.locator('[data-zellij-navitem]:focus')).toHaveCount(1);
  });

  for (const direction of ['shrink', 'grow'] as const) {
    for (const destination of ['navigation', 'content', 'brand', 'pointer'] as const) {
      test(`preserves ${destination} focus when overflow ${direction} commits before queued focus`, async ({ page }) => {
        const initial = direction === 'shrink' ? 1600 : 901;
        await page.setViewportSize({ width: initial, height: 900 });
        await page.goto('/');
        await page.evaluate(() => document.fonts.ready);
        await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
        const nav = page.getByRole('navigation', { name: 'Main', exact: true });
        const more = nav.locator('[data-nav-index="more"] > button');
        const guide = nav.getByRole('link', { name: 'Guide', exact: true });
        const before = direction === 'shrink' ? guide : more;
        const after = direction === 'shrink' ? more : nav.locator('[data-zellij-navitem]').first();
        await expect(nav).toHaveAttribute('data-measured', 'true');
        if (direction === 'grow') {
          await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
        } else {
          await expect(more).toHaveCount(0);
        }
        await before.focus();
        await expect(before).toBeFocused();
        await holdApplicationFrames(page);
        await page.setViewportSize({ width: direction === 'shrink' ? 901 : 1600, height: 900 });
        await page.evaluate(() =>
          (window as unknown as { delayedFocusFrames: { advance(): Promise<void> } }).delayedFocusFrames.advance());
        await expect(before).toHaveCount(0);
        await expect(after).toBeVisible();
        const content = page.locator('main a[href]').first();
        const brand = page.locator('.zellij-logo');
        if (destination === 'content' || destination === 'brand') {
          const target = destination === 'content' ? content : brand;
          await target.focus();
          await expect(target).toBeFocused();
        } else if (destination === 'pointer') {
          const paragraph = page.locator('main').getByText('Lumen 3 Pro', { exact: true });
          await paragraph.evaluate(element => element.addEventListener('pointerdown', () => {
            element.setAttribute('data-pointer-received', 'true');
          }, { once: true }));
          const bounds = await paragraph.boundingBox();
          expect(bounds).not.toBeNull();
          await page.mouse.click(bounds!.x + bounds!.width / 2, bounds!.y + bounds!.height / 2);
          await expect(paragraph).toHaveAttribute('data-pointer-received', 'true');
        }
        await page.evaluate(() =>
          (window as unknown as { delayedFocusFrames: { release(): Promise<void> } }).delayedFocusFrames.release());
        if (destination === 'navigation') await expect(after).toBeFocused();
        else if (destination === 'content') await expect(content).toBeFocused();
        else if (destination === 'brand') await expect(brand).toBeFocused();
        else await expect(page.locator('body')).toBeFocused();
      });
    }
  }

  for (const width of [901, 1600]) {
    test(`restores overflow focus for the final ${width}px width after two queued resizes`, async ({ page }) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/');
      await page.evaluate(() => document.fonts.ready);
      await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
      const nav = page.getByRole('navigation', { name: 'Main', exact: true });
      const more = nav.locator('[data-nav-index="more"] > button');
      await expect(more).toHaveCount(width === 901 ? 1 : 0);
      if (width === 901) await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
      await (width === 901 ? more : nav.getByRole('link', { name: 'Guide', exact: true })).focus();
      await holdApplicationFrames(page);
      for (const next of [width === 901 ? 1600 : 901, width]) {
        await page.setViewportSize({ width: next, height: 900 });
        await page.evaluate(() =>
          (window as unknown as { delayedFocusFrames: { advance(): Promise<void> } }).delayedFocusFrames.advance());
        await expect(more).toHaveCount(next === 901 ? 1 : 0);
      }
      await page.evaluate(() =>
        (window as unknown as { delayedFocusFrames: { release(): Promise<void> } }).delayedFocusFrames.release());
      await expect(nav.locator('[data-zellij-navitem]:focus')).toHaveCount(1);
    });
  }

  test('reacts to larger text and retains nested destinations and external links', async ({ page }) => {
    await page.goto('/');
    await page.setViewportSize({ width: 1024, height: 480 });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 2.25rem; } .zellij-brand { width: 18rem; max-width: 25vw; }' });
    const nav = page.getByRole('navigation', { name: 'Main', exact: true });
    await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
    const more = nav.locator('[data-nav-index="more"] > button');
    await more.focus();
    await page.keyboard.press('ArrowDown');
    const panel = nav.locator('[data-nav-index="more"] .zellij-mega-panel');
    await expect(panel.getByRole('link', { name: 'Trade in', exact: true })).toBeVisible();
    await expect(panel.getByRole('link', { name: 'Camera' })).toBeVisible();
    await expect(panel.getByRole('link', { name: 'Source' })).toHaveAttribute('target', '_blank');
    await panel.getByRole('link', { name: 'Guide', exact: true }).focus();
    await expect(panel.getByRole('link', { name: 'Guide', exact: true })).toBeInViewport();
    await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  });

  test('keeps focus reachable when a focused item moves into overflow', async ({ page }) => {
    await page.setViewportSize({ width: 1600, height: 900 });
    await page.goto('/');
    const nav = page.getByRole('navigation', { name: 'Main', exact: true });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    await nav.getByRole('link', { name: 'Guide', exact: true }).focus();
    await page.setViewportSize({ width: 901, height: 900 });
    await expect(nav.locator('[data-nav-index="more"] > button')).toBeFocused();
  });

  test('keeps a flat More panel full width below the header', async ({ page }) => {
    await page.goto('/');
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    // Reserve enough centered space for the three non-leaf items, leaving the
    // final leaf links in a flat More panel. A hard-coded viewport instead
    // selects a mega panel when branding or utility widths change.
    const width = await page.locator('.zellij-nav-inner').evaluate(bar => {
      const probes = [...bar.querySelectorAll('.zellij-nav-measure > li')].map(item => item.getBoundingClientRect().width);
      const nav = bar.querySelector('.zellij-nav-measure')!;
      const gap = parseFloat(getComputedStyle(nav).columnGap) || 0;
      const actions = bar.querySelector('.zellij-nav-actions')!.getBoundingClientRect().width;
      const brand = bar.querySelector('.zellij-brand')!.getBoundingClientRect().width;
      const budget = probes.slice(0, 3).reduce((sum, value) => sum + value, 0) + 3 * gap + probes.at(-1)! + 8;
      return Math.ceil(2 * actions + 2 * 32 + 2 * 24 + brand + 24 + budget);
    });
    await page.setViewportSize({ width, height: 900 });
    const more = page.locator('[data-nav-index="more"]');
    await expect(more).toHaveAttribute('data-mega', 'false');
    await more.locator('button').focus();
    await page.keyboard.press('ArrowDown');
    const panel = more.locator('.zellij-dropdown');
    await expect(panel).toBeVisible();
    await expect.poll(() => panel.evaluate((element) => {
      const bounds = element.getBoundingClientRect();
      return Math.abs(bounds.left) < 1 && Math.abs(bounds.right - document.documentElement.clientWidth) < 1;
    })).toBe(true);
    await page.keyboard.press('Escape');
  });

  test('closes a mobile drawer and releases scroll lock when resized to desktop', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    await expect(page.getByRole('dialog', { name: 'Site navigation' })).toBeVisible();
    await page.setViewportSize({ width: 1280, height: 900 });
    await expect(page.locator('.zellij-drawer-root')).toHaveCount(0);
    await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
    await expect(page.locator('.zellij-nav-list [data-zellij-navitem]').first()).toBeFocused();
  });

  test('preserves a focused drawer link while the header reacts to scrolling', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    const header = page.locator('.zellij-header');
    await expect(header).toHaveAttribute('data-solid', 'false');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    const drawer = page.getByRole('dialog', { name: 'Site navigation' });
    await expect(drawer).toBeFocused();
    const link = drawer.getByRole('link').first();
    await link.focus();
    await expect(link).toBeFocused();
    // A programmatic scroll can update the header even while body scroll is
    // locked. Changing header state must not restart the open drawer's trap.
    await page.evaluate(() => window.scrollTo({ top: 400, behavior: 'instant' }));
    await expect(header).toHaveAttribute('data-solid', 'true');
    await expect(drawer).toBeVisible();
    await expect(link).toBeFocused();
    await page.keyboard.press('Escape');
    await expect(page.locator('.zellij-drawer-root')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
  });

  test('moves keyboard focus between visible controls across reciprocal resizes', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto('/');
    // The load event can precede header hydration, especially in WebKit.
    await expect(page.locator('.zellij-nav-desktop')).toHaveAttribute('data-measured', 'true');
    const first = page.locator('.zellij-nav-list [data-zellij-navitem]').first();
    await first.focus();
    await expect(first).toBeFocused();
    for (let cycle = 0; cycle < 3; cycle++) {
      await page.setViewportSize({ width: 390, height: 900 });
      await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
      await page.setViewportSize({ width: 1280, height: 900 });
      await expect(first).toBeFocused();
    }
  });

  test('does not take focus from document content on a breakpoint change', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto('/guide');
    await expect(page.locator('.zellij-nav-desktop')).toHaveAttribute('data-measured', 'true');
    const link = page.locator('main a[href]').first();
    await link.focus();
    for (const width of [390, 1280, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await expect(link).toBeFocused();
    }
  });

  test('keeps the final breakpoint in charge during rapid drawer resizes', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    const drawer = page.getByRole('dialog', { name: 'Site navigation' });
    await expect(drawer).toBeVisible();
    // Browsers can coalesce transient widths. Require actual DOM removal:
    // role queries omit a CSS-hidden drawer before React has closed it.
    await page.setViewportSize({ width: 1280, height: 900 });
    await expect(page.locator('.zellij-drawer-root')).toHaveCount(0);
    for (const width of [390, 1280, 390]) await page.setViewportSize({ width, height: 900 });
    await expect(page.locator('.zellij-drawer-root')).toHaveCount(0);
    await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
    await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
  });
  for (const destination of ['navigation', 'content', 'brand', 'pointer'] as const) {
    test(`preserves ${destination} focus across resizes before queued frames run`, async ({ page }) => {
      await page.setViewportSize({ width: 390, height: 900 });
      await page.goto('/');
      await page.getByRole('button', { name: 'Open navigation' }).click();
      const drawer = page.getByRole('dialog', { name: 'Site navigation' });
      await expect(drawer).toBeFocused();
      // Hold only animation callbacks after the drawer is ready. DOM commits and
      // real media-query events still run, modelling a delayed rendering frame.
      await holdApplicationFrames(page);
      const events = () => page.evaluate(() =>
        (window as unknown as { delayedFocusFrames: { events: number } }).delayedFocusFrames.events);
      await page.setViewportSize({ width: 1280, height: 900 });
      await expect(page.locator('.zellij-drawer-root')).toHaveCount(0);
      await expect.poll(events).toBe(1);
      await page.setViewportSize({ width: 390, height: 900 });
      await expect.poll(events).toBe(2);
      const contentLink = page.locator('main a[href]').first();
      const brand = page.locator('.zellij-logo');
      if (destination === 'content') {
        await contentLink.focus();
        await expect(contentLink).toBeFocused();
      } else if (destination === 'brand') {
        await brand.focus();
        await expect(brand).toBeFocused();
      } else if (destination === 'pointer') {
        // The page h1 is screen-reader-only. Click visible, non-focusable
        // content and prove the real pointer event arrived before releasing.
        const paragraph = page.locator('main').getByText('Lumen 3 Pro', { exact: true });
        await expect(paragraph).toBeInViewport();
        await paragraph.evaluate(element => element.addEventListener('pointerdown', () => {
          element.setAttribute('data-pointer-received', 'true');
        }, { once: true }));
        const bounds = await paragraph.boundingBox();
        expect(bounds).not.toBeNull();
        await page.mouse.click(bounds!.x + bounds!.width / 2, bounds!.y + bounds!.height / 2);
        await expect(paragraph).toHaveAttribute('data-pointer-received', 'true');
      }
      await page.evaluate(() =>
        (window as unknown as { delayedFocusFrames: { release(): Promise<void> } }).delayedFocusFrames.release());
      await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
      if (destination === 'navigation') {
        await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
      } else if (destination === 'content') {
        await expect(contentLink).toBeFocused();
      } else if (destination === 'brand') {
        await expect(brand).toBeFocused();
      } else {
        await expect(page.locator('body')).toBeFocused();
      }
    });
  }

});
