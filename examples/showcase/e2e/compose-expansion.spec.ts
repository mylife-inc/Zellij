import { expect, test } from '@playwright/test';

for (const route of ['compose-layers', 'compose-content']) {
  test(`${route} retains readable layouts, loaded media and light/dark presentation`, async ({ page, isMobile }, info) => {
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(`/${route}`);
    await expect(page.locator('main h1')).toHaveCount(1);
    await page.evaluate(() => document.fonts.ready);
    for (const img of await page.locator('main img').all()) {
      await img.scrollIntoViewIfNeeded();
      await expect.poll(() => img.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
    if (route === 'compose-layers') {
      const layout = page.locator('#layered-study [data-layout="layers"]');
      const places = await layout.locator(':scope > *').evaluateAll(nodes => nodes.map(node => ({ top: node.getBoundingClientRect().top, left: node.getBoundingClientRect().left })));
      if (isMobile) { expect(places[1]!.top).toBeGreaterThan(places[0]!.top); expect(places[2]!.top).toBeGreaterThan(places[1]!.top); }
      else { expect(places[1]!.left).toBeGreaterThan(places[0]!.left); expect(places[2]!.top).toBeGreaterThan(places[1]!.top); }
      await page.getByRole('link', {name: 'Explore the elements'}).focus();
      await expect(page.getByRole('link', {name: 'Explore the elements'})).toBeFocused();
    }
    for (const colorScheme of ['light', 'dark'] as const) {
      await page.emulateMedia({colorScheme});
      await page.evaluate(() => scrollTo({top: 0, behavior: 'instant'}));
      await page.screenshot({path: info.outputPath(`composition-${route}-${colorScheme}.png`), fullPage: true, animations: 'disabled'});
    }
    expect(errors).toEqual([]);
  });
}

test('backdrop video is opt-in, keeps its button focused and pauses for reduced motion', async ({ page }) => {
  await page.goto('/compose-layers');
  const section = page.locator('#layered-video');
  const video = section.locator('video');
  expect(await video.evaluate((el: HTMLVideoElement) => el.paused)).toBe(true);
  await expect(video).toHaveAttribute('preload', 'none');
  const play = section.getByRole('button', {name: 'Play background video: Three sheets in motion'});
  await play.focus(); await page.keyboard.press('Enter');
  const pause = section.getByRole('button', {name: 'Pause background video: Three sheets in motion'});
  await expect(pause).toBeFocused();
  await expect.poll(() => video.evaluate((el: HTMLVideoElement) => el.currentTime)).toBeGreaterThan(0);
  await page.emulateMedia({reducedMotion: 'reduce'});
  await expect(play).toBeFocused();
  expect(await video.evaluate((el: HTMLVideoElement) => el.paused)).toBe(true);
});

test('semantic nodes retain keyboard operation, independent dialogs and print content', async ({ page }) => {
  await page.goto('/compose-content');
  const table = page.getByRole('table', {name: 'One story, every reading mode'});
  await expect(table.locator('th[scope="row"]')).toHaveCount(3);
  const summary = page.locator('.zellij-compose-disclosure summary');
  await summary.focus(); await page.keyboard.press('Enter');
  await expect(page.locator('.zellij-compose-disclosure')).toHaveAttribute('open', '');
  await page.keyboard.press('Enter');
  await expect(page.locator('.zellij-compose-disclosure')).not.toHaveAttribute('open');
  const first = page.getByRole('link', {name: 'View First paper study full size'});
  await first.focus(); await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog', {name: 'Paper studies'});
  await expect(dialog).toBeVisible();
  await page.keyboard.press('ArrowRight');
  await expect(dialog.locator('.zellij-lightbox-count')).toHaveText('2 of 2');
  await page.keyboard.press('Escape'); await expect(first).toBeFocused();
  const independent = page.getByRole('link', {name: 'View Independent paper study full size'});
  await independent.click();
  await expect(page.getByRole('dialog', {name: 'An independent gallery'})).toBeVisible();
  await expect(dialog).not.toBeVisible();
  await page.keyboard.press('Escape'); await expect(independent).toBeFocused();
  const track = page.getByRole('group', {name: 'Three principles'});
  await page.emulateMedia({reducedMotion: 'reduce'});
  const next = page.locator('.zellij-compose-carousel').getByRole('button', {name: 'Scroll right'});
  await next.focus(); await page.keyboard.press('Enter');
  await expect.poll(() => track.evaluate(el => el.scrollLeft)).toBeGreaterThan(0);
  await page.emulateMedia({media: 'print'});
  await expect(page.locator('.zellij-compose-disclosure > div')).toBeVisible();
  await expect(track).toHaveCSS('overflow-x', 'visible');
  await expect(page.getByRole('heading', {name: 'Care', exact: true})).toBeVisible();
});

test('scroll stages follow authored ranges and reverse, with focus and size fallbacks', async ({ page, isMobile }, info) => {
  await page.emulateMedia({reducedMotion: 'no-preference'});
  await page.goto('/compose-scroll');
  const stage = page.locator('#composed-scroll .zellij-compose-scroll');
  const content = stage.locator(':scope > .zellij-compose-scroll-content');
  if (isMobile) {
    await expect(stage).toHaveAttribute('data-scroll-active', 'false');
    await expect(content).toHaveCSS('transform', 'none');
  } else {
    await expect(stage).toHaveAttribute('data-scroll-active', 'true');
    const go = async (progress: number) => {
      await stage.evaluate((node, p) => {
        const inner = node.firstElementChild!;
        const top = parseFloat(getComputedStyle(inner).top);
        scrollTo({top: scrollY + node.getBoundingClientRect().top - top + innerHeight * p, behavior: 'instant'});
      }, progress);
    };
    await go(0.1); await expect(content).toHaveCSS('opacity', '0.6');
    await go(0.5);
    await expect.poll(async () => Number(await content.evaluate(el => getComputedStyle(el).opacity))).toBeGreaterThan(0.7);
    await page.screenshot({path: info.outputPath('composition-scroll-mid.png'), animations: 'disabled'});
    await go(0.9); await expect(content).toHaveCSS('opacity', '1');
    await go(0.1); await expect(content).toHaveCSS('opacity', '0.6');
    await page.getByRole('link', {name: 'Read without waiting'}).focus();
    await expect(content).toHaveCSS('transform', 'none');
    await expect(content).toHaveCSS('opacity', '1');
    await content.evaluate(node => { const p = document.createElement('p'); p.textContent = 'Long reading content. '.repeat(900); node.append(p); });
    await expect(stage).toHaveAttribute('data-scroll-active', 'false');
    await expect(content).toHaveCSS('position', 'static');
  }
  await page.emulateMedia({reducedMotion: 'reduce'});
  await expect(stage).toHaveAttribute('data-scroll-active', 'false');
  await page.emulateMedia({media: 'print'});
  await expect(content).toHaveCSS('transform', 'none');
});

test.describe('without JavaScript', () => {
  test.use({javaScriptEnabled: false});
  test('layers, semantic controls and scroll stages remain usable', async ({ page }) => {
    await page.goto('/compose-layers');
    await expect(page.locator('main h1')).toBeVisible();
    await expect(page.getByRole('button', {name: /Play background/})).toBeHidden();
    await page.getByRole('link', {name: 'Explore the elements'}).click();
    await expect(page).toHaveURL(/compose-content$/);
    await page.locator('.zellij-compose-disclosure summary').click();
    await expect(page.locator('.zellij-compose-disclosure > div')).toBeVisible();
    const image = page.getByRole('link', {name: 'View First paper study full size'});
    await expect(image).toHaveAttribute('href', /_zellij\/assets\/compose-study\.svg/);
    await image.click(); await expect(page).toHaveURL(/compose-study\.svg/);
    await page.goto('/compose-scroll');
    await expect(page.locator('.zellij-compose-scroll').first()).toHaveAttribute('data-scroll-active', 'false');
    await expect(page.locator('.zellij-compose-scroll-content').first()).toHaveCSS('transform', 'none');
  });
});


test('unpinned stages keep normal flow and disable after resizing or forced colors', async ({ page, isMobile }) => {
  await page.goto('/compose-scroll');
  const stage = page.locator('#composed-scroll-flow .zellij-compose-scroll');
  const content = stage.locator(':scope > .zellij-compose-scroll-content');
  await expect(content).toHaveCSS('position', 'static');
  if (!isMobile) {
    await expect(stage).toHaveAttribute('data-scroll-active', 'true');
    await stage.scrollIntoViewIfNeeded();
    expect(await stage.evaluate(el => el.style.height)).toBe('');
    await page.setViewportSize({width: 600, height: 800});
    await expect(stage).toHaveAttribute('data-scroll-active', 'false');
    await expect(content).toHaveCSS('transform', 'none');
    await page.setViewportSize({width: 1280, height: 800});
    await expect(stage).toHaveAttribute('data-scroll-active', 'true');
  }
  await page.emulateMedia({forcedColors: 'active'});
  await expect(content).toHaveCSS('transform', 'none');
  await expect(content).toHaveCSS('opacity', '1');
});

test('missing intersection observers leave stages in ordinary readable flow', async ({ page }) => {
  await page.addInitScript(() => { Object.defineProperty(window, 'IntersectionObserver', {value: undefined}); });
  await page.goto('/compose-scroll');
  for (const stage of await page.locator('.zellij-compose-scroll').all()) {
    await expect(stage).toHaveAttribute('data-scroll-active', 'false');
    await expect(stage.locator(':scope > div')).toHaveCSS('position', 'static');
  }
  await expect(page.locator('main h1')).toBeVisible();
});
