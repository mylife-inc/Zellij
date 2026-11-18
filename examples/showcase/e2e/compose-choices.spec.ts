import { expect, test, type Page } from '@playwright/test';

const choice = (page: Page, role: 'radiogroup' | 'tablist', name: string) => page.locator('.zellij-compose-choice').filter({has: page.getByRole(role, {name, exact:true})});
for (const route of ['compose-selector', 'compose-explorer']) {
  test(`${route} stays readable in light/dark modes and fits its available space`, async ({page, isMobile}, info) => {
    const errors: string[] = []; page.on('pageerror', error => errors.push(error.message));
    await page.goto(`/${route}`);
    await expect(page.locator('.zellij-compose-choice').first()).toHaveAttribute('data-ready','true');
    await expect(page.locator('main h1')).toHaveCount(1);
    await page.evaluate(() => document.fonts.ready);
    for (const img of await page.locator('main img:visible').all()) {
      await img.scrollIntoViewIfNeeded();
      await expect.poll(() => img.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
    }
    const layout = page.locator('.zellij-choice-panel:visible .zellij-choice-panel-layout').first();
    const positions = await layout.evaluate(node => ({copy: node.querySelector('.zellij-choice-copy')!.getBoundingClientRect().toJSON(), media: node.querySelector('.zellij-choice-media')!.getBoundingClientRect().toJSON()}));
    if (isMobile) expect(positions.media.top).toBeGreaterThan(positions.copy.bottom);
    else expect(positions.media.left).toBeGreaterThan(positions.copy.left);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
    const ids = await page.locator('main [id]').evaluateAll(nodes => nodes.map(node => node.id));
    expect(new Set(ids).size).toBe(ids.length);
    for (const colorScheme of ['light','dark'] as const) {
      await page.emulateMedia({colorScheme}); await page.evaluate(() => scrollTo({top:0,behavior:'instant'}));
      for (const node of await page.locator('.zellij-compose-choice').all()) {
        const color = await node.evaluate(el => getComputedStyle(el).color);
        await expect(node.locator('.zellij-choice-heading:visible')).toHaveCSS('color', color);
      }
      await page.screenshot({path:info.outputPath(`${route}-${colorScheme}.png`),fullPage:true,animations:'disabled'});
    }
    expect(errors).toEqual([]);
  });
}

test('product choices synchronize copy, media, facts and actions without changing another instance', async ({page}) => {
  await page.goto('/compose-selector');
  const primary = choice(page,'radiogroup','Choose a finish');
  const group = primary.getByRole('radiogroup');
  const model = choice(page,'radiogroup','Choose a model');
  await expect(group.getByRole('radio',{name:'Copper',exact:true})).toBeChecked();
  await expect(primary.getByRole('heading',{name:'A warmer perspective.'})).toBeVisible();
  await group.getByRole('radio',{name:'Silver',exact:true}).check();
  await expect(primary.getByRole('heading',{name:'Quiet clarity.'})).toBeVisible();
  await expect(primary.locator('.zellij-choice-panel:visible img')).toHaveAttribute('src',/choice-silver\.svg/);
  await expect(primary.locator('.zellij-choice-panel:visible dd').first()).toHaveText('Silver');
  await expect(primary.getByRole('link',{name:'Explore silver'})).toHaveAttribute('href','/guide');
  await expect(model.getByRole('radio',{name:'Essential',exact:true})).toBeChecked();
  expect(await group.getByRole('radio').first().getAttribute('name')).not.toBe(await model.getByRole('radio').first().getAttribute('name'));
  await group.getByRole('radio',{name:'Silver',exact:true}).focus();
  await page.keyboard.press('End'); await expect(group.getByRole('radio',{name:'Night',exact:true})).toBeFocused();
  await page.keyboard.press('ArrowRight'); await expect(group.getByRole('radio',{name:'Silver',exact:true})).toBeChecked();
  await page.keyboard.press('ArrowDown'); await expect(group.getByRole('radio',{name:'Copper',exact:true})).toBeChecked();
  await page.keyboard.press('ArrowUp'); await expect(group.getByRole('radio',{name:'Silver',exact:true})).toBeFocused();
  await page.keyboard.press('Tab'); await expect(primary.locator('.zellij-choice-panel:visible')).toBeFocused();
  await model.getByRole('radio',{name:'Studio Pro',exact:true}).check();
  await expect(model.getByRole('heading',{name:'A little more room.'})).toBeVisible();
  await expect(group.getByRole('radio',{name:'Silver',exact:true})).toBeChecked();
  await primary.getByRole('link',{name:'Explore silver'}).click(); await expect(page).toHaveURL(/\/guide(?:\/|$)/);
});

test('feature tabs and vertical lists have matching panels, isolated state and directional keyboard operation', async ({page}) => {
  await page.goto('/compose-explorer');
  const primary = choice(page,'tablist','Explore the features');
  const tabs = primary.getByRole('tablist');
  const secondary = choice(page,'tablist','An independent feature list');
  await tabs.getByRole('tab',{name:'Sound',exact:true}).focus();
  await page.keyboard.press('ArrowRight');
  await expect(tabs.getByRole('tab',{name:'Battery',exact:true})).toBeFocused();
  await expect(tabs.getByRole('tab',{name:'Battery',exact:true})).toHaveAttribute('aria-selected','true');
  await expect(primary.getByRole('tabpanel',{name:'Battery',exact:true})).toBeVisible();
  await expect(primary.getByRole('heading',{name:'A longer conversation.'})).toBeVisible();
  await expect(secondary.getByRole('tab',{name:'Design',exact:true})).toHaveAttribute('aria-selected','true');
  await tabs.evaluate(node => window.addEventListener('keydown', event => { if (node.contains(event.target as Node)) node.setAttribute('data-prevented',String(event.defaultPrevented)); }));
  await page.keyboard.press('ArrowDown'); await expect(tabs).toHaveAttribute('data-prevented','false');
  await page.keyboard.press('End'); await expect(tabs.getByRole('tab',{name:'Design',exact:true})).toBeFocused();
  await page.keyboard.press('ArrowRight'); await expect(tabs.getByRole('tab',{name:'Sound',exact:true})).toBeFocused();
  await page.keyboard.press('Tab'); await expect(primary.getByRole('tabpanel',{name:'Sound',exact:true})).toBeFocused();
  await expect(secondary.getByRole('tablist')).toHaveAttribute('aria-orientation','vertical');
  await secondary.getByRole('tab',{name:'Design',exact:true}).focus();
  await page.keyboard.press('Home'); await page.keyboard.press('ArrowDown');
  await expect(secondary.getByRole('tab',{name:'Battery',exact:true})).toBeFocused();
  await expect(secondary.getByRole('tabpanel',{name:'Battery',exact:true})).toBeVisible();
  await expect(tabs.getByRole('tab',{name:'Sound',exact:true})).toHaveAttribute('aria-selected','true');
  for (const tab of await page.getByRole('tab').all()) {
    const target = (await tab.getAttribute('aria-controls'))!;
    expect(await tab.evaluate((node,id) => document.getElementById(id)?.getAttribute('aria-labelledby') === node.id,target)).toBe(true);
  }
});

test('RTL navigation follows the horizontal reading direction', async ({page}) => {
  for (const [route,role,name] of [['compose-selector','radio','Choose a finish'],['compose-explorer','tab','Explore the features']] as const) {
    await page.goto(`/${route}`); await page.evaluate(() => {document.documentElement.dir='rtl';});
    const group = page.getByRole(role==='radio'?'radiogroup':'tablist',{name,exact:true});
    await group.getByRole(role).first().focus(); await page.keyboard.press('Home'); await page.keyboard.press('ArrowLeft');
    await expect(group.getByRole(role).nth(1)).toBeFocused();
    await page.keyboard.press('ArrowRight'); await expect(group.getByRole(role).first()).toBeFocused();
  }
});

test('print reveals every choice while reduced motion and forced colours preserve selection', async ({page}) => {
  for (const route of ['compose-selector','compose-explorer']) {
    await page.goto(`/${route}`);
    const primary = page.locator('.zellij-compose-choice').first();
    await expect(primary).toHaveAttribute('data-ready','true');
    await page.emulateMedia({reducedMotion:'reduce',forcedColors:'active'});
    const control = primary.locator('.zellij-choice-controls').getByRole(route==='compose-selector'?'radio':'tab').last();
    await control.focus(); await page.keyboard.press('End');
    await expect(control).toBeFocused();
    await expect(primary.locator('.zellij-choice-panel:visible')).toHaveCount(1);
    await expect(primary.locator('.zellij-choice-panel:visible')).toHaveCSS('animation-name','none');
    await page.emulateMedia({media:'print'});
    for (const panel of await page.locator('.zellij-choice-panel').all()) await expect(panel).toBeVisible();
    for (const controls of await page.locator('.zellij-choice-controls').all()) await expect(controls).toBeHidden();
    await page.emulateMedia({media:'screen',forcedColors:'none'});
    await expect(primary.locator('.zellij-choice-panel:visible')).toHaveCount(1);
  }
});

test('nested choices remain contained with long labels and larger text', async ({page}) => {
  await page.setViewportSize({width:360,height:800}); await page.goto('/compose-selector');
  await page.locator('.zellij-compose-choice').last().evaluate(node => {
    document.documentElement.style.fontSize='24px';
    node.querySelector('.zellij-choice-option-face > span')!.textContent='An unusually long product model name that still needs a readable control';
  });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.getByRole('radio',{name:'Studio Pro',exact:true}).check();
  await expect(page.getByRole('heading',{name:'A little more room.'})).toBeVisible();
});

test.describe('without JavaScript', () => {
  test.use({javaScriptEnabled:false});
  test('all stories, facts, images and actions remain in source order, including print', async ({page}) => {
    for (const route of ['compose-selector','compose-explorer']) {
      await page.goto(`/${route}`);
      for (const controls of await page.locator('.zellij-choice-controls').all()) await expect(controls).toBeHidden();
      for (const panel of await page.locator('.zellij-choice-panel').all()) await expect(panel).toBeVisible();
      await expect(page.locator('.zellij-compose-choice').first()).toHaveAttribute('data-ready','false');
      await page.emulateMedia({media:'print'});
      for (const panel of await page.locator('.zellij-choice-panel').all()) await expect(panel).toBeVisible();
      await page.emulateMedia({media:'screen'});
    }
    await page.getByRole('link',{name:'Explore sound',exact:true}).first().click();
    await expect(page).toHaveURL(/\/guide(?:\/|$)/);
  });
});
