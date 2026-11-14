import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { expect } from '@playwright/test';
import { browserFailureState } from './browser-failure-state.mjs';
export async function checkTerms(page, origin, captures, prefix) {
  const errors=[];const onError=e=>errors.push(e.message);page.on('pageerror',onError);
  const url=relative=>new URL(relative,origin).href;
  async function ready(relative) {await page.goto(url(relative));await page.waitForLoadState('load');await page.evaluate(()=>document.fonts.ready);}
  try {
    await ready('chapters/energy.html');
    const term=page.locator('#_zel-use-glossary-energy-1');
    await expect(term).toHaveText('energy');await term.focus();await expect(term).toBeFocused();await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/\/glossary\.html#_zel-glossary-energy$/);
    await expect(page.locator('#_zel-glossary-energy')).toBeInViewport();
    await expect(page.locator('#_zel-glossary-energy')).toContainText('The capacity to do work');
    await page.locator('#_zel-glossary-energy a[href="#_zel-glossary-sum"]').click();
    await expect(page.locator('#_zel-glossary-sum')).toBeInViewport();
    await page.locator('#_zel-glossary-sum a[href$="chapters/energy.html#_zel-use-glossary-sum-1"]').click();
    await expect(page.locator('#_zel-use-glossary-sum-1')).toBeInViewport();
    await ready('chapters/energy.html');
    await page.locator('#_zel-use-index-energy-2').click();
    await expect(page).toHaveURL(/\/book-index\.html#_zel-index-energy$/);
    await expect(page.locator('#_zel-index-energy .zellij-term-occurrences a')).toHaveCount(4);
    await page.locator('#_zel-index-energy a[href$="chapters/energy.html#_zel-use-index-energy-2"]').click();
    await expect(page.locator('#_zel-use-index-energy-2')).toBeInViewport();
    await mkdir(captures,{recursive:true});
    for(const [file,id] of [['glossary.html','_zel-glossary-energy'],['book-index.html','_zel-index-energy']]) {
      await ready(file);
      await expect(page.locator('.zellij-terms > li')).toHaveCount(3);
      const link=page.locator(`#${id} a`).first();await link.focus();await expect(link).toBeFocused();
      for(const width of [1280,390]) {
        await page.setViewportSize({width,height:900});await page.evaluate(async()=>{await document.fonts.ready;scrollTo(0,0);});
        assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'No terms overflow');
        await page.screenshot({path:path.join(captures,`${prefix}-${file.replace('.html','')}-${width}.png`),fullPage:true});
      }
    }
    assert.deepEqual(errors,[]);
  } catch(error) {
    try {await mkdir(captures,{recursive:true});await writeFile(path.join(captures,`${prefix}-failure.json`),JSON.stringify({error:String(error),origin,errors,state:await browserFailureState(page,{sampleFrames:false})},null,2));await page.screenshot({path:path.join(captures,`${prefix}-failure.png`),fullPage:true,timeout:3000});}catch{/* original error wins */}
    throw error;
  } finally {page.off('pageerror',onError);}
}
