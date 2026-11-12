import { expect, test } from '@playwright/test';

test('compositions retain source order, responsive layouts, links and images', async ({ page, isMobile }, info) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/compose');
  await expect(page.locator('main h1')).toHaveCount(1);
  await expect(page.getByRole('heading', { name: 'Clarity, composed.', level: 1 })).toBeVisible();
  const types = await page.locator('[data-zellij-section]').evaluateAll(nodes => nodes.map(node => node.getAttribute('data-zellij-section')));
  expect(types).toEqual(['compose', 'compose', 'compose']);
  await page.getByRole('link', { name: 'Explore the composition' }).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#composed-story$/);
  const illustration = page.getByAltText('Three folded sheets arranged around a copper circle');
  await expect.poll(() => illustration.evaluate((image: HTMLImageElement) => image.complete && image.naturalWidth > 0)).toBe(true);
  const split = page.locator('#composed-story > .zellij-container > .zellij-compose-layout');
  const columns = await split.evaluate(element => getComputedStyle(element).gridTemplateColumns.split(' ').length);
  expect(columns).toBe(isMobile ? 1 : 2);
  await page.getByRole('link', { name: 'See the details' }).click();
  await expect(page).toHaveURL(/#composed-details$/);
  const cards = page.locator('#composed-details [data-layout="grid"]');
  const cardColumns = await cards.evaluate(element => getComputedStyle(element).gridTemplateColumns.split(' ').length);
  expect(cardColumns).toBe(isMobile ? 1 : 3);
  const entrance = page.locator('#composed-details [data-zellij-reveal]');
  await entrance.scrollIntoViewIfNeeded();
  await expect(entrance).toHaveAttribute('data-zellij-reveal', 'in');
  if (!isMobile) {
    const surfaces = cards.locator(':scope > .zellij-compose-group, :scope > .zellij-compose-reveal > .zellij-compose-group');
    await expect(surfaces).toHaveCount(3);
    await expect.poll(() => surfaces.evaluateAll(nodes => {
      const heights = nodes.map(node => node.getBoundingClientRect().height);
      return Math.max(...heights) - Math.min(...heights);
    })).toBeLessThan(2);
    await expect.poll(() => surfaces.evaluateAll(nodes => {
      const tops = nodes.map(node => node.querySelector('.zellij-compose-heading')!.getBoundingClientRect().top);
      return Math.max(...tops) - Math.min(...tops);
    })).toBeLessThan(2);
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.evaluate(() => window.scrollTo({ top: 0, behavior: 'instant' }));
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: info.outputPath('composition-light.png'), fullPage: true, animations: 'disabled' });
  await info.attach('composition-page', { path: info.outputPath('composition-light.png'), contentType: 'image/png' });
  const light = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  await page.emulateMedia({ colorScheme: 'dark' });
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor)).not.toBe(light);
  await page.screenshot({ path: info.outputPath('composition-dark.png'), fullPage: true, animations: 'disabled' });
  await info.attach('composition-dark', { path: info.outputPath('composition-dark.png'), contentType: 'image/png' });
  expect(errors).toEqual([]);
});

test('gradient reveal follows visibility while its solid text remains readable', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('/compose');
  const heading = page.locator('#composed-story .zellij-gradient-reveal');
  const paint = heading.locator('.zellij-gradient-reveal-paint');
  await expect(paint).toHaveCSS('opacity', '0');
  await heading.scrollIntoViewIfNeeded();
  await expect(paint).toHaveCSS('opacity', '1');
  await expect(heading.locator('span').first()).toBeVisible();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.evaluate(() => window.scrollTo({ top: 0, behavior: 'instant' }));
  await expect(paint).toHaveCSS('opacity', '1');
});

test('forced colors and print keep a solid heading and all composed text', async ({ page }) => {
  await page.goto('/compose');
  await page.emulateMedia({ forcedColors: 'active' });
  await expect(page.locator('.zellij-gradient-reveal-paint').first()).toHaveCSS('display', 'none');
  await expect(page.locator('main h1')).toBeVisible();
  await page.emulateMedia({ forcedColors: 'none', media: 'print' });
  await expect(page.locator('.zellij-gradient-reveal-paint').first()).toHaveCSS('display', 'none');
  await expect(page.locator('#composed-details [data-zellij-reveal]')).toHaveCSS('opacity', '1');
  await expect(page.getByRole('link', { name: 'Explore the composition' })).toHaveCSS('color', 'rgb(17, 17, 17)');
  for (const text of ['A natural rhythm.', 'Your point of view.', 'Every reader counts.']) {
    await expect(page.getByRole('heading', { name: text })).toBeVisible();
  }
});

test('missing IntersectionObserver leaves a complete gradient heading', async ({ page }) => {
  await page.addInitScript(() => { Object.defineProperty(window, 'IntersectionObserver', { value: undefined }); });
  await page.goto('/compose');
  await expect(page.locator('.zellij-gradient-reveal-paint').first()).toHaveCSS('opacity', '1');
  await expect(page.locator('main h1')).toBeVisible();
});

test.describe('composition without JavaScript', () => {
  test.use({ javaScriptEnabled: false });
  test('retains all text, images and ordinary navigation', async ({ page }) => {
    await page.goto('/compose');
    await expect(page.locator('main h1')).toBeVisible();
    await expect(page.locator('.zellij-gradient-reveal-paint').first()).toHaveCSS('opacity', '1');
    await page.getByRole('link', { name: 'Explore the composition' }).click();
    await expect(page).toHaveURL(/#composed-story$/);
    await expect(page.getByAltText('Three folded sheets arranged around a copper circle')).toBeVisible();
    await expect(page.getByText('Every reader counts.')).toBeVisible();
  });
});

test('nested surfaces retain their own foreground in light, dark and print', async ({ page }) => {
  await page.goto('/compose');
  const inverse = page.locator('#composed-details .zellij-compose-group[data-surface="inverted"]');
  const inner = inverse.locator('.zellij-compose-group[data-surface="tinted"]');
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme });
    const foreground = await page.evaluate(() => getComputedStyle(document.body).color);
    await expect(inner).toHaveCSS('color', foreground);
    // Some themes intentionally use the same foreground for both dark surfaces.
    // Verify the selected palette, rather than assuming inversion changes text.
    const palette = await inverse.evaluate(element => {
      const probe = document.createElement('span');
      probe.style.color = 'var(--zellij-inverted-fg)';
      probe.style.backgroundColor = 'var(--zellij-inverted-bg)';
      element.append(probe);
      const style = getComputedStyle(probe);
      const result = { foreground: style.color, background: style.backgroundColor };
      probe.remove();
      return result;
    });
    await expect(inverse).toHaveCSS('color', palette.foreground);
    await expect(inverse).toHaveCSS('background-color', palette.background);
    await expect(inverse.locator('.zellij-compose-heading')).toHaveCSS('color', await inverse.evaluate(element => getComputedStyle(element).color));
  }
  await page.locator('#composed-details').evaluate(element => element.setAttribute('data-zellij-background', 'inverted'));
  await page.emulateMedia({ media: 'print' });
  await expect(page.locator('#composed-details > .zellij-container > .zellij-compose-layout > h2')).toHaveCSS('color', 'rgb(17, 17, 17)');
  await expect(inner).toHaveCSS('color', 'rgb(17, 17, 17)');
});
