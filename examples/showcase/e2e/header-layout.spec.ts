import { test, expect } from '@playwright/test';

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
});
