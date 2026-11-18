import {expect,test} from '@playwright/test';
const types=['feature-grid','stats-band','pricing-table','cta-banner','bento-grid','team-grid','timeline','logo-strip'];

test('reused sections remain readable in both themes with subordinate headings and semantic content',async({page},info)=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/compose-reuse');
  await page.evaluate(()=>document.fonts.ready);
  await expect(page.locator('main h1')).toHaveCount(1);
  await expect(page.getByRole('group',{name:'People',exact:true}).locator('h3')).toHaveText('Meet the people.');
  await expect(page.getByRole('group',{name:'People',exact:true}).locator('h4')).toHaveText('The maker');
  for(const type of types) await expect(page.locator(`.zellij-compose-section > [data-zellij-section="${type}"]`).first()).toBeVisible();
  await expect(page.locator('.zellij-timeline-list')).toHaveJSProperty('tagName','OL');
  await expect(page.locator('.zellij-pricing-features')).toHaveCount(2);
  for(const img of await page.locator('main img').all()) {
    await img.scrollIntoViewIfNeeded();
    await expect.poll(()=>img.evaluate((el:HTMLImageElement)=>el.complete&&el.naturalWidth>0)).toBe(true);
  }
  await page.evaluate(()=>scrollTo(0,0));
  for(const colorScheme of ['light','dark'] as const) {
    await page.emulateMedia({colorScheme});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    const team=page.getByRole('group',{name:'People',exact:true});
    await expect(team.locator('.zellij-team-name')).toHaveCSS('color',await team.evaluate(el=>getComputedStyle(el).color));
    for(const card of await page.locator('.zellij-feature-card,.zellij-pricing-card').all()) {
      const color=await card.evaluate(el=>getComputedStyle(el).color);
      await expect(card.locator('.zellij-feature-title,.zellij-pricing-name')).toHaveCSS('color',color);
    }
    await page.screenshot({path:info.outputPath(`compose-reuse-${colorScheme}.png`),fullPage:true,animations:'disabled'});
  }
  expect(errors).toEqual([]);
});

test('embedded grids resize to their own column and keep bento spans within it',async({page},info)=>{
  await page.setViewportSize({width:1280,height:900});
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/compose-reuse');
  const feature=page.getByRole('group',{name:'Product features',exact:true});
  const bento=page.getByRole('group',{name:'The studio',exact:true});
  for(const node of [feature,bento]) await node.evaluate(el=>{el.style.width='300px';});
  for(const list of [feature.locator('.zellij-feature-list'),bento.locator('.zellij-bento-grid')]) {
    await expect.poll(()=>list.evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(1);
    expect(await list.evaluate(el=>el.scrollWidth<=el.clientWidth+1)).toBe(true);
  }
  await bento.scrollIntoViewIfNeeded();
  await page.screenshot({path:info.outputPath('compose-reuse-narrow.png'),animations:'disabled'});
  await feature.evaluate(el=>{el.style.width='100%';});
  await expect.poll(()=>feature.locator('.zellij-feature-list').evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(2);
});

test('copy controls are independent and keyboard operable',async({page})=>{
  await page.addInitScript(()=>{Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async(text:string)=>{(window as unknown as {copied:string}).copied=text;}}});});
  await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/compose-reuse');
  const first=page.getByRole('group',{name:'Begin creating',exact:true});
  const second=page.getByRole('group',{name:'Read your work',exact:true});
  const button=first.getByRole('button',{name:'Copy the terminal command'});
  await button.focus();await page.keyboard.press('Enter');
  await expect(first.getByRole('button',{name:'Copied the terminal command'})).toBeFocused();
  await expect(second.getByRole('button',{name:'Copy the terminal command'})).toBeVisible();
  expect(await page.evaluate(()=>(window as unknown as {copied:string}).copied)).toBe('zel build');
  await second.getByRole('button',{name:'Copy the terminal command'}).click();
  expect(await page.evaluate(()=>(window as unknown as {copied:string}).copied)).toBe('zel view');
});

test('statistics preserve an authored accessible and print value independently of their animation',async({page},info)=>{
  await page.goto('/compose-reuse');
  const stats=page.getByRole('group',{name:'At a glance',exact:true});
  await stats.scrollIntoViewIfNeeded();
  await expect(stats.locator('.zellij-count-up-animated').first()).toHaveText('48');
  await expect(stats.locator('.zellij-count-up-literal').first()).toHaveText('48');
  await expect(stats.locator('.zellij-count-up-animated').first()).toHaveAttribute('aria-hidden','true');
  // CSS print emulation does not open a print dialog or load offscreen lazy images.
  // Capture the fully loaded document, as in the ordinary light/dark visual check.
  for(const img of await page.locator('main img').all()) {
    await img.scrollIntoViewIfNeeded();
    await expect.poll(()=>img.evaluate((el:HTMLImageElement)=>el.complete&&el.naturalWidth>0)).toBe(true);
  }
  // Model any intermediate frame without depending on animation/capture timing.
  await stats.locator('.zellij-count-up-animated').first().evaluate(el=>{el.textContent='17';});
  await page.emulateMedia({media:'print',colorScheme:'dark'});
  await expect(stats.locator('.zellij-count-up-animated').first()).toBeHidden();
  await expect(page.locator('.zellij-header')).toBeHidden();
  for(const composition of await page.locator('[data-zellij-section="compose"]').all())
    await expect(composition).toHaveCSS('background-color','rgb(255, 255, 255)');
  await expect(stats.locator('.zellij-count-up-literal').first()).toHaveCSS('clip-path','none');
  await expect(stats.locator('.zellij-count-up-literal').first()).toBeVisible();
  await expect(stats.locator('.zellij-count-up-literal').first()).toHaveText('48');
  await expect(page.getByRole('button',{name:'Copy the terminal command'})).toHaveCount(0);
  await expect(page.locator('.zellij-command-text').first()).toHaveCSS('color','rgb(17, 17, 17)');
  await expect(page.locator('.zellij-gradient-text')).toHaveCSS('-webkit-text-fill-color','rgb(17, 17, 17)');
  await expect(page.locator('.zellij-gradient-text')).toHaveCSS('background-image','none');
  for (const card of await page.locator('.zellij-compose-section :is(.zellij-feature-card,.zellij-pricing-card,.zellij-bento-tile)').all()) {
    await expect(card).toHaveCSS('background-color','rgba(0, 0, 0, 0)');
    await expect(card).toHaveCSS('color','rgb(17, 17, 17)');
  }
  for(const img of await page.locator('main img').all())
    await expect(img).toBeVisible();
  await page.screenshot({path:info.outputPath('compose-reuse-print.png'),fullPage:true,animations:'disabled'});
  for(const list of await page.locator('.zellij-compose-section .zellij-feature-list,.zellij-compose-section .zellij-pricing-list,.zellij-compose-section .zellij-bento-grid').all())
    expect(await list.evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(1);
});

test('missing clipboard and observers retain commands and complete statistics without errors',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>{
    Object.defineProperty(window,'IntersectionObserver',{value:undefined});
    Object.defineProperty(navigator,'clipboard',{configurable:true,value:undefined});
  });
  await page.goto('/compose-reuse');
  await expect(page.getByRole('button',{name:'Copy the terminal command'})).toHaveCount(0);
  await expect(page.locator('.zellij-command-text').first()).toHaveText('zel build');
  await expect(page.locator('.zellij-count-up-animated').first()).toHaveText('48');
  expect(errors).toEqual([]);
});

test('forced colours and reduced motion keep links and all section content usable',async({page})=>{
  await page.emulateMedia({forcedColors:'active',reducedMotion:'reduce'});await page.goto('/compose-reuse');
  for(const type of types) await expect(page.locator(`.zellij-compose-section > [data-zellij-section="${type}"]`).first()).toBeVisible();
  await expect(page.locator('.zellij-count-up-animated').first()).toHaveText('48');
  const link=page.getByRole('link',{name:'The maker — Read the story'});
  await link.scrollIntoViewIfNeeded();
  await expect.poll(()=>link.evaluate(el=>{
    const rect=el.getBoundingClientRect();
    const hit=document.elementFromPoint(rect.x+rect.width/2,rect.y+rect.height/2);
    return rect.height>=44 && !!hit && el.contains(hit);
  })).toBe(true);
  await link.click();
  await expect(page).toHaveURL(/\/compose$/);
});

test.describe('without JavaScript',()=>{
  test.use({javaScriptEnabled:false});
  test('all reused sections, static statistics and commands remain readable',async({page})=>{
    await page.goto('/compose-reuse');
    for(const type of types) await expect(page.locator(`.zellij-compose-section > [data-zellij-section="${type}"]`).first()).toBeVisible();
    await expect(page.locator('.zellij-count-up-animated').first()).toHaveText('48');
    await expect(page.getByRole('button',{name:'Copy the terminal command'})).toHaveCount(0);
    await expect(page.locator('.zellij-command-text').first()).toHaveText('zel build');
    await page.getByRole('link',{name:'Explore the composition',exact:true}).click();
    await expect(page).toHaveURL(/\/compose$/);
  });
});
