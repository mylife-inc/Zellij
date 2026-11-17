import { expect, test } from '@playwright/test';

test('video loads only on request, keeps keyboard focus and exposes captions', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  const requests: string[] = [];
  page.on('request', request => { if (/compose-motion\.mp4/.test(request.url())) requests.push(request.url()); });
  await page.goto('/compose');
  const section = page.locator('#composed-media');
  await section.scrollIntoViewIfNeeded();
  expect(requests).toEqual([]);
  await expect(section.locator('video')).toHaveCount(0);
  const play = section.getByRole('button', { name: 'Play video: Three sheets in motion' });
  await play.focus();
  await page.keyboard.press('Enter');
  const video = section.locator('video');
  await expect(video).toBeFocused();
  await expect(video).toHaveAttribute('controls', '');
  await expect(video).toHaveAttribute('playsinline', '');
  await expect(video).toHaveAttribute('aria-label', 'Three sheets in motion');
  await expect.poll(() => video.evaluate((element: HTMLVideoElement) => element.readyState)).toBeGreaterThanOrEqual(2);
  await expect.poll(() => video.evaluate((element: HTMLVideoElement) => element.currentTime)).toBeGreaterThan(0);
  await expect(video.locator('track')).toHaveAttribute('srclang', 'en');
  await video.evaluate((element: HTMLVideoElement) => { element.textTracks[0]!.mode = 'showing'; });
  await expect.poll(() => video.evaluate((element: HTMLVideoElement) => element.textTracks[0]?.cues?.length ?? 0)).toBe(1);
  await section.getByRole('link', { name: 'Read transcript' }).click();
  await expect(page).toHaveURL(/#composed-media$/);
  expect(errors).toEqual([]);
});

test('nested grids respond to their own width and print keeps the video alternatives', async ({ page, isMobile }) => {
  await page.goto('/compose');
  const section = page.locator('#composed-media');
  const outer = section.locator(':scope > .zellij-container > .zellij-compose-layout');
  const inner = section.locator('[data-layout="grid"]');
  const columns = (element: Element) => getComputedStyle(element).gridTemplateColumns.split(' ').length;
  expect(await outer.evaluate(columns)).toBe(isMobile ? 1 : 2);
  expect(await inner.evaluate(columns)).toBe(1);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect(section.getByRole('button', { name: /Play video/ })).toBeVisible();
  await expect(section.locator('video')).toHaveCount(0);
  await page.emulateMedia({ media: 'print' });
  await expect(section.getByRole('button', { name: /Play video/ })).toBeHidden();
  await expect(section.getByRole('link', { name: 'Open video: Three sheets in motion' })).toBeVisible();
  await expect(section.getByRole('link', { name: 'Read transcript' })).toBeVisible();
  await expect(section.getByText('Animation transcript:', { exact: true })).toBeVisible();
});

test.describe('without JavaScript', () => {
  test.use({ javaScriptEnabled: false });
  test('self-hosted video has native controls and caption tracks instead of an inert play button', async ({ page }) => {
    await page.goto('/compose');
    const section = page.locator('#composed-media');
    await expect(section.getByRole('button', { name: /Play video/ })).toBeHidden();
    await expect(section.locator('video')).toBeVisible();
    await expect(section.locator('video')).toHaveAttribute('controls', '');
    await expect(section.locator('video')).toHaveAttribute('playsinline', '');
    await expect(section.locator('video')).toHaveAttribute('preload', 'none');
    await expect(section.locator('video')).not.toHaveAttribute('autoplay');
    await expect(section.locator('track')).toHaveAttribute('label', 'English');
    await section.getByRole('link', { name: 'Read transcript' }).click();
    await expect(page).toHaveURL(/#composed-media$/);
  });
});
