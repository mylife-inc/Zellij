// Hosted browser acceptance: native export, relocated file navigation and Next.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
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
    // A self-link is keyboard reachable and the shared stable target exists.
    const number = page.getByRole('link', { name: 'Equation 1.1', exact: true });
    await number.focus(); await expect(number).toBeFocused(); await page.keyboard.press('Enter');
    await expect(page.locator('[id="_zel-eq-eq:energy"]')).toBeInViewport();
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
  } finally { page.off('pageerror', onError); }
}
