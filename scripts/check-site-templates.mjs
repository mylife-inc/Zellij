// CLI-to-engine acceptance. --build/--browser are hosted-CI only.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { packageTemplates } from './package-templates.mjs';
const [binary,target,...flags]=process.argv.slice(2);
assert(binary&&target&&flags.every(f=>['--build','--browser'].includes(f)));
assert(!flags.includes('--browser')||flags.includes('--build'));
if(flags.length) assert.equal(process.env.GITHUB_ACTIONS,'true','Container/browser acceptance runs only in hosted CI');
const repo=fileURLToPath(new URL('../',import.meta.url)),zel=path.resolve(binary),root=path.resolve(target);
await mkdir(root); // Never overwrite existing acceptance artifacts.
async function run(command,args) {
  return new Promise((resolve,reject)=>{
    const child=spawn(command,args,{stdio:['ignore','pipe','pipe']});let out='',error='';
    const timer=setTimeout(()=>{child.kill();reject(new Error(`Timed out: ${command}`));},600000);
    child.stdout.on('data',b=>out+=b);child.stderr.on('data',b=>error+=b);
    child.on('error',reject);child.on('close',code=>{clearTimeout(timer);code===0?resolve(out):reject(new Error(`${command} ${args.join(' ')}: ${out}\n${error}`));});
  });
}
const info=JSON.parse(await run(zel,['version','--json']));
const assets=await packageTemplates(zel,`v${info.cli.version}`,path.join(root,'release'));
const again=await packageTemplates(zel,`v${info.cli.version}`,path.join(root,'release-again'));
assert.deepEqual(assets,again,'Template archives must be byte-reproducible');
const catalog=JSON.parse(await readFile(path.join(root,'release/templates-catalog.json'),'utf8'));
const requests=[];
const server=createServer(async(req,res)=>{
  try {
    requests.push({path:req.url,auth:!!req.headers.authorization});
    const name=new URL(req.url,'http://localhost').pathname.slice(1);assert(assets.some(a=>a.name===name));
    res.end(await readFile(path.join(root,'release',name)));
  }catch {res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const catalogUrl=`http://127.0.0.1:${server.address().port}/templates-catalog.json`;
try {
  for(const {manifest} of catalog.templates){
    const args=['-t',`${manifest.id}@${manifest.version}`];
    await run(zel,['create',path.join(root,'bundled',manifest.id),...args,'--offline']);
    await run(zel,['new','site',path.join(root,'remote',manifest.id),...args,'--catalog',catalogUrl,'--cache-dir',path.join(root,'cache')]);
    const receipts=await Promise.all(['bundled','remote'].map(async kind=>JSON.parse(await readFile(path.join(root,kind,manifest.id,'.zel-template.json'),'utf8'))));
    assert.deepEqual(receipts[0].files,receipts[1].files);
    assert.equal(receipts[1].source_sha256,catalog.templates.find(t=>t.manifest.id===manifest.id).archive.sha256);
  }
  const publicDriver=JSON.parse(await run(process.execPath,[path.join(repo,'scripts/check-published-templates.mjs'),zel,catalogUrl.replace('/templates-catalog.json',''),path.join(root,'public-driver')]));
  assert.equal(publicDriver.length,catalog.templates.length);
} finally {await new Promise(resolve=>server.close(resolve));}
assert(requests.every(r=>!r.auth),'Catalog and archive requests must carry no credentials');
for(const {manifest} of catalog.templates){
  const site=path.join(root,'offline',manifest.id);
  await run(zel,['create',site,'-t',manifest.id,'--catalog',catalogUrl,'--cache-dir',path.join(root,'cache'),'--offline']);
  if(manifest.renderer==='portable'){
    await run(zel,['check',site,'--strict']);await run(zel,['build',site]);
  }else{
    await run(process.execPath,[path.join(repo,'engine/packages/zellij/bin/zellij.mjs'),'check',path.join(site,'content'),'--strict','--deep']);
    if(flags.includes('--build'))await run(zel,['build',site]);
  }
}
if(flags.includes('--browser')){
  const {chromium}=await import('playwright');const browser=await chromium.launch();
  try{
    for(const {manifest} of catalog.templates){
      const output=path.join(root,'offline',manifest.id,'site');let host;
      let url=pathToFileURL(path.join(output,'index.html')).href;
      if(manifest.renderer==='hosted'){
        host=createServer(async(req,res)=>{
          try{
            let file=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
            if(file.endsWith('/'))file+='index.html';else if(!path.extname(file))file+='.html';
            const selected=path.resolve(output,'.'+file);assert(selected.startsWith(output+path.sep));
            const types={'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.woff2':'font/woff2','.json':'application/json','.txt':'text/plain'};
            res.setHeader('Content-Type',types[path.extname(selected)]||'application/octet-stream');res.end(await readFile(selected));
          }catch{res.writeHead(404);res.end();}
        });await new Promise(resolve=>host.listen(0,'127.0.0.1',resolve));url=`http://127.0.0.1:${host.address().port}/`;
      }
      try{
        for(const width of [1440,390]){
          const page=await browser.newPage({viewport:{width,height:1000},reducedMotion:'reduce'});const errors=[];
          page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
          await page.goto(url,{waitUntil:'networkidle'});assert(await page.locator('h1').count());
          assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'Template overflows viewport');
          assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,`${manifest.id}-${width}.png`),fullPage:true});
          await page.close();
        }
      }finally{if(host)await new Promise(resolve=>host.close(resolve));}
    }
  }finally{await browser.close();}
}
console.log('Three templates: deterministic packages, local/bundled/remote parity, exact versions, offline cache and strict validation passed.');
