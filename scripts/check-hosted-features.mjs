// Standalone installs and real browser rendering for optional hosted features.
import assert from 'node:assert/strict';
import { execFileSync, spawn, spawnSync } from 'node:child_process';
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import net from 'node:net';
import { chromium, expect } from '@playwright/test';
const zel=path.resolve(`cli/target/debug/zel${process.platform==='win32'?'.exe':''}`);
const root=await mkdtemp(path.join(tmpdir(),'zellij-hosted-features-'));
const app=path.join(root,'app'); const engine=path.join(root,'engine');
const env={...process.env,NEXT_TELEMETRY_DISABLED:'1'};
for(const name of ['ZELLIJ_OUTPUT','ZELLIJ_BASE_PATH','ZELLIJ_MATHS','NEXT_PUBLIC_ZELLIJ_DIAGRAMS']) delete env[name];
const packages=['mermaid','remark-math','rehype-mathjax'];
let server; let serverDone; let browser;
const run=(file,args,cwd=app)=>execFileSync(file,args,{cwd,env,stdio:'inherit',timeout:600000});
const npm=(args)=>process.platform==='win32'
  ? run(process.execPath,[path.join(path.dirname(process.execPath),'node_modules/npm/bin/npm-cli.js'),...args])
  : run('npm',args);
async function stop() { if(server && server.exitCode===null && server.signalCode===null) { server.kill('SIGTERM'); await serverDone; } server=undefined; }
async function verify(enabled,bundler) {
  await rm(path.join(app,'.next'),{recursive:true,force:true});
  npm(['run','build',...(bundler ? ['--',bundler] : [])]);
  npm(['run','typecheck']);
  const listener=net.createServer(); await new Promise(resolve=>listener.listen(0,'127.0.0.1',resolve));
  const port=listener.address().port; await new Promise(resolve=>listener.close(resolve));
  server=spawn(process.execPath,[path.join(app,'node_modules/next/dist/bin/next'),'start','--hostname','127.0.0.1','--port',String(port)],{cwd:app,env,stdio:'inherit'});
  serverDone=new Promise((resolve,reject)=>{server.once('exit',resolve);server.once('error',reject);});
  const origin=`http://127.0.0.1:${port}`;
  let ready=false;
  for(let i=0;i<120;i++) { try { ready=(await fetch(`${origin}/guide`,{signal:AbortSignal.timeout(1000)})).ok; } catch {} if(ready) break; await new Promise(resolve=>setTimeout(resolve,250)); }
  assert(ready,'feature fixture server did not start');
  const page=await browser.newPage(); const errors=[];
  page.on('pageerror',error=>errors.push(error.message));
  await page.goto(`${origin}/guide`);
  await expect(page.getByRole('heading',{name:'Feature selection fixture',exact:true})).toBeVisible();
  if(enabled) {
    await expect(page.locator('mjx-container')).toHaveCount(1);
    await expect(page.locator('figure.zellij-mermaid[data-status="drawn"]')).toHaveCount(1);
    await expect(page.locator('figure.zellij-mermaid svg')).toHaveCount(1);
  } else {
    await expect(page.locator('mjx-container')).toHaveCount(0);
    await expect(page.locator('article')).toContainText('$x^2$');
    await expect(page.locator('figure.zellij-mermaid')).toHaveCount(0);
    await expect(page.locator('figure.zellij-mermaid svg')).toHaveCount(0);
    await expect(page.locator('article')).toContainText('graph TD; A-->B;');
  }
  await expect(page.locator('figure.zellij-mindmap svg')).toHaveCount(1);
  assert.deepEqual(errors,[]);
  await page.close(); await stop();
}
try {
  await mkdir(engine,{recursive:true});
  const packageRoot=path.resolve('engine/packages/zellij');
  const manifest=JSON.parse(await readFile(path.join(packageRoot,'package.json'),'utf8'));
  for(const file of ['package.json',...manifest.files]) {
    await mkdir(path.dirname(path.join(engine,file)),{recursive:true});
    await cp(path.join(packageRoot,file),path.join(engine,file),{recursive:true});
  }
  run(zel,['new','app',app,'--with-docs','--engine',engine],root);
  const content=path.join(app,'docs/zellij/content');
  await rm(content,{recursive:true});
  await mkdir(path.join(content,'guide'),{recursive:true});
  await writeFile(path.join(content,'site.yaml'),'name: Feature fixture\nhome: guide\n');
  await writeFile(path.join(content,'guide/_sidebar.yaml'),'title: Guide\nnav: [index]\n');
  await writeFile(path.join(content,'guide/index.mdx'),`# Feature selection fixture

$x^2$

\`\`\`mermaid
graph TD; A-->B;
\`\`\`

\`\`\`mindmap
Root:
  Child: []
\`\`\`
`);
  npm(['install','--no-audit','--no-fund']);
  // Node caches package lookup results; use a fresh process after each npm install.
  const checkPackages=(installed)=>run(process.execPath,['--input-type=commonjs','-e',`
    const assert=require('node:assert/strict');
    for(const name of ${JSON.stringify(packages)}) {
      ${installed ? 'assert(require.resolve(name));' : "assert.throws(()=>require.resolve(name),{code:'MODULE_NOT_FOUND'});"}
    }
  `]);
  checkPackages(false);
  browser=await chromium.launch();
  await verify(false);
  run(zel,['features','all']);
  // Fail early with guidance instead of silently dropping an enabled feature.
  const missing=spawnSync(process.execPath,['--input-type=module','-e',"import {withZellij} from '@shebka/zellij/next-config'; withZellij();"],{cwd:app,env,encoding:'utf8'});
  assert.notEqual(missing.status,0); assert.match(missing.stderr,/packages are missing.*npm install/);
  npm(['install','--no-audit','--no-fund']);
  checkPackages(true);
  await verify(true);
  run(zel,['features','none']);
  // Deliberately retain all packages: disabling rendering must be independent
  // of npm pruning or other consumers of these optional peers.
  checkPackages(true);
  await verify(false,'--webpack');
  console.log('Hosted features: fresh none omits optional peers; enabled maths/Mermaid render; none disables installed packages; native mindmaps survive; Turbopack/Webpack and typechecks passed.');
} finally { await stop(); if(browser) await browser.close(); await rm(root,{recursive:true,force:true,maxRetries:5,retryDelay:200}); }
