// Hosted acceptance shared by relocated file:// books and prefixed Next images.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';
import { browserFailureState } from './browser-failure-state.mjs';
export async function checkBibliography(page, origin, captures, prefix) {
  const errors = []; const onError = error => errors.push(error.message);
  page.on('pageerror', onError);
  const url = relative => new URL(relative, origin).href;
  try {
    await page.goto(url('chapters/energy.html'));
    const citation = page.locator('article a[title="Reference 2: Finite sums and careful arguments"]').first();
    await expect(citation).toHaveText('2');
    await expect(page.locator('article')).toContainText('[2, pp. 12–14; 1, chapter 2]');
    await expect(page.locator('article')).toContainText('[@not-a-citation]');
    await citation.click();
    await expect(page).toHaveURL(/\/bibliography\.html#_zel-bib-river2021$/);
    await expect(page.locator('#_zel-bib-river2021')).toBeInViewport();
    await expect(page.locator('.zellij-bibliography > li')).toHaveCount(4);
    await expect(page.locator('.zellij-bibliography > li').first()).toContainText('[1] Mira Vale. (2024). An atlas of quiet ideas.');
    await expect(page.locator('#_zel-bib-river2021 a')).toHaveAttribute('href', 'https://doi.org/10.1234/example.sum');
    await page.goto(url('bibliography.html'));
    await page.waitForLoadState('load');
    await page.evaluate(() => document.fonts.ready);
    const online = page.locator('#_zel-bib-vale2024 a');
    await online.focus(); await expect(online).toBeFocused();
    // Assert the destination without opening an external service.
    await expect(online).toHaveAttribute('href', 'https://example.com/atlas');
    for (const width of [1280, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await page.evaluate(() => document.fonts.ready);
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), 'No bibliography overflow');
      await mkdir(captures, { recursive: true });
      await page.screenshot({ path: path.join(captures, `${prefix}-${width}.png`), fullPage: true });
    }
    await page.goto(url('series.html'));
    await page.locator('article').getByRole('link', { name: 'bibliography', exact: true }).click();
    await expect(page.locator('#_zel-bib-river2021')).toBeInViewport();
    assert.deepEqual(errors, []);
  } catch (error) {
    try {
      await mkdir(captures, { recursive: true });
      await writeFile(path.join(captures, `${prefix}-failure.json`), JSON.stringify({ error: String(error), origin, errors, state: await browserFailureState(page, { sampleFrames: false }) }, null, 2));
      await page.screenshot({ path: path.join(captures, `${prefix}-failure.png`), fullPage: true, timeout: 3000 });
    } catch { /* Retain the original assertion. */ }
    throw error;
  } finally { page.off('pageerror', onError); }
}
