// Hosted CI only: build, publish, pull and serve complete PDF-enabled Next books.
import assert from 'node:assert/strict';
import { writeLanguageBook, normalizedPrintText, requiredPrintTerms } from './fixtures/book-languages.mjs';
import { checkArabicBook } from './check-language-book.mjs';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { chromium, expect } from '@playwright/test';
import { localRegistry, pullPublishedImage } from './fixtures/local-registry.mjs';

import { printStyles, withPrintStyle } from './fixtures/print-styles.mjs';

const zel = path.resolve(process.argv[2] || 'cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-next-pdf-'));
const captures = path.resolve('test-results/next-pdf');
const containers = [], images = [];
const suffix = `${process.pid}-${Date.now()}`;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const docker = args => execFileSync('docker', args, {encoding:'utf8',timeout:120000}).trim();
const run = args => execFileSync(zel, args, {encoding:'utf8',timeout:600000,stdio:['ignore','pipe','pipe']});
const summary = [];
async function ready(url) {
  for (let n=0;n<120;n++) {
    try { if ((await fetch(url,{signal:AbortSignal.timeout(1000)})).ok) return; } catch { /* startup */ }
    await new Promise(resolve=>setTimeout(resolve,250));
  }
  throw new Error(`Next image did not serve ${url}`);
}
try {
  const registry = await localRegistry(docker, containers, suffix);
  for (const [edition,prefix,style,language] of [['study',''],['technical','/books/technical'],['technical','/books/custom',printStyles.technical],['technical','/books/arabic',undefined,'ar']]) {
    const variant = language ? language+'-'+edition : style ? edition+'-custom' : edition;
    const source = path.join(root,variant);
    if(language) writeLanguageBook(source,language,edition);
    else run(['create',source,'-t',`book-${edition}`,'--offline']);
    const tag = `${registry}/books/${variant}:${suffix}`; images.push(tag);
    const configFile = path.join(source,'zel.yaml');
    let config = language ? JSON.stringify({...JSON.parse(await readFile(configFile,'utf8')),image:{name:tag},publish:{action:'push',image:'next'}},null,2)+'\n' : (await readFile(configFile,'utf8')).replace('enabled: false','enabled: true') + `\nimage: {name: '${tag}'}\npublish: {action: push, image: next}\n`;
    if (style) config = withPrintStyle(config,style);
    await writeFile(configFile,config);
    await mkdir(path.join(source,'site'));
    await writeFile(path.join(source,'site/index.html'),'previous export');
    const chapterFile = path.join(source,'content/chapters/01-first.md');
    const chapter = await readFile(chapterFile,'utf8');
    const flags = prefix ? ['--base-path',prefix] : [];
    run(['pub',source,...flags]);
    pullPublishedImage(docker,tag);
    const image = JSON.parse(docker(['image','inspect',tag]))[0];
    const name = `zel-pdf-${variant}-${suffix}`;containers.push(name);
    docker(['run','-d','--name',name,'-p','127.0.0.1::3000',tag]);
    const info = JSON.parse(docker(['inspect',name]))[0];
    assert.equal(info.Config.User,'nextjs');
    docker(['exec',name,'sh','-ec','! command -v lualatex; ! command -v pandoc; ! command -v npm; test ! -d /usr/share/texlive']);
    const origin = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
    await ready(`${origin}${prefix}/`);
    docker(['exec',name,'sh','-c',info.Config.Healthcheck.Test[1]]);
    const asset = `${origin}${prefix}/_zellij/assets/pdf`;
    const response = await fetch(`${asset}/book.pdf`);
    assert.equal(response.status,200);assert.match(response.headers.get('content-type'),/application\/pdf/);
    const pdf = Buffer.from(await response.arrayBuffer());assert(pdf.subarray(0,5).equals(Buffer.from('%PDF-')));
    const receipt = await (await fetch(`${asset}/pdf.json`)).json();
    const book = await (await fetch(`${origin}${prefix}/_zellij/assets/book.json`)).json();
    assert.equal(receipt.revision,book.publication.revision);assert.equal(receipt.pdf_sha256,sha(pdf));
    assert.equal(receipt.template,`${edition}-v1`);
    if (style) assert.deepEqual(receipt.settings.style,style);
    for(const file of ['book.tex','build.log','console.log','assets/image-0.png','fonts/amiri/Amiri-Regular.ttf','fonts/libertinus/LibertinusSerif-Regular.otf']) assert.equal((await fetch(`${asset}/${file}`)).status,404);
    assert.equal((await fetch(`${origin}${prefix}/.zel-pdf-work/book.pdf`)).status,404);
    if(prefix) assert.equal((await fetch(`${origin}/_zellij/assets/pdf/book.pdf`)).status,404);
    const out = path.join(captures,variant);await mkdir(out,{recursive:true});
    const pdfPath = path.join(out,'book.pdf');await writeFile(pdfPath,pdf);
    await writeFile(path.join(out,'pdf.json'),JSON.stringify(receipt,null,2));
    await writeFile(path.join(out,'book.json'),JSON.stringify(book,null,2));
    execFileSync('qpdf',['--check',pdfPath]);
    const text = execFileSync('pdftotext',['-layout',pdfPath,'-'],{encoding:'utf8'});
    for(const word of language ? requiredPrintTerms(language) : [book.publication.manifest.title,'Contents','Chapter 1','Chapter 2','Bibliography','Glossary','Index']) assert(language ? normalizedPrintText(text).includes(normalizedPrintText(word)) : text.replace(/\s+/g,' ').includes(word),word);
    assert(!text.includes('??'));
    const pdfInfo = execFileSync('pdfinfo',[pdfPath],{encoding:'utf8'});
    const pages = Number(pdfInfo.match(/Pages:\s+(\d+)/)[1]);
    await writeFile(path.join(out,'text.txt'),text);await writeFile(path.join(out,'pdfinfo.txt'),pdfInfo);
    await writeFile(path.join(out,'fonts.txt'),execFileSync('pdffonts',[pdfPath]));
    for(let n=1;n<=pages;n++) execFileSync('pdftoppm',['-f',String(n),'-singlefile','-scale-to','1300','-png',pdfPath,path.join(out,`page-${n}`)]);
    const browser = await chromium.launch();
    try {
      for(const javaScriptEnabled of [true,false]) {
        const context = await browser.newContext({javaScriptEnabled,acceptDownloads:true});
        const page = await context.newPage(), errors=[];
        page.on('pageerror',e=>errors.push(e.message));
        for(const width of [1280,390]) {
          await page.setViewportSize({width,height:900});
          const route = width===1280 && !language ? '/' : book.routes.find(r=>r.id==='first').route;
          await page.goto(`${origin}${prefix}${route}`);
          if(language) {
            await checkArabicBook(page,out,`arabic-${width}-${javaScriptEnabled}`);
            await page.goto(`${origin}${prefix}${route}`);
          }
          const link = page.getByRole('link',{name:'Download the complete book (PDF)',exact:true});
          await expect(link).toHaveAttribute('href',`${prefix}/_zellij/assets/pdf/book.pdf`);
          await expect(link).toHaveAttribute('download','');
          await link.focus();await expect(link).toBeFocused();
          assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);
          // Viewport capture preserves the focused reader's actual sticky chrome.
          await page.screenshot({path:path.join(out,`download-${width}-${javaScriptEnabled}.png`)});
          const downloadPromise = page.waitForEvent('download');await page.keyboard.press('Enter');
          const download = await downloadPromise;assert.equal(download.suggestedFilename(),'book.pdf');
          const downloaded = path.join(out,'downloaded.pdf');await download.saveAs(downloaded);
          assert.equal(sha(await readFile(downloaded)),receipt.pdf_sha256);await rm(downloaded);
        }
        assert.deepEqual(errors,[]);await context.close();
      }
    } finally { await browser.close(); }
    assert.equal(await readFile(configFile,'utf8'),config);
    assert.equal(await readFile(chapterFile,'utf8'),chapter);
    assert.equal(await readFile(path.join(source,'site/index.html'),'utf8'),'previous export');
    // A print-incompatible edit cannot replace the successful image or push it.
    await writeFile(chapterFile,chapter+'\n![Remote print image](https://example.com/not-local.png)\n');
    const failed = spawnSync(zel,['pub',source,...flags],{encoding:'utf8',timeout:600000});
    assert.notEqual(failed.status,0);assert.match(failed.stderr,/print images must be local/);
    assert.equal(JSON.parse(docker(['image','inspect',tag]))[0].Id,image.Id);
    // The fresh-pull helper removes the tag first; release our own server before that.
    docker(['rm','-f',name]);containers.splice(containers.indexOf(name),1);
    pullPublishedImage(docker,tag);
    assert.equal(JSON.parse(docker(['image','inspect',tag]))[0].Id,image.Id);
    assert.equal(await readFile(path.join(source,'site/index.html'),'utf8'),'previous export');
    summary.push({edition,variant,prefix,pages,revision:receipt.revision,pdf_sha256:receipt.pdf_sha256,pdf_bytes:pdf.length,image_bytes:image.Size,architecture:image.Architecture});
    await writeFile(path.join(captures,'summary.json'),JSON.stringify(summary,null,2));
    console.log(`${variant}: Next image PDF publication, download, revision, private-file exclusion and failure preservation passed.`);
  }
} finally {
  for(const name of containers) {try {docker(['rm','-f',name]);} catch {}}
  for(const tag of images) {try {docker(['image','rm',tag]);} catch {}}
  await rm(root,{recursive:true,force:true});
}
