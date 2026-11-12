import { expect, test } from '@playwright/test';

const firstAlt = 'A single copper circle centered on a cream page';
const secondAlt = 'An open notebook with text on the left and a green illustration on the right';

test('media selection stays synchronized and fits wide and narrow containers', async ({ page, isMobile }, info) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/media-accordion');
  await expect(page.locator('main h1')).toHaveCount(1);
  const header = page.locator('[data-zellij-section="media-accordion"] [data-zellij-reveal]');
  await header.scrollIntoViewIfNeeded();
  await expect(header).toHaveAttribute('data-zellij-reveal', 'in');
  await expect(header).toHaveCSS('opacity', '1');
  const wide = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  const compact = page.getByRole('group', { name: 'A compact composition', exact: true });
  await expect(wide.getByRole('button', { name: 'Find your focus.', exact: true })).toHaveAttribute('aria-expanded', 'true');
  await expect(wide.locator('.zellij-media-accordion-panel:visible')).toHaveCount(1);
  await expect(wide.locator('.zellij-media-accordion-layout')).toHaveCSS('display', isMobile ? 'block' : 'grid');
  await expect(compact.locator('.zellij-media-accordion-layout')).toHaveCSS('display', 'block');
  await wide.getByRole('button', { name: 'Make room for ideas.', exact: true }).click();
  const illustration = wide.getByRole('img', { name: secondAlt, exact: true });
  await expect(illustration).toBeVisible();
  await expect.poll(() => illustration.evaluate((image: HTMLImageElement) => image.complete && image.naturalWidth > 0)).toBe(true);
  await expect(wide.getByRole('img', { name: firstAlt, exact: true })).toHaveCount(0);
  await expect(compact.getByRole('button', { name: 'A quiet beginning.', exact: true })).toHaveAttribute('aria-expanded', 'true');
  await compact.getByRole('button', { name: 'A wider perspective.', exact: true }).click();
  await expect(compact.getByRole('img', { name: 'Three shapes arranged on a wide page', exact: true })).toBeVisible();
  await expect(wide.getByRole('button', { name: 'Make room for ideas.', exact: true })).toHaveAttribute('aria-expanded', 'true');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.evaluate(() => { window.scrollTo({ top: 0, behavior: 'instant' }); });
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: info.outputPath('media-accordion-light.png'), fullPage: true, animations: 'disabled' });
  const light = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  await page.emulateMedia({ colorScheme: 'dark' });
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor)).not.toBe(light);
  await page.screenshot({ path: info.outputPath('media-accordion-dark.png'), fullPage: true, animations: 'disabled' });
  expect(errors).toEqual([]);
});

test('keyboard headers expose independent panel identities and preserve focus', async ({ page }) => {
  await page.goto('/media-accordion');
  const root = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  const buttons = root.getByRole('button');
  await buttons.first().focus();
  await page.keyboard.press('ArrowDown');
  await expect(buttons.nth(1)).toBeFocused();
  await expect(buttons.first()).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('Enter');
  await expect(buttons.nth(1)).toHaveAttribute('aria-expanded', 'true');
  await expect(buttons.nth(1)).toBeFocused();
  await page.keyboard.press('End');
  await expect(buttons.last()).toBeFocused();
  await page.keyboard.press('Space');
  await expect(buttons.last()).toHaveAttribute('aria-expanded', 'true');
  await expect(buttons.last()).toHaveAttribute('aria-disabled', 'true');
  await page.keyboard.press('Space');
  await expect(buttons.last()).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('Home');
  await page.keyboard.press('ArrowUp');
  await expect(buttons.last()).toBeFocused();
  const references = await page.locator('.zellij-media-accordion button').evaluateAll(nodes => nodes.map(node => ({ id: node.id, panel: node.getAttribute('aria-controls')! })));
  expect(new Set(references.flatMap(ref => [ref.id, ref.panel])).size).toBe(references.length * 2);
  for (const reference of references) await expect(page.locator(`[id="${reference.panel}"]`)).toHaveCount(1);
  await root.getByRole('link', { name: 'Continue to the guide' }).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/guide$/);
});

test('print restores all authored stories and images and returns to the selected screen state', async ({ page }) => {
  await page.goto('/media-accordion');
  const root = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  await root.getByRole('button', { name: 'Keep the whole picture.', exact: true }).click();
  await page.emulateMedia({ media: 'print' });
  await expect(page.locator('.zellij-media-accordion-panel:visible')).toHaveCount(5);
  await expect(page.locator('.zellij-media-accordion-inline img:visible')).toHaveCount(5);
  await expect.poll(() => page.locator('.zellij-media-accordion-inline img').evaluateAll(images => images.every(image => (image as HTMLImageElement).complete && (image as HTMLImageElement).naturalWidth > 0))).toBe(true);
  await expect(page.locator('.zellij-media-accordion-companion:visible')).toHaveCount(0);
  await expect(root).toHaveCSS('color', 'rgb(17, 17, 17)');
  await page.emulateMedia({ media: 'screen' });
  await expect(root.locator('.zellij-media-accordion-panel:visible')).toHaveCount(1);
  await expect(root.getByRole('button', { name: 'Keep the whole picture.', exact: true })).toHaveAttribute('aria-expanded', 'true');
});

test('reduced motion and forced colors retain readable controls without animation', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/media-accordion');
  const root = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  await root.getByRole('button', { name: 'Make room for ideas.', exact: true }).click();
  await expect(root.locator('.zellij-media-accordion-companion')).toHaveCSS('animation-name', 'none');
  await page.emulateMedia({ forcedColors: 'active' });
  const button = root.getByRole('button').first();
  await root.getByRole('button').nth(1).focus();
  await page.keyboard.press('Home');
  await expect(button).toBeFocused();
  await expect(button).toHaveCSS('outline-style', 'solid');
  await expect(root.locator('.zellij-media-accordion-item').first()).toHaveCSS('border-top-style', 'solid');
  await expect(root.getByRole('img', { name: secondAlt, exact: true })).toBeVisible();
});

test('resizing changes presentation without resetting selection', async ({ page }) => {
  await page.goto('/media-accordion');
  const root = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  await root.getByRole('button', { name: 'Make room for ideas.', exact: true }).click();
  const captionLink = root.getByRole('link', { name: 'Read the design notes', exact: true });
  await expect(captionLink).toHaveCount(1);
  await captionLink.focus();
  for (const width of [650, 1280, 390]) {
    await page.setViewportSize({ width, height: 900 });
    await expect(root.getByRole('button', { name: 'Make room for ideas.', exact: true })).toHaveAttribute('aria-expanded', 'true');
    await expect(root.getByRole('img', { name: secondAlt, exact: true })).toBeVisible();
    await expect(captionLink).toBeFocused();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  }
});

test('media side and fit settings use the same content without changing reading order', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('/media-accordion');
  const root = page.getByRole('group', { name: 'Fieldnotes details', exact: true });
  await expect(root.locator('.zellij-media-accordion-layout')).toHaveAttribute('data-ready', 'true');
  await root.locator('.zellij-media-accordion-layout').evaluate(element => element.setAttribute('data-media-side', 'start'));
  const media = root.locator('.zellij-media-accordion-companion');
  const text = root.locator('.zellij-media-accordion-items');
  expect((await media.boundingBox())!.x).toBeLessThan((await text.boundingBox())!.x);
  await expect(media.locator('img')).toHaveCSS('object-fit', 'contain');
  await media.locator('figure').evaluate(element => element.setAttribute('data-fit', 'cover'));
  await expect(media.locator('img')).toHaveCSS('object-fit', 'cover');
  await root.locator('.zellij-media-accordion-layout').evaluate(element => element.setAttribute('data-motion', 'none'));
  await expect(media).toHaveCSS('animation-name', 'none');
  expect(await root.locator('.zellij-media-accordion-layout > *').evaluateAll(nodes => nodes.map(node => node.className))).toEqual(['zellij-media-accordion-items', 'zellij-media-accordion-companion']);
});

test.describe('media accordion without JavaScript', () => {
  test.use({ javaScriptEnabled: false });
  test('retains every story, illustration and ordinary link in source order', async ({ page }) => {
    await page.goto('/media-accordion');
    await expect(page.locator('.zellij-media-accordion button')).toHaveCount(0);
    await expect(page.locator('.zellij-media-accordion-panel:visible')).toHaveCount(5);
    await expect(page.locator('.zellij-media-accordion-inline img:visible')).toHaveCount(5);
    await expect(page.getByRole('img', { name: secondAlt, exact: true })).toBeVisible();
    await page.emulateMedia({ media: 'print' });
    await expect(page.locator('.zellij-media-accordion-inline img:visible')).toHaveCount(5);
    await page.getByRole('link', { name: 'Continue to the guide' }).click();
    await expect(page).toHaveURL(/\/guide$/);
  });
});
