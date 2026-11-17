import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';
import { languages } from './fixtures/book-languages.mjs';

// Shared assertions for a moved file:// book and a prefixed Next publication.
export async function checkArabicBook(page, captures, name) {
  await page.emulateMedia({reducedMotion:'reduce'});
  await expect(page.locator('html')).toHaveAttribute('lang','ar');
  await expect(page.locator('html')).toHaveAttribute('dir','rtl');
  const heading = page.getByRole('heading',{name:languages.ar.first,exact:true});
  await expect(heading).toBeVisible();
  await page.evaluate(()=>document.fonts.ready);
  await expect(heading).toHaveCSS('font-family',/Zellij Amiri/);
  assert(await page.evaluate(()=>[...document.fonts].some(f=>f.family.replaceAll('"','').replaceAll("'",'')==='Zellij Amiri'&&f.status==='loaded')),'Pinned Amiri is loaded');
  await expect(page.getByRole('cell',{name:languages.ar.row[2],exact:true})).toBeVisible();
  assert.equal(await page.locator('pre').first().evaluate(e=>getComputedStyle(e).direction),'ltr');
  await mkdir(captures,{recursive:true});
  const overflow = await page.evaluate(() => ({width:innerWidth,scrollWidth:document.documentElement.scrollWidth,
    elements:[...document.querySelectorAll('body *')].flatMap(el=>{
      const r=el.getBoundingClientRect(); return r.left < -1 || r.right > innerWidth+1 ? [{tag:el.tagName,class:el.className,left:r.left,right:r.right,width:r.width}] : [];
    })}));
  if (overflow.scrollWidth > overflow.width+1) {
    await writeFile(path.join(captures,`${name}-overflow.json`),JSON.stringify(overflow,null,2));
    await page.screenshot({path:path.join(captures,`${name}-overflow.png`),fullPage:true});
  }
  assert(overflow.scrollWidth <= overflow.width+1, `${name}: RTL page must not overflow (${overflow.scrollWidth}/${overflow.width})`);
  const skip = page.locator('.zellij-skip-link');
  if(await skip.count()) {
    await skip.focus(); await expect(skip).toBeFocused();
    const initialBounds = await skip.boundingBox();
    // Logical positioning/font metrics can land on a fractional CSS pixel.
    // Poll layout as well as focus, using the same one-pixel tolerance as overflow.
    await expect.poll(()=>skip.evaluate(el=>{const r=el.getBoundingClientRect();return {
      left:r.left,right:r.right,width:innerWidth,reachable:r.left>=-1&&r.right<=innerWidth+1,
    };}), {message:'RTL skip link is reachable on screen'}).toMatchObject({reachable:true});
    await writeFile(path.join(captures,`${name}-skip-link.json`),JSON.stringify({initial:initialBounds,settled:await skip.boundingBox(),viewport:page.viewportSize()},null,2));
    await page.keyboard.press('Enter');
  }
  for (const mode of ['light','dark']) {
    await page.evaluate(m=>document.documentElement.setAttribute('data-zellij-mode',m),mode);
    await page.screenshot({path:path.join(captures,`${name}-${mode}.png`),fullPage:true});
  }
  const link=page.getByRole('link',{name:languages.ar.second,exact:true}).last();
  await link.focus();await expect(link).toBeFocused();await page.keyboard.press('Enter');
  await expect(page.getByRole('heading',{name:languages.ar.second,exact:true})).toBeVisible();
}
