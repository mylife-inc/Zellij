import { expect, test, type Locator } from '@playwright/test';

async function progress(scene: Locator, value: number, settle = true) {
  await scene.evaluate((node, p) => {
    const copy = node.querySelector<HTMLElement>('.zellij-story-copy')!;
    const top = parseFloat(getComputedStyle(copy).top);
    const parts = [...node.children] as HTMLElement[];
    const travel = node.getBoundingClientRect().height - Math.max(...parts.map(part => part.offsetHeight));
    scrollTo({top: scrollY + node.getBoundingClientRect().top - top + travel * p, behavior: 'instant'});
  }, value);
  if (settle) await expect.poll(async () => Number(await scene.getAttribute('data-progress'))).toBeCloseTo(value, 1);
}

async function wideStory(page: import('@playwright/test').Page) {
  await page.setViewportSize({width: 1280, height: 900});
  await page.emulateMedia({reducedMotion:'no-preference'});
  await page.goto('/compose-story');
  const story = page.getByRole('group', {name:'The folding study'});
  await expect(story).toHaveAttribute('data-story-active','true');
  return story;
}

test('stories retain readable light/dark layouts, imagery and ordinary scene order', async ({page}, info) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.emulateMedia({reducedMotion:'reduce'});
  await page.goto('/compose-story');
  await expect(page.locator('main h1')).toHaveCount(1);
  await page.evaluate(() => document.fonts.ready);
  for (const img of await page.locator('main img').all()) await expect.poll(() => img.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
  for (const colorScheme of ['light','dark'] as const) {
    await page.emulateMedia({colorScheme});
    const heading = page.locator('#independent-story .zellij-compose-heading').first();
    expect(await heading.evaluate(el => getComputedStyle(el).color)).toBe(await heading.evaluate(el => getComputedStyle(el.parentElement!).color));
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
    await page.screenshot({path:info.outputPath(`compose-story-${colorScheme}.png`),fullPage:true,animations:'disabled'});
  }
  expect(errors).toEqual([]);
});

test('pinned scenes seek actual video forwards and backwards without playback', async ({page}, info) => {
  const story = await wideStory(page);
  const scene = story.locator('.zellij-story-scene').nth(1);
  const video = scene.locator('video');
  await expect(scene.locator('.zellij-story-copy')).toHaveCSS('position','sticky');
  await progress(scene,0.2);
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.readyState)).toBeGreaterThan(1);
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(0.46,1);
  await progress(scene,0.8);
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(1.54,1);
  await page.screenshot({path:info.outputPath('compose-story-scrub.png'),animations:'disabled'});
  await progress(scene,0.2);
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(0.46,1);
  await expect(video).not.toHaveAttribute('controls');
  expect(await video.evaluate((v: HTMLVideoElement) => v.paused && v.muted)).toBe(true);
  await expect(video.locator('track')).toHaveAttribute('src',/_zellij\/assets\/compose-motion.en.vtt$/);
  await expect(story.locator('.zellij-story-copy')).toHaveCount(3);
  // Jump beyond both ends: final seeks still settle after leaving the observer.
  await scene.evaluate(node => scrollTo({top:scrollY+node.getBoundingClientRect().bottom+innerHeight,behavior:'instant'}));
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(1.9,1);
  await page.evaluate(() => scrollTo({top:0,behavior:'instant'}));
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(0.1,1);
});

test('late metadata uses the latest scroll position and does not start playback', async ({page}) => {
  let release!: () => void;
  let held = false;
  const gate = new Promise<void>(resolve => {release=resolve;});
  await page.route('**/compose-motion.mp4',async route => {held=true;await gate;await route.continue();});
  const story = await wideStory(page);
  const scene = story.locator('.zellij-story-scene').nth(1);
  await progress(scene,0.2);
  await expect.poll(() => held).toBe(true);
  await progress(scene,0.8);
  release();
  const video=scene.locator('video');
  await expect.poll(() => video.evaluate((v: HTMLVideoElement) => v.currentTime)).toBeCloseTo(1.54,1);
  expect(await video.evaluate((v: HTMLVideoElement) => v.paused)).toBe(true);
});

test('reading control is local and keyboard navigation returns to ordinary flow', async ({page}) => {
  const story=await wideStory(page);
  const other=page.getByRole('group',{name:'A second perspective'});
  await expect(other).toHaveAttribute('data-story-active','true');
  await story.getByRole('button',{name:'Read without scroll effects'}).click();
  await expect(story).toHaveAttribute('data-story-active','false');
  await expect(story.getByRole('button',{name:'Scroll effects off'})).toBeFocused();
  await expect(other).toHaveAttribute('data-story-active','true');
  await expect(story.locator('video')).toHaveAttribute('controls','');
  await page.keyboard.press('PageDown');
  await expect(other).toHaveAttribute('data-story-active','false');
  await expect(other.locator('.zellij-story-media').first()).toHaveCSS('position','static');
});

test('narrow containers, tall text, motion preferences and printing keep all scenes', async ({page}) => {
  const story=await wideStory(page);
  await story.evaluate(node => {node.style.width='600px';});
  await expect(story).toHaveAttribute('data-story-active','false');
  await story.evaluate(node => {node.style.removeProperty('width');});
  await expect(story).toHaveAttribute('data-story-active','true');
  const body=story.locator('.zellij-compose-text').first();
  const text=await body.textContent();
  await body.evaluate(node => {node.textContent='Long reading content. '.repeat(180);});
  await expect(story).toHaveAttribute('data-story-active','false');
  await body.evaluate((node,text) => {node.textContent=text;},text);
  await expect(story).toHaveAttribute('data-story-active','true');
  await page.emulateMedia({forcedColors:'active'});
  await expect(story).toHaveAttribute('data-story-active','false');
  await page.emulateMedia({forcedColors:'none',reducedMotion:'reduce'});
  await expect(story).toHaveAttribute('data-story-active','false');
  await page.emulateMedia({reducedMotion:'no-preference'});
  await expect(story).toHaveAttribute('data-story-active','true');
  await page.emulateMedia({media:'print'});
  await expect(story).toHaveAttribute('data-story-active','false');
  for (const scene of await story.locator('.zellij-story-scene').all()) {
    await expect(scene).toHaveCSS('min-height','0px');
    await expect(scene.locator('.zellij-story-copy')).toHaveCSS('position','static');
    await expect(scene.locator('figure')).toBeVisible();
  }
});

for (const failure of ['duration','network'] as const) test(`video ${failure} failure preserves reading and native controls`, async ({page}) => {
  if (failure==='network') await page.route('**/compose-motion.mp4',route=>route.abort());
  const story=await wideStory(page);
  const scene=story.locator('.zellij-story-scene').nth(1);
  if (failure==='duration') await scene.locator('video').evaluate(node=>{node.dataset.end='30';});
  await progress(scene,0.5,false);
  await expect(story).toHaveAttribute('data-story-active','false');
  await expect(scene.locator('video')).toHaveAttribute('controls','');
  await expect(story.getByRole('link',{name:'Open video: Three sheets folding into place'})).toBeVisible();
  await expect(story.getByRole('heading',{name:'Keep what matters.'})).toBeVisible();
});

test('missing observers retain ordinary reading without starting media', async ({page}) => {
  await page.addInitScript(()=>{Object.defineProperty(window,'IntersectionObserver',{value:undefined});});
  await page.goto('/compose-story');
  for(const story of await page.locator('.zellij-compose-story').all()) await expect(story).toHaveAttribute('data-story-active','false');
  await expect(page.locator('video')).toHaveAttribute('preload','none');
});

test.describe('without JavaScript',()=>{
  test.use({javaScriptEnabled:false});
  test('every scene and native video remains accessible',async({page})=>{
    await page.goto('/compose-story');
    await expect(page.locator('.zellij-story-scene')).toHaveCount(5);
    for(const heading of await page.locator('.zellij-story-copy h3').all()) await expect(heading).toBeVisible();
    await expect(page.locator('video')).toHaveAttribute('controls','');
    await expect(page.getByRole('button',{name:'Read without scroll effects'})).toHaveCount(0);
    await page.getByRole('link',{name:'Explore the composition'}).click();
    await expect(page).toHaveURL(/\/compose$/);
  });
});
