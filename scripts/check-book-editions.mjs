import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';

// Shared real-browser acceptance for native file:// and adapted Next pages.
export async function checkEdition(page, edition, captures, prefix) {
  const article = page.locator(`.zellij-book-${edition}`);
  await expect(article).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
  await mkdir(captures, { recursive: true });
  for (const width of [1280, 390]) {
    await page.setViewportSize({ width, height: 900 });
    for (const mode of ['light', 'dark']) {
      await page.evaluate(mode => document.documentElement.setAttribute('data-zellij-mode', mode), mode);
      const state = await article.evaluate(el => ({
        accent: getComputedStyle(el).getPropertyValue('--zellij-accent').trim(),
        font: getComputedStyle(el.querySelector('h1')).fontFamily,
        border: getComputedStyle(el).borderTopWidth,
        overflow: document.documentElement.scrollWidth > innerWidth + 1,
      }));
      assert.equal(state.overflow, false, `${prefix}: horizontal overflow at ${width}`);
      assert.equal(state.accent, edition === 'technical' ? (mode === 'light' ? '#234f86' : '#9ec5fb') : (mode === 'light' ? '#86462d' : '#edbba1'));
      assert.match(state.font, edition === 'technical' ? /outfit/i : /cormorant/i);
      assert.equal(state.border, edition === 'technical' ? '4px' : '1px');
      await page.screenshot({ path: path.join(captures, `${prefix}-${width}-${mode}.png`), fullPage: true });
    }
  }
  await page.emulateMedia({ forcedColors: 'active' });
  await expect(article.locator('h1')).toBeVisible();
  const link=article.locator('a').first();
  if(await link.count()) {await link.focus();await expect(link).toBeFocused();}
  await page.screenshot({ path: path.join(captures, `${prefix}-forced-colors.png`), fullPage: true });
  await page.emulateMedia({ forcedColors: 'none' });
  await page.evaluate(() => document.documentElement.setAttribute('data-zellij-mode', 'system'));
}
