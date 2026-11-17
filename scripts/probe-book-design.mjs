// Temporary hosted inspection; no local browser or typesetter is launched.
import { chromium } from '@playwright/test';
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
const out = 'test-results/book-design-reference'; mkdirSync(out, { recursive: true });
const image = readFileSync('cli/crates/zellij-cli/src/pdf.rs','utf8').match(/pub\(crate\) const IMAGE: &str = "([^"]+)"/)[1];
const fonts = execFileSync('docker', ['run','--rm','--network','none','--entrypoint','sh',image,'-c', String.raw`find /usr/share/fonts /opt /usr/local/share /usr/local/texlive -type f \( -iname "*.ttf" -o -iname "*.otf" -o -name "babel.sty" -o -name "bidi.sty" -o -name "arabic.ldf" -o -name "babel-ar.ini" \) 2>/dev/null || true`], {encoding:'utf8', maxBuffer:16*1024*1024});
writeFileSync(out+'/fonts.txt', fonts);
const browser = await chromium.launch({headless:true});
try {
  for (const [name, viewport] of [['desktop',{width:1440,height:1000}], ['mobile',{width:390,height:844}]]) {
    const context = await browser.newContext({viewport, colorScheme:'light', reducedMotion:'reduce'});
    const page = await context.newPage();
    const response = await page.goto('https://kie.ai', {waitUntil:'domcontentloaded',timeout:60000});
    await page.locator('h1').waitFor({timeout:30000}); await page.evaluate(()=>document.fonts.ready);
    await page.screenshot({path:out+'/'+name+'.png',fullPage:true,animations:'disabled'});
    const data = await page.evaluate(()=>({url:location.href,title:document.title,elements:[...document.querySelectorAll('body, header, nav, h1, h2, h3, button, a, section')].slice(0,180).map(e=>{const c=getComputedStyle(e),r=e.getBoundingClientRect();return {tag:e.tagName,text:e.textContent.trim().slice(0,90),box:{x:r.x,y:r.y,width:r.width,height:r.height},font:c.fontFamily,size:c.fontSize,weight:c.fontWeight,color:c.color,background:c.background,border:c.border,radius:c.borderRadius,tracking:c.letterSpacing,lineHeight:c.lineHeight,padding:c.padding,gap:c.gap};})}));
    data.status=response.status();writeFileSync(out+'/'+name+'.json',JSON.stringify(data,null,2));await context.close();
  }
} finally { await browser.close(); }
