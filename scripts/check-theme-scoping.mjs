// CSS semantics need a real browser. Run only on hosted CI, using the actual stylesheet.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {chromium,webkit} from '@playwright/test';
assert.equal(process.env.GITHUB_ACTIONS,'true');
const css=readFileSync(new URL('../engine/packages/zellij/src/styles/themes.css',import.meta.url),'utf8');
for(const [name,type] of [['chromium',chromium],['webkit',webkit]]) {
 const browser=await type.launch();try {
  const page=await browser.newPage({viewport:{width:1440,height:1000}});
  await page.setContent(`<html data-zellij-theme="kie"><style>
    :root {font-size:16px;--zellij-fg:#111;--zellij-font-mono:monospace;--zellij-surface:#eee}
    .zellij-hero-headline {font-size:64px;font-weight:600}
    .zellij-hero {text-align:center}
    .zellij-eyebrow {font-size:13px}
    .zellij-button {border:1px solid #111;font-weight:400}
    ${css}</style><body>
    <section class="zellij-hero"><h1 class="zellij-hero-headline" id="outer">Root</h1></section>
    <main data-zellij-theme="paper"><section class="zellij-hero"><h2 class="zellij-hero-headline" id="nested">Nested</h2></section>
      <div data-zellij-theme="kie"><section class="zellij-hero"><h2 class="zellij-hero-headline" id="inner">Re-entered</h2></section></div>
    </main>
    <div data-zellij-theme="cupertino"><p class="zellij-eyebrow" id="cupertino">Eyebrow</p></div>
    <div data-zellij-theme="mono"><button class="zellij-button" id="mono">Action</button></div>
    <div data-zellij-theme="porcelain"><h2 class="zellij-section-heading" id="porcelain">Heading</h2></div>
    <div data-zellij-theme="aurora"><div class="zellij-feature-card" id="aurora">Card</div></div>
    <div data-zellij-theme="carbon"><p class="zellij-eyebrow" id="carbon">Eyebrow</p></div>
    </body></html>`);
  const style=(id,key)=>page.locator('#'+id).evaluate((el,k)=>getComputedStyle(el)[k],key);
  assert.equal(await style('outer','fontSize'),'72px');assert.equal(await style('outer','textAlign'),'start');
  assert.equal(await style('nested','fontSize'),'64px');assert.equal(await style('nested','textAlign'),'center');
  assert.equal(await style('inner','fontSize'),'72px');
  assert.equal(await style('cupertino','fontSize'),'20px');
  assert.equal(await style('mono','borderTopWidth'),'2px');
  assert.equal(await style('porcelain','fontWeight'),'400');
  assert.equal(await style('aurora','backdropFilter'),'blur(12px)');
  assert.match(await style('carbon','fontFamily'),/monospace/);
  console.log(`${name}: theme presentation and nested boundaries verified`);
 }finally{await browser.close();}
}
