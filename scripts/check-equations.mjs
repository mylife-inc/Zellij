// Hosted browser acceptance: native export, relocated file navigation and Next.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { browserFailureState } from './browser-failure-state.mjs';
import path from 'node:path';
import { expect } from '@playwright/test';

export async function checkEquations(page, origin, captures, prefix) {
  const errors = [];
  const onError = error => errors.push(error.message);
  page.on('pageerror', onError);
  const url = relative => new URL(relative, origin).href;
  try {
    await page.goto(url('chapters/energy.html'));
    await expect(page.locator('[id="_zel-eq-eq:energy"] .zellij-equation-number')).toHaveText('(1.1)');
    await expect(page.locator('[id="_zel-eq-eq:system"] .zellij-equation-number')).toHaveText('(1.2)');
    assert.equal(await page.locator('.zellij-equation').count(), 2);
    assert(await page.locator('.zellij-equation').evaluateAll(nodes => nodes.every(node => node.querySelector('math, mjx-container'))));
    await page.getByRole('link', { name: '(2.1)', exact: true }).click();
    await expect(page).toHaveURL(/\/series\.html#_zel-eq-eq%3Asum$/);
    await expect(page.locator('[id="_zel-eq-eq:sum"] .zellij-equation-number')).toHaveText('(2.1)');
    await page.getByRole('link', { name: '(1.1)', exact: true }).click();
    await expect(page).toHaveURL(/\/chapters\/energy\.html#_zel-eq-eq%3Aenergy$/);
    await page.getByRole('link', { name: 'Link with custom text', exact: true }).click();
    await expect(page).toHaveURL(/\/series\.html#_zel-eq-eq(?::|%3A)sum$/);
    await page.getByRole('link', { name: '(1.1)', exact: true }).click();
    // Finish native fragment navigation before taking focus from it. This is
    // required with JavaScript disabled too; it is a browser navigation step.
    await expect(page).toHaveURL(/\/chapters\/energy\.html#_zel-eq-eq%3Aenergy$/);
    await page.waitForLoadState('load');
    await page.evaluate(() => document.fonts.ready);
    await expect(page.locator('[id="_zel-eq-eq:energy"]')).toBeInViewport();
    // A self-link is keyboard reachable and the shared stable target exists.
    const number = page.getByRole('link', { name: 'Equation 1.1', exact: true });
    await number.focus(); await expect(number).toBeFocused(); await page.keyboard.press('Enter');
    await expect(page.locator('[id="_zel-eq-eq:energy"]')).toBeInViewport();
    await page.goto(url('chapters/energy.html'));
    for (const width of [1280, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await page.evaluate(() => document.fonts.ready);
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
      await expect(page.locator('[id="_zel-eq-eq:energy"] .zellij-equation-number')).toBeVisible();
      await mkdir(captures, { recursive: true });
      await page.screenshot({ path: path.join(captures, `${prefix}-${width}.png`), fullPage: true });
    }
    await page.goto(url('notes.html'));
    await expect(page.locator('.zellij-equation-number')).toHaveText('(A.1)');
    await page.getByRole('link', { name: '(F1.1)', exact: true }).click();
    await expect(page.locator('.zellij-equation-number')).toHaveText('(F1.1)');
    assert.deepEqual(errors, []);
  } catch (error) {
    try {
      await mkdir(captures, { recursive: true });
      await writeFile(path.join(captures, `${prefix}-failure.json`), JSON.stringify({
        error: String(error), origin, prefix, errors,
        state: await browserFailureState(page, { sampleFrames: false }),
      }, null, 2));
      await page.screenshot({ path: path.join(captures, `${prefix}-failure.png`), fullPage: true, timeout: 3000 });
    } catch { /* Keep the original assertion if evidence cannot be captured. */ }
    throw error;
  } finally { page.off('pageerror', onError); }
}
