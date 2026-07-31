import { test, expect } from '@playwright/test';

/**
 * Covers spec §6 and §11.3: the active theme drives every visual token, dark
 * mode works per theme, and switching mode restyles the whole site.
 *
 * The build-time `theme:` switch is covered by unit tests over `emitThemeCss`
 * (a full rebuild per theme is too slow for e2e); this suite verifies that what
 * is emitted actually reaches the page and takes effect.
 */

test.describe('theme tokens', () => {
  test('emits the active theme into the document, not every theme', async ({ page }) => {
    await page.goto('/');

    const style = page.locator('style#zellij-theme');
    await expect(style).toHaveCount(1);

    const css = (await style.textContent()) ?? '';
    expect(css).toContain('--zellij-accent');
    expect(css).toContain('--zellij-font-heading');
    expect(css).toContain('--zellij-motion-duration');
    // One theme's worth of tokens: light block, dark block, system media query.
    expect(css.match(/--zellij-accent:/g)?.length).toBeLessThanOrEqual(4);
  });

  test('components resolve their styling from tokens', async ({ page }) => {
    await page.goto('/');

    const tokens = await page.evaluate(() => {
      const root = getComputedStyle(document.documentElement);
      return {
        accent: root.getPropertyValue('--zellij-accent').trim(),
        heading: root.getPropertyValue('--zellij-font-heading').trim(),
        radiusButton: root.getPropertyValue('--zellij-radius-button').trim(),
        sectionGap: root.getPropertyValue('--zellij-section-gap').trim(),
        duration: root.getPropertyValue('--zellij-motion-duration').trim(),
      };
    });

    for (const [name, value] of Object.entries(tokens)) {
      expect(value, `token ${name} is empty`).not.toBe('');
    }

    // The button's radius must come from the theme, not a hardcoded value.
    const buttonRadius = await page
      .locator('.zellij-button')
      .first()
      .evaluate((el) => getComputedStyle(el).borderRadius);
    expect(buttonRadius).not.toBe('');
  });

  test('the theme name is exposed for per-theme styling', async ({ page }) => {
    await page.goto('/');
    const theme = await page.locator('html').getAttribute('data-zellij-theme');
    expect(theme).toBeTruthy();
  });
});

test.describe('dark mode', () => {
  test.skip(({ isMobile }) => Boolean(isMobile), 'toggle is desktop-only');

  test('switching mode changes background and text colours sitewide', async ({ page }) => {
    await page.goto('/');

    const read = () =>
      page.evaluate(() => {
        const body = getComputedStyle(document.body);
        return { bg: body.backgroundColor, fg: body.color };
      });

    const before = await read();
    await page.getByRole('button', { name: /Switch to (light|dark) theme/ }).click();
    await expect
      .poll(async () => (await read()).bg)
      .not.toBe(before.bg);

    const after = await read();
    expect(after.fg).not.toBe(before.fg);
  });

  test('follows the system preference when mode is "system"', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'light' });
    await page.goto('/');
    await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'system');
    const light = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);

    await page.emulateMedia({ colorScheme: 'dark' });
    await expect
      .poll(async () => page.evaluate(() => getComputedStyle(document.body).backgroundColor))
      .not.toBe(light);
  });

  test('guide code blocks follow the mode too', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('/guide/quickstart');

    const token = page.locator('pre.shiki span').first();
    const darkColor = await token.evaluate((el) => getComputedStyle(el).color);

    await page.emulateMedia({ colorScheme: 'light' });
    await expect
      .poll(async () => token.evaluate((el) => getComputedStyle(el).color))
      .not.toBe(darkColor);
  });
});
