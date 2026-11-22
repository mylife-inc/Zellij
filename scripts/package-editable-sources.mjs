// Maintainer packaging only. Installed zel extracts this with its native reader.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {cp, lstat, mkdir, mkdtemp, readFile, readdir, rm, writeFile} from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
const repo=path.resolve(import.meta.dirname,'..');
const version=(await readFile(path.join(repo,'cli/Cargo.toml'),'utf8')).match(/^version = "([^"]+)"/m)[1];
const out=path.resolve(process.argv[2] ?? 'dist/source-packages');
const engine=path.join(repo,'engine/packages/zellij');
const stage=await mkdtemp(path.join(os.tmpdir(),'zel-editable-package-'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
try {
 const root=path.join(stage,'package');await mkdir(root);await mkdir(out,{recursive:true});
 // npm's published-file inventory includes sources, built entry points and fonts;
 // --ignore-scripts prevents package lifecycle execution during this inventory.
 const [packed]=JSON.parse(execFileSync('npm',['pack','--dry-run','--ignore-scripts','--json'],{cwd:engine,encoding:'utf8'}));
 for(const {path:relative} of packed.files) {
  assert(!relative.split('/').some(p=>['..','node_modules','.git','.next'].includes(p)) && !path.isAbsolute(relative));
  assert((await lstat(path.join(engine,relative))).isFile(),`Not a regular package file: ${relative}`);
  const dest=path.join(root,'engine',relative);await mkdir(path.dirname(dest),{recursive:true});await cp(path.join(engine,relative),dest);
 }
 // Share the same canonical initializer used by base images; nothing is installed.
 execFileSync(process.execPath,[path.join(engine,'bin/zellij.mjs'),'init',path.join(root,'app'),'--name','zellij-site','--features','none'],{cwd:repo,stdio:'inherit'});
 const packageFile=path.join(root,'app/package.json');
 const manifest=JSON.parse(await readFile(packageFile,'utf8'));
 manifest.dependencies['@shebka/zellij']='file:/opt/zellij';
 await writeFile(packageFile,JSON.stringify(manifest,null,2)+'\n');
 const notices=await readFile(path.join(repo,'dist/standalone-engine/SERVER-THIRD-PARTY-NOTICES.txt'),'utf8');
 assert(notices.includes('zod@') && notices.includes('yaml@'),'Build standalone license inventory before packaging editable sources');
 await writeFile(path.join(root,'engine/BUNDLED-THIRD-PARTY-NOTICES.txt'),notices);
 const files={};
 async function walk(dir) {
  for(const entry of (await readdir(dir,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))) {
   const full=path.join(dir,entry.name);
   if(entry.isDirectory())await walk(full);
   else {assert(entry.isFile(),`Not a regular file: ${full}`);files[path.relative(root,full).split(path.sep).join('/')]=hash(await readFile(full));}
  }
 }
 await walk(root);
 await writeFile(path.join(root,'manifest.json'),JSON.stringify({format:1,kind:'editable-sources',version,files},null,2)+'\n');
 const archive=`zel-sources-v${version}.tar.gz`;
 await writeFile(path.join(stage,'files.txt'),[...Object.keys(files),'manifest.json'].join('\n')+'\n');
 execFileSync('tar',['-czf',path.join(out,archive),'-C',root,'-T',path.join(stage,'files.txt')],{env:{...process.env,COPYFILE_DISABLE:'1'}});
 const bytes=await readFile(path.join(out,archive));
 await writeFile(path.join(out,archive+'.sha256'),`${hash(bytes)}  ${archive}\n`);
 console.log(JSON.stringify({archive,bytes:bytes.length,files:Object.keys(files).length,runtimeIncluded:false}));
}finally{await rm(stage,{recursive:true,force:true});}
