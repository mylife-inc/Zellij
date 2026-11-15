import { test, expect } from '@playwright/test';

// Run the same view/focus contract on Chromium and WebKit, including desktop
// keyboard contexts at phone widths. All browser execution belongs to CI.
test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/', { waitUntil: 'networkidle' });
});

const dialog = (page: import('@playwright/test').Page) => page.getByRole('dialog', { name: 'Site navigation', exact: true });
async function open(page: import('@playwright/test').Page) {
  const trigger = page.getByRole('button', { name: 'Open navigation', exact: true });
  await trigger.evaluate(node => node.focus({ preventScroll: true }));
  await page.keyboard.press('Enter');
  await expect(dialog(page)).toBeFocused();
  return dialog(page);
}

test('moves through section views, returns focus, closes and starts fresh', async ({ page }) => {
  const drawer = await open(page);
  await expect(drawer.getByRole('heading', { name: 'Menu', exact: true })).toBeVisible();
  await drawer.getByRole('button', { name: 'Shop', exact: true }).click();
  await expect(drawer.getByRole('heading', { name: 'Shop', exact: true })).toBeFocused();
  await expect(drawer.getByRole('button', { name: 'Explore', exact: true })).toHaveCount(0);
  await expect(drawer.getByRole('link', { name: 'Trade in', exact: true })).toBeVisible();
  await expect(drawer.getByRole('link', { name: 'GitHub repository' })).toHaveCount(0);
  await drawer.getByRole('button', { name: 'Back to main menu' }).click();
  await expect(drawer.getByRole('button', { name: 'Shop', exact: true })).toBeFocused();
  await drawer.getByRole('button', { name: 'Explore', exact: true }).click();
  await drawer.getByRole('button', { name: 'Close navigation', exact: true }).click();
  await expect(drawer).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Open navigation', exact: true })).toBeFocused();
  await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
  await open(page);
  await expect(drawer.getByRole('heading', { name: 'Menu', exact: true })).toBeVisible();
  await expect(drawer.getByRole('button', { name: 'Back to main menu' })).toHaveCount(0);
});

test('preserves grouped headings, descriptions and external new-tab navigation', async ({ page, context }) => {
  // Exercise the browser's real new-tab behavior without contacting GitHub.
  await context.route('https://github.com/**', route => route.fulfill({ contentType: 'text/html', body: '<title>External destination</title>' }));
  const drawer = await open(page);
  await drawer.getByRole('button', { name: 'Explore', exact: true }).click();
  await expect(drawer.getByRole('heading', { name: 'The product', exact: true })).toBeVisible();
  await expect(drawer.getByRole('heading', { name: 'More', exact: true })).toBeVisible();
  await expect(drawer.getByText('Grade 5 titanium, twelve grams lighter.', { exact: true })).toBeVisible();
  const source = drawer.getByRole('link', { name: /Source/ });
  await expect(source).toHaveAttribute('target', '_blank');
  await expect(source).toHaveAttribute('rel', 'noopener noreferrer');
  const [popup] = await Promise.all([context.waitForEvent('page'), source.click()]);
  await expect(popup).toHaveURL('https://github.com/shebka/zellij');
  await popup.close();
  await expect(drawer).toHaveCount(0);
});

test('direct links and section links navigate immediately and close the menu', async ({ page }) => {
  const drawer = await open(page);
  await drawer.getByRole('link', { name: 'Lumen 3 Pro', exact: true }).click();
  await expect(page).toHaveURL(/\/lumen-3-pro$/);
  await expect(drawer).toHaveCount(0);
  await open(page);
  await drawer.getByRole('button', { name: 'Shop', exact: true }).click();
  const trade = drawer.getByRole('link', { name: 'Trade in', exact: true });
  await expect(trade).not.toHaveAttribute('target', '_blank');
  await trade.click();
  await expect(page).toHaveURL(/\/lumen-3-pro#trade$/);
  await expect(drawer).toHaveCount(0);
});

test('Back restores the parent scroll position and focus in a short viewport', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 320 });
  const drawer = await open(page);
  const nav = drawer.locator('.zellij-drawer-nav');
  const explore = drawer.getByRole('button', { name: 'Explore', exact: true });
  await explore.scrollIntoViewIfNeeded();
  await explore.focus();
  const parentScroll = await nav.evaluate(node => node.scrollTop);
  expect(parentScroll).toBeGreaterThan(0);
  await page.keyboard.press('Enter');
  await expect(drawer.getByRole('heading', { name: 'Explore', exact: true })).toBeFocused();
  await expect.poll(() => nav.evaluate(node => node.scrollTop)).toBe(0);
  await nav.evaluate(node => { node.scrollTop = node.scrollHeight; });
  await drawer.getByRole('button', { name: 'Back to main menu' }).click();
  await expect(explore).toBeFocused();
  await expect.poll(() => nav.evaluate(node => node.scrollTop)).toBeCloseTo(parentScroll, 0);
  await page.keyboard.press('Enter');
  await expect(drawer.getByRole('heading', { name: 'Explore', exact: true })).toBeFocused();
  await expect.poll(() => nav.evaluate(node => node.scrollTop)).toBe(0);
});

test('Escape closes a section and preserves the document scroll position', async ({ page }) => {
  await page.evaluate(() => scrollTo(0, 400));
  const before = await page.evaluate(() => scrollY);
  const drawer = await open(page);
  await expect.poll(() => page.evaluate(() => scrollY)).toBeCloseTo(before, 0);
  await drawer.getByRole('button', { name: 'Shop', exact: true }).click();
  await expect.poll(() => page.evaluate(() => scrollY)).toBeCloseTo(before, 0);
  await page.keyboard.press('Escape');
  await expect(drawer).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Open navigation', exact: true })).toBeFocused();
  await expect.poll(() => page.evaluate(() => scrollY)).toBeCloseTo(before, 0);
});

test('resizing a section view to desktop releases the drawer and resets its level', async ({ page }) => {
  const drawer = await open(page);
  await drawer.getByRole('button', { name: 'Explore', exact: true }).click();
  await page.setViewportSize({ width: 1440, height: 900 });
  await expect(drawer).toHaveCount(0);
  await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
  await page.setViewportSize({ width: 390, height: 844 });
  await open(page);
  await expect(drawer.getByRole('heading', { name: 'Menu', exact: true })).toBeVisible();
});

test('keeps both levels usable in light/dark, phone/tablet and short layouts', async ({ page }, info) => {
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' });
    for (const width of [320, 390, 768]) {
      await page.setViewportSize({ width, height: width === 320 ? 340 : 844 });
      const drawer = await open(page);
      for (const section of [false, true]) {
        if (section) await drawer.getByRole('button', { name: 'Explore', exact: true }).click();
        const bounds = await drawer.boundingBox();
        expect(bounds!.x).toBeGreaterThanOrEqual(0);
        expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width + 1);
        const nav = drawer.locator('.zellij-drawer-nav');
        expect(await nav.evaluate(node => node.scrollWidth <= node.clientWidth)).toBe(true);
        for (const control of await drawer.locator('.zellij-drawer-control').all()) {
          await expect(control).toBeInViewport();
          const box = await control.boundingBox();
          expect(box!.width).toBeGreaterThanOrEqual(44);
          expect(box!.height).toBeGreaterThanOrEqual(44);
        }
        if (info.project.name === 'mobile' && width === 390) {
          await drawer.screenshot({ path: info.outputPath(`drawer-${colorScheme}-${section ? 'section' : 'root'}.png`) });
        }
      }
      await drawer.getByRole('button', { name: 'Close navigation', exact: true }).click();
    }
  }
});

test('respects reduced motion and motion none for opening and level changes', async ({ page }) => {
  for (const mode of ['reduce', 'none']) {
    await page.emulateMedia({ reducedMotion: mode === 'reduce' ? 'reduce' : 'no-preference' });
    if (mode === 'none') await page.locator('html').evaluate(node => node.setAttribute('data-zellij-motion', 'none'));
    const drawer = await open(page);
    await expect(drawer).toHaveCSS('animation-name', 'none');
    await drawer.getByRole('button', { name: 'Shop', exact: true }).click();
    await expect(drawer.locator('.zellij-drawer-view')).toHaveCSS('animation-name', 'none');
    await drawer.getByRole('button', { name: 'Back to main menu' }).click();
    await expect(drawer.locator('.zellij-drawer-view')).toHaveCSS('animation-name', 'none');
    await drawer.getByRole('button', { name: 'Close navigation', exact: true }).click();
  }
});

test('keeps section keyboard focus trapped and supports the tablet backdrop', async ({ page, browserName }) => {
  await page.setViewportSize({ width: 768, height: 844 });
  const drawer = await open(page);
  await drawer.getByRole('button', { name: 'Shop', exact: true }).click();
  if (browserName !== 'webkit') {
    await drawer.getByRole('button', { name: 'Back to main menu' }).focus();
    await page.keyboard.press('Shift+Tab');
    await expect(drawer.getByRole('link', { name: 'Compare models', exact: true })).toBeFocused();
    for (let i = 0; i < 20; i++) await page.keyboard.press('Tab');
    expect(await drawer.evaluate(node => node.contains(document.activeElement))).toBe(true);
  }
  await page.locator('.zellij-drawer-backdrop').click({ position: { x: 5, y: 5 } });
  await expect(drawer).toHaveCount(0);
});
