// Cross-platform extraction protocol, failures and Ctrl-C; no local daemon.
import assert from 'node:assert/strict';
import { execFileSync, spawn, spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile, rename } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const windows = process.platform === 'win32';
const zel = path.resolve(`cli/target/debug/zel${windows ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-eject-'));
const bin = path.join(root,'bin');
const fixture = path.join(root,'fixture');
const out = path.join(root,'Editable 日本 #1%');
const env = { ...process.env };
for (const key of Object.keys(env)) if (['PATH','DOCKER_HOST','DOCKER_CONTEXT','CONTAINER_CONNECTION','CONTAINER_HOST','CONTAINERD_ADDRESS','CONTAINERD_NAMESPACE'].includes(key.toUpperCase())) delete env[key];
Object.assign(env, {PATH:bin, CONTAINER_CONNECTION:'eject-fixture', ZEL_EJECT_FIXTURE:fixture, ZEL_EJECT_EXPECT_IMAGE:'selected:fixture', ZELLIJ_IMAGE:'selected:fixture'});
async function put(file,body) { await mkdir(path.dirname(file),{recursive:true}); await writeFile(file,body); }
function run(args,code=0,extra={}) {
  const result = spawnSync(zel,args,{cwd:root,env:{...env,...extra},encoding:'utf8',timeout:30000});
  assert.equal(result.status,code,JSON.stringify({args,...result})); return result;
}
async function clean() {
  assert(!(await readdir(fixture)).includes('container'),'stopped container leaked');
  assert(!(await readdir(root)).some(name=>name.startsWith('zel-eject-')),'staging leaked');
}
try {
  await mkdir(bin,{recursive:true}); await mkdir(fixture,{recursive:true});
  execFileSync('rustc',['--edition=2024','scripts/fixtures/eject-engine.rs','-o',path.join(bin,`podman${windows ? '.exe' : ''}`)],{stdio:'inherit'});
  if (windows) execFileSync('rustc',['--edition=2024','scripts/fixtures/preview-console.rs','-o',path.join(bin,'console.exe')],{stdio:'inherit'});
  const template=JSON.parse(await readFile('cli/crates/zellij-scaffold/generated/app.json','utf8'));
  const manifest=structuredClone(template.manifest); manifest.dependencies['@shebka/zellij']='file:/opt/zellij';
  manifest.name='selected-image'; manifest.dependencies.mermaid='^11.17.2';
  await put(path.join(fixture,'app/package.json'),JSON.stringify(manifest));
  for (const [name,text] of Object.entries(template.files)) await put(path.join(fixture,'app',name),text);
  await put(path.join(fixture,'app/src/app/layout.tsx'),'// selected image wrapper\n');
  await put(path.join(fixture,'app/content/site.yaml'),'name: Starter\n');
  await put(path.join(fixture,'engine/package.json'),JSON.stringify({name:'@shebka/zellij',version:template.engineVersion,zellij:{wrapperConfigVersion:1}}));
  for(const name of ['bin/zellij.mjs','next-config.mjs','deployment-config.cjs','src/index.ts','dist/cli/check-bundle.js','dist/cli/dev-bundle.js','dist/cli/init-bundle.js','dist/cli/sync-assets-bundle.js','dist/cli/write-search-index-bundle.js']) await put(path.join(fixture,'engine',name),'engine fixture');
  await put(path.join(fixture,'keep'),'authored');
  await put(path.join(root,'docs/zellij/zel.yaml'),'version: 1\nrenderer: hosted\n');
  await put(path.join(root,'docs/zellij/content/site.yaml'),'name: External\n');
  run(['eject','--out',out]); await clean();
  assert.equal(await readFile(path.join(out,'src/app/layout.tsx'),'utf8'),'// selected image wrapper\n');
  const pkg=JSON.parse(await readFile(path.join(out,'package.json'),'utf8'));
  assert(JSON.parse(await readFile(path.join(out,'tsconfig.json'),'utf8')).exclude.includes('vendor/zellij'));
  assert.equal(pkg.dependencies['@shebka/zellij'],'file:vendor/zellij'); assert.equal(pkg.dependencies.mermaid,'^11.17.2');
  assert((await readFile(path.join(out,'zellij.config.yaml'),'utf8')).includes('content:'));
  run(['features','maths','--site-root',out]);
  assert.equal(JSON.parse(await readFile(path.join(out,'package.json'),'utf8')).dependencies.mermaid,undefined);
  const before=await readFile(path.join(out,'package.json'));
  run(['eject','--out',out],2); assert.deepEqual(await readFile(path.join(out,'package.json')),before);
  await rm(out,{recursive:true});
  const override='registry.example/engine@sha256:'+'a'.repeat(64);
  run(['eject','--out',out,'--with-docs','--image',override],0,{ZEL_EJECT_EXPECT_IMAGE:override}); await clean();
  await rename(out,`${out} moved`);
  run(['features','all','--site-root',`${out} moved`]);
  assert.equal(await readFile(path.join(`${out} moved`,'docs/zellij/content/site.yaml'),'utf8'),'name: Starter\n');
  await rm(`${out} moved`,{recursive:true});
  for (const mode of ['fail',...(windows ? [] : ['link'])]) {
    run(['eject','--out',out,'--with-docs'],mode==='link'?2:3,{ZEL_EJECT_MODE:mode}); await clean();
    assert(!(await readdir(root)).includes(path.basename(out)));
  }
  await put(path.join(fixture,'engine/package.json'),'{}');
  run(['eject','--out',out,'--with-docs'],2); await clean();
  // Ctrl-C during the first copy must remove the snapshot and temporary files.
  const args=['eject','--out',out,'--with-docs'];
  const child=spawn(windows ? path.join(bin,'console.exe') : zel,windows ? [zel,...args] : args,{cwd:root,env:{...env,ZEL_EJECT_MODE:'slow'},stdio:['pipe','pipe','pipe']});
  let output=''; child.stdout.on('data',b=>output+=b); child.stderr.on('data',b=>output+=b);
  const done=new Promise((resolve,reject)=>{child.once('exit',(code)=>resolve(code));child.once('error',reject);});
  for(let n=0;n<150 && !(await readdir(fixture)).includes('copying');n++) await delay(100);
  assert((await readdir(fixture)).includes('copying'),output);
  if(windows) child.stdin.write('stop\n'); else child.kill('SIGINT');
  const timeout=setTimeout(()=>child.kill('SIGKILL'),10000);
  try { assert.equal(await done,3,output); } finally { clearTimeout(timeout); }
  await clean();
  assert.equal(await readFile(path.join(fixture,'keep'),'utf8'),'authored');
  const log=await readFile(path.join(fixture,'log'),'utf8');
  assert(!log.split('\n').some(line=>/^(run|start|exec|pull)\|/.test(line)),log);
  console.log('Eject: selected sources, movable engine, features, protected output, copy failure, compatibility, cleanup and Ctrl-C passed.');
} finally { await rm(root,{recursive:true,force:true,maxRetries:5,retryDelay:200}); }
