import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';
import { languages } from './fixtures/book-languages.mjs';

// Shared assertions for a moved file:// book and a prefixed Next publication.
export async function checkArabicBook(page, captures, name) {
  await expect(page.locator('html')).toHaveAttribute('lang','ar');
  await expect(page.locator('html')).toHaveAttribute('dir','rtl');
  const heading = page.getByRole('heading',{name:languages.ar.first,exact:true});
  await expect(heading).toBeVisible();
  await page.evaluate(()=>document.fonts.ready);
  await expect(heading).toHaveCSS('font-family',/Zellij Amiri/);
  assert(await page.evaluate(()=>[...document.fonts].some(f=>f.family.replaceAll('"','').replaceAll("'",'')==='Zellij Amiri'&&f.status==='loaded')),'Pinned Amiri is loaded');
  await expect(page.getByRole('cell',{name:languages.ar.row[2],exact:true})).toBeVisible();
  assert.equal(await page.locator('pre').first().evaluate(e=>getComputedStyle(e).direction),'ltr');
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);
  await mkdir(captures,{recursive:true});
  for (const mode of ['light','dark']) {
    await page.evaluate(m=>document.documentElement.setAttribute('data-zellij-mode',m),mode);
    await page.screenshot({path:path.join(captures,`${name}-${mode}.png`),fullPage:true});
  }
  const link=page.getByRole('link',{name:languages.ar.second,exact:true}).last();
  await link.focus();await expect(link).toBeFocused();await page.keyboard.press('Enter');
  await expect(page.getByRole('heading',{name:languages.ar.second,exact:true})).toBeVisible();
}
