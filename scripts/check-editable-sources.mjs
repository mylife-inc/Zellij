// Real source archive acceptance; the CLI's PATH contains no build/runtime tools.
import assert from 'node:assert/strict';
import {spawn,execFileSync} from 'node:child_process';
import {createServer} from 'node:http';
import net from 'node:net';
import {mkdtemp,mkdir,readFile,writeFile,rm,rename,readdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
const windows=process.platform==='win32';
const binary=path.resolve(process.argv[2]??`cli/target/debug/zel${windows?'.exe':''}`);
const version=execFileSync(binary,['--version'],{encoding:'utf8'}).trim().replace('zel ','');
const root=await mkdtemp(path.join(tmpdir(),'zel-source-package-'));
const resources=path.join(root,'resources 日本');const empty=path.join(root,'no-tools');await mkdir(empty);
const env={...process.env,PATH:empty,ZELLIJ_HOME:resources,DOCKER_HOST:'unusable-fixture',NO_COLOR:'1'};
for(const key of Object.keys(env))if(['PATH','ZELLIJ_IMAGE','ZEL_SOURCE_BASE_URL','NODE_OPTIONS','NODE_PATH'].includes(key.toUpperCase()))delete env[key];
env.PATH=empty;
let requests=[];let mode='normal';let stalled=false;let server;let realBytes;
const published=process.env.ZEL_TEST_SOURCE_BASE_URL;
if(published)env.ZEL_SOURCE_BASE_URL=published;
else {
 const directory=path.resolve(process.env.ZEL_SOURCE_PACKAGES??'dist/source-packages');
 const name=`zel-sources-v${version}.tar.gz`;
 realBytes=await readFile(path.join(directory,name));
 const checksum=await readFile(path.join(directory,name+'.sha256'));
 server=createServer((req,res)=>{
  requests.push(req.url);
  if(mode==='stall'){stalled=true;return;}
  if(![name,name+'.sha256'].includes(req.url.slice(1))){res.writeHead(404).end();return;}
  if(req.url.endsWith('.sha256')){res.end(mode==='checksum'?` ${'0'.repeat(64)}  ${name}\n`:checksum);return;}
  if(mode==='truncated'){res.writeHead(200,{'content-length':realBytes.length});res.write(realBytes.subarray(0,1000));setTimeout(()=>res.destroy(),30);return;}
  res.end(realBytes);
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 env.ZEL_SOURCE_BASE_URL=`http://127.0.0.1:${server.address().port}`;
}
function command(args,extra={},options={}) {
 const child=spawn(binary,args,{cwd:root,env:{...env,...extra},stdio:['pipe','pipe','pipe'],...options});
 let stdout='',stderr='';child.stdout.on('data',b=>stdout+=b);child.stderr.on('data',b=>stderr+=b);
 const done=new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',(code,signal)=>resolve({code,signal,stdout,stderr}));});
 const timer=setTimeout(()=>child.kill('SIGKILL'),180000);done.finally(()=>clearTimeout(timer));
 return {child,done};
}
async function run(args,code=0,extra={},input='') {
 const {child,done}=command(args,extra);child.stdin.end(input);const result=await done;
 assert.equal(result.code,code,JSON.stringify({args,...result}));return result;
}
async function missing(file){await assert.rejects(readFile(file),e=>e.code==='ENOENT');}
async function cleanDownloads(home){
 try{assert(!(await readdir(path.join(home,'cache/sources'))).some(name=>name.startsWith('.download-')));}catch(e){if(e.code!=='ENOENT')throw e;}
}
try{
 await run(['eject','--with-docs','--out','missing','--offline'],2);await missing(path.join(root,'missing/package.json'));assert.equal(requests.length,0);
 await run(['eject','--with-docs','--out','ejected']);
 if(!published)assert.equal(requests.length,2,'Cold package uses archive + checksum only');
 const cache=path.join(resources,'cache/sources',version);
 const original=await readFile(path.join(cache,'engine/src/index.ts'));
 await rename(path.join(root,'ejected'),path.join(root,'Moved 日本 #1%'));
 const ejected=path.join(root,'Moved 日本 #1%');
 const manifest=JSON.parse(await readFile(path.join(ejected,'package.json'),'utf8'));
 assert.equal(manifest.dependencies['@shebka/zellij'],'file:vendor/zellij');
 assert((await readFile(path.join(ejected,'docs/zellij/content/site.yaml'),'utf8')).length>0);
 await run(['features','all','--site-root',ejected]);
 assert.equal(JSON.parse(await readFile(path.join(ejected,'package.json'),'utf8')).zellij.features.length,2);
 await writeFile(path.join(ejected,'vendor/zellij/src/index.ts'),'editable independent copy');
 assert.deepEqual(await readFile(path.join(cache,'engine/src/index.ts')),original);
 // Both interfaces share this cache. A fresh git repository retains its metadata.
 await mkdir(path.join(root,'app/.git'),{recursive:true});await writeFile(path.join(root,'app/.git/config'),'authored metadata');
 await run(['new','app','app','--with-docs','--offline','--name','Source package acceptance','--features','all']);
 assert.equal(await readFile(path.join(root,'app/.git/config'),'utf8'),'authored metadata');
 assert(JSON.parse(await readFile(path.join(root,'app/tsconfig.json'),'utf8')).exclude.includes('vendor/zellij'));
 assert.deepEqual(await readFile(path.join(root,'app/vendor/zellij/src/index.ts')),original);
 await run(['na','wizard','--with-docs','--offline','-i'],0,{},'\nWizard\n4127\nnone\n');
 await run(['eject',path.join(root,'app/docs/zellij'),'--out','external','--offline']);
 assert((await readFile(path.join(root,'external/zellij.config.yaml'),'utf8')).includes('app'));
 await missing(path.join(root,'external/content/site.yaml'));
 await run(['na','reference','--site-root',path.join(root,'app/docs/zellij'),'--offline']);
 await run(['eject','--out','app','--with-docs'],2);
 await run(['na',path.join(resources,'cache/sources/overlap'),'--with-docs'],2);
 await run(['eject','--out',path.join(resources,'cache/sources/overlap'),'--with-docs'],2);
 await run(['eject','--out','image','--with-docs','--offline'],2,{ZELLIJ_IMAGE:'explicit:selection'});
 const saved=await readFile(path.join(root,'app/package.json'));
 await run(['na','app','--with-docs'],2);assert.deepEqual(await readFile(path.join(root,'app/package.json')),saved);
 await writeFile(path.join(cache,'engine/src/index.ts'),'damaged');
 await run(['na','damaged','--with-docs','--offline'],2);await missing(path.join(root,'damaged/package.json'));
 await writeFile(path.join(cache,'engine/src/index.ts'),original);
 await writeFile(path.join(cache,'engine/unlisted.js'),'extra');
 await run(['eject','--out','unlisted','--with-docs','--offline'],2);await rm(path.join(cache,'engine/unlisted.js'));
 if(!published){
  assert.equal(requests.length,2,'Offline reuse and preflight failures must not download');
  for(const failure of ['checksum','truncated']){
   mode=failure;const home=path.join(root,`bad-${failure}`);
   await run(['eject','--out',`bad-out-${failure}`,'--with-docs'],2,{ZELLIJ_HOME:home});
   await missing(path.join(home,'cache/sources',version,'manifest.json'));await cleanDownloads(home);
   await missing(path.join(root,`bad-out-${failure}/package.json`));
  }
  mode='stall';const home=path.join(root,'cancelled');
  let child,done;
  if(windows){
   const console=path.join(root,'console.exe');execFileSync('rustc',['--edition=2024','scripts/fixtures/preview-console.rs','-o',console]);
   child=spawn(console,[binary,'eject','--with-docs','--out','cancelled-output'],{cwd:root,env:{...env,ZELLIJ_HOME:home},stdio:['pipe','pipe','pipe']});
   let output='';child.stdout.on('data',b=>output+=b);child.stderr.on('data',b=>output+=b);
   done=new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',(code,signal)=>resolve({code,signal,stderr:output}));});
  }else({child,done}=command(['eject','--with-docs','--out','cancelled-output'],{ZELLIJ_HOME:home}));
  for(let n=0;n<100&&!stalled;n++)await delay(50);assert(stalled,'Cancellation fixture never received the request');
  if(windows)child.stdin.write('stop\n');else child.kill('SIGINT');
  const timer=setTimeout(()=>child.kill('SIGKILL'),10000);
  try{const result=await done;assert.equal(result.code,2,JSON.stringify(result));}finally{clearTimeout(timer);}
  await cleanDownloads(home);await missing(path.join(root,'cancelled-output/package.json'));
  mode='normal';
 }
 await cleanDownloads(resources);
 if(process.env.ZEL_SOURCE_BUILD==='true'){
  // This is maintainer acceptance, after proving generation with an empty PATH.
  // Running an editable Next app still requires its documented Node/npm tools.
  const app=path.join(root,'app');
  const buildEnv={...process.env,NEXT_TELEMETRY_DISABLED:'1',ZELLIJ_OUTPUT:''};delete buildEnv.ZELLIJ_BASE_PATH;delete buildEnv.ZELLIJ_IMAGE;
  const site=path.join(app,'docs/zellij/content/site.yaml');
  await writeFile(site,(await readFile(site,'utf8'))+'\nbasePath: /sources\n');
  const npm=windows?path.join(path.dirname(process.execPath),'node_modules/npm/bin/npm-cli.js'):null;
  for(const args of [['install','--no-audit','--no-fund'],['run','check','--','--strict','--deep'],['run','typecheck'],['run','build','--','--webpack']]){
   execFileSync(windows?process.execPath:'npm',windows?[npm,...args]:args,{cwd:app,env:buildEnv,stdio:'inherit',timeout:600000});
  }
  const listener=net.createServer();await new Promise(resolve=>listener.listen(0,'127.0.0.1',resolve));
  const port=listener.address().port;await new Promise(resolve=>listener.close(resolve));
  const child=spawn(process.execPath,[path.join(app,'node_modules/next/dist/bin/next'),'start','--port',String(port),'--hostname','127.0.0.1'],{cwd:app,env:buildEnv,stdio:['ignore','pipe','pipe']});
  let output='';child.stdout.on('data',b=>output+=b);child.stderr.on('data',b=>output+=b);
  const exited=new Promise(resolve=>child.once('exit',resolve));
  try {
   const origin=`http://127.0.0.1:${port}`;let ready=false;
   for(let n=0;n<120;n++){
    try{const response=await fetch(origin+'/sources',{signal:AbortSignal.timeout(1000)});if(response.ok){assert.match(await response.text(),/Source package acceptance/);ready=true;break;}}catch{}
    if(child.exitCode!==null)throw new Error('Wrapper server exited: '+output);await delay(250);
   }
   assert(ready,'Wrapper server did not become ready: '+output);
   assert.equal((await fetch(origin+'/sources/guide')).status,200);
   assert.equal((await fetch(origin+'/sources/missing')).status,404);
   const search=await (await fetch(origin+'/sources/_zellij/search-index.json')).json();assert(search.length>0);
  } finally {if(child.exitCode===null)child.kill('SIGTERM');await exited;}
  assert.deepEqual(await readFile(path.join(cache,'engine/src/index.ts')),original,'Installing the wrapper mutated the source cache');
 }
 console.log(JSON.stringify({editableSources:true,version,toolFreeGeneration:true,offlineReuse:true,copiedEngine:true,gitMetadataPreserved:true,cacheIntegrity:true,failedDownloads:!published,cancellation:!published,installedWrapper:process.env.ZEL_SOURCE_BUILD==='true'}));
}finally{
 if(server){server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
 await rm(root,{recursive:true,force:true,maxRetries:5,retryDelay:200});
}
