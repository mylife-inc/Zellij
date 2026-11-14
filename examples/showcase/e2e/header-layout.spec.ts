import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';

test('branding and controls sit at opposite viewport edges without overlap', async ({ page }) => {
  await page.goto('/');
  for (const width of [1600, 1280, 1024, 901, 900, 390, 360, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await expect.poll(async () => page.locator('.zellij-nav-inner').evaluate((bar) => {
      const brand = bar.querySelector('.zellij-brand')!.getBoundingClientRect();
      const actions = bar.querySelector('.zellij-nav-actions')!.getBoundingClientRect();
      const width = document.documentElement.clientWidth;
      const links = [...bar.querySelectorAll('.zellij-nav-list:not(.zellij-nav-measure) > li > .zellij-nav-link')]
        .filter((link) => link.getClientRects().length).map((link) => link.getBoundingClientRect());
      return links.every((link) => link.left >= brand.right && link.right <= actions.left) && brand.left >= 15 && brand.left <= 33 && width - actions.right >= 15 &&
        width - actions.right <= 33 && brand.right <= actions.left && document.documentElement.scrollWidth <= width;
    })).toBe(true);
  }
});

test.describe('measured overflow', () => {
  test.skip(({ isMobile }) => Boolean(isMobile), 'desktop keyboard and resize behavior');

  test('moves trailing links into More and restores them when space returns', async ({ page }) => {
    await page.goto('/');
    const nav = page.getByRole('navigation', { name: 'Main', exact: true });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    await page.setViewportSize({ width: 901, height: 900 });
    const more = nav.locator('[data-nav-index="more"] > button');
    await expect(more).toBeVisible();
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

  test('reacts to larger text and retains nested destinations and external links', async ({ page }) => {
    await page.goto('/');
    await page.setViewportSize({ width: 1024, height: 480 });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 2.25rem; } .zellij-brand { width: 18rem; max-width: none; flex-shrink: 0; }' });
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

  test('keeps a compact More panel inside either viewport edge', async ({ page }) => {
    await page.goto('/');
    await page.setViewportSize({ width: 960, height: 900 });
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    const more = page.locator('[data-nav-index="more"]');
    await expect(more).toBeVisible();
    // Exercise both anchor extremes independently of the fixture's short labels.
    for (const edge of ['left', 'right']) {
      await more.evaluate((item, edge) => {
        item.setAttribute('style', `position: absolute; top: 0; ${edge}: 1rem;`);
      }, edge);
      await more.locator('button').focus();
      await page.keyboard.press('ArrowDown');
      const panel = more.locator('.zellij-dropdown');
      await expect(panel).toBeVisible();
      await expect.poll(() => panel.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        return bounds.left >= 15 && bounds.right <= document.documentElement.clientWidth - 15;
      })).toBe(true);
      await page.keyboard.press('Escape');
    }
  });

  test('closes a mobile drawer and releases scroll lock when resized to desktop', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    await expect(page.getByRole('dialog', { name: 'Site navigation' })).toBeVisible();
    await page.setViewportSize({ width: 1280, height: 900 });
    await expect(page.getByRole('dialog', { name: 'Site navigation' })).toHaveCount(0);
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
    await expect(drawer).toHaveCount(0);
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

  test('keeps the final breakpoint in charge during rapid drawer resizes', async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    const entries: unknown[] = [];
    (window as any).__focusProbe = entries;
    const identify = (el: Element | null) => el && ({ tag: el.tagName, class: el.className, label: el.getAttribute('aria-label'), text: el.textContent?.slice(0, 50), connected: el.isConnected, rects: el.getClientRects().length });
    const record = (event: string, target?: Element | null, stack?: string) => entries.push({
      at: performance.now(), event, active: identify(document.activeElement), target: identify(target ?? null), stack,
      width: innerWidth, mobile: matchMedia('(max-width: 900px)').matches, drawer: !!document.querySelector('.zellij-drawer-root'),
    });
    window.addEventListener('resize', () => record('resize'));
    const focus = HTMLElement.prototype.focus;
    HTMLElement.prototype.focus = function (...args) {
      record('focus-call-before', this, new Error().stack);
      focus.apply(this, args);
      record('focus-call-after', this);
    };
    document.addEventListener('focusin', event => record('focusin', event.target as Element), true);
    document.addEventListener('focusout', event => record('focusout', event.target as Element), true);
    matchMedia('(max-width: 900px)').addEventListener('change', () => {
      record('media-before-react');
      requestAnimationFrame(() => {
        record('media-frame-1');
        requestAnimationFrame(() => record('media-frame-2'));
      });
    });
  });
    let assertionsPassed = false;
    try {
      await page.setViewportSize({ width: 390, height: 900 });
      await page.goto('/');
      await page.getByRole('button', { name: 'Open navigation' }).click();
      const drawer = page.getByRole('dialog', { name: 'Site navigation' });
      await expect(drawer).toBeVisible();
      // A browser may coalesce transient widths without dispatching a desktop
      // media-query change. Establish that first transition before the burst.
      await page.setViewportSize({ width: 1280, height: 900 });
      await expect(drawer).toHaveCount(0);
      for (const width of [390, 1280, 390]) await page.setViewportSize({ width, height: 900 });
      await expect(drawer).toHaveCount(0);
      await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
      await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
      assertionsPassed = true;
    } finally {
      const entries = await page.evaluate(() => (window as any).__focusProbe);
      mkdirSync('focus-probe-results', { recursive: true });
      const result = { project: testInfo.project.name, repetition: testInfo.repeatEachIndex,
        assertionsPassed, events: entries };
      writeFileSync(`focus-probe-results/${testInfo.project.name}-${testInfo.repeatEachIndex}-${testInfo.workerIndex}.json`, JSON.stringify(result, null, 2));
      await testInfo.attach('focus-events', { body: JSON.stringify(result, null, 2), contentType: 'application/json' });
    }
  });
});
