// Shared hosted acceptance for relocated file:// books and prefixed Next images.
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';

export async function checkStatements(page, origin, captures, prefix) {
  const errors = [];
  const onError = error => errors.push(error.message);
  page.on('pageerror', onError);
  const url = relative => new URL(relative, origin).href;
  const block = id => page.locator(`[id="_zel-stmt-${id}"]`);
  try {
    await page.goto(url('chapters/energy.html'));
    await expect(block('thm:energy').locator('.zellij-statement-head')).toHaveText('Theorem 1.3 (Energy & equality)');
    await expect(block('lem:square').locator('.zellij-statement-head')).toHaveText('Lemma 1.2 (A square is nonnegative)');
    await expect(block('cor:zero').locator('.zellij-statement-head')).toHaveText('Corollary 1.4 (Zero speed)');
    await expect(block('proof:energy').locator('.zellij-statement-head')).toHaveText('Proof (Of the energy theorem)');
    await expect(block('proof:energy').locator('.zellij-statement-qed')).toHaveText('∎');
    assert.equal(await block('proof:energy').locator('.zellij-statement-qed').getAttribute('aria-hidden'), 'true');
    assert.equal(await block('thm:energy').locator('li').count(), 2);
    await expect(block('thm:energy').locator('blockquote')).toContainText('Equality is a special case.');
    await expect(block('thm:energy').locator('pre')).toContainText('energy = mass * speed * speed');
    assert(await block('lem:square').locator('math, mjx-container').count() > 0);
    assert(!(await page.locator('article').innerText()).includes('[!THEOREM]'));
    await page.getByRole('link', { name: 'Proposition 2.1', exact: true }).click();
    await expect(page).toHaveURL(/\/series\.html#_zel-stmt-prop%3Asum$/);
    await expect(block('prop:sum')).toBeInViewport();
    await expect(block('ex:sum').locator('table')).toContainText('First');
    await page.getByRole('link', { name: 'Theorem 1.3', exact: true }).click();
    await expect(page).toHaveURL(/\/chapters\/energy\.html#_zel-stmt-thm%3Aenergy$/);
    await page.getByRole('link', { name: 'custom statement link', exact: true }).click();
    await expect(page).toHaveURL(/\/series\.html#_zel-stmt-prop(?::|%3A)sum$/);
    await page.getByRole('link', { name: 'Theorem 1.3', exact: true }).click();
    const proof = page.getByRole('link', { name: 'Proof', exact: true });
    await proof.focus(); await expect(proof).toBeFocused(); await page.keyboard.press('Enter');
    await expect(block('proof:energy')).toBeInViewport();
    for (const width of [1280, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await page.evaluate(() => document.fonts.ready);
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
      await mkdir(captures, { recursive: true });
      await page.screenshot({ path: path.join(captures, `${prefix}-${width}.png`), fullPage: true });
    }
    await page.goto(url('notes.html'));
    await expect(block('remark:notation').locator('.zellij-statement-head')).toHaveText('Remark A.1 (Notation)');
    await page.getByRole('link', { name: 'Definition F1.1', exact: true }).click();
    await expect(block('def:scope').locator('.zellij-statement-head')).toHaveText('Definition F1.1 (Scope)');
    await page.goto(url(new URL(origin).protocol === 'file:' ? 'index.html' : ''));
    await expect(block('remark:title').locator('.zellij-statement-head')).toHaveText('Remark T.1 (How to read this book)');
    assert.deepEqual(errors, []);
  } finally { page.off('pageerror', onError); }
}
