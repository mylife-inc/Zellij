// Real toolchain acceptance runs only on hosted native Linux runners.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync, spawn } from 'node:child_process';
import { mkdtempSync, cpSync, readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
const binary=path.resolve(process.argv[2] || 'cli/target/debug/zel');
const root=mkdtempSync(path.join(tmpdir(),'zel-pdf-'));
const site=path.join(root,'book');
const artifacts=path.resolve('test-results/pdf'); mkdirSync(artifacts,{recursive:true});
const run=(args)=>execFileSync(binary,args,{encoding:'utf8',stdio:['ignore','pipe','pipe']});
const waitFor = async (condition, label) => {
  const end=Date.now()+180_000;
  while(!await condition()) { assert(Date.now()<end,`Timed out: ${label}`); await new Promise(r=>setTimeout(r,200)); }
};
try {
  cpSync('scripts/fixtures/pdf-book',site,{recursive:true});
  run(['pdf',site,'--check']);
  const config=readFileSync(path.join(site,'zel.yaml'),'utf8');
  writeFileSync(path.join(site,'zel.yaml'),config+'\npdf: {enabled: true}\n');
  run(['build',site]);
  const output=path.join(site,'site'); const printed=path.join(output,'_zellij/pdf');
  const receipt=JSON.parse(readFileSync(path.join(printed,'pdf.json')));
  const book=JSON.parse(readFileSync(path.join(output,'_zellij/book.json')));
  assert.equal(receipt.revision,book.revision);
  assert.equal(receipt.pdf_sha256,createHash('sha256').update(readFileSync(path.join(printed,'book.pdf'))).digest('hex'));
  for(const page of ['index.html','principles.html','practice.html','reference.html']) {
    assert.match(readFileSync(path.join(output,page),'utf8'),/href="_zellij\/pdf\/book.pdf" download/);
  }
  const pdf=path.join(printed,'book.pdf');
  const info=execFileSync('pdfinfo',[pdf],{encoding:'utf8'});
  assert.match(info,/Title:\s+A small atlas of proofs/); assert.match(info,/Author:\s+Mira Vale/);
  const pages=Number(info.match(/Pages:\s+(\d+)/)[1]); assert(pages>=7);
  const text=execFileSync('pdftotext',['-layout',pdf,'-'],{encoding:'utf8'});
  for(const required of ['A small atlas of proofs','Contents','Before the first proof','Foundations','Principles','Practice','Reference','A proof establishes every case','\\end{document}','Direct','Induction']) assert(text.includes(required),`Missing PDF text: ${required}`);
  assert(text.indexOf('Principles',text.indexOf('Contents')+8)<text.lastIndexOf('Practice'));
  assert(text.split('\f').some(page=>page.includes('A short comparison')&&page.includes('Direct')&&page.includes('Induction')),'Table heading and first rows stay together');
  const fonts=execFileSync('pdffonts',[pdf],{encoding:'utf8'}); assert.match(fonts,/yes\s+yes\s+yes/);
  execFileSync('qpdf',['--check',pdf]);
  const object=JSON.parse(execFileSync('qpdf',['--json',pdf],{encoding:'utf8',maxBuffer:8*1024*1024}));
  assert(JSON.stringify(object).includes('/GoTo'),'Internal PDF links/bookmarks survive');
  for(let n=1;n<=pages;n++) execFileSync('pdftoppm',['-f',String(n),'-singlefile','-scale-to','1300','-png',pdf,path.join(artifacts,`page-${n}`)]);
  cpSync(printed,path.join(artifacts,'publication'),{recursive:true});
  writeFileSync(path.join(artifacts,'pdfinfo.txt'),info); writeFileSync(path.join(artifacts,'text.txt'),text); writeFileSync(path.join(artifacts,'fonts.txt'),fonts);
  const first=readFileSync(pdf); const home=readFileSync(path.join(output,'index.html'));
  writeFileSync(path.join(site,'content/practice.md'),'# Practice\n\n`'+'W'.repeat(160)+'`');
  const failed=spawnSync(binary,['build',site,'--pdf'],{encoding:'utf8'});
  assert.notEqual(failed.status,0,'Overfull print content must fail');
  assert.match(failed.stderr,/Overfull/); assert.deepEqual(readFileSync(pdf),first); assert.deepEqual(readFileSync(path.join(output,'index.html')),home);
  cpSync('scripts/fixtures/pdf-book/content/practice.md',path.join(site,'content/practice.md'));
  run(['export','pdf',site]);
  assert.deepEqual(readFileSync(path.join(output,'index.html')),home,'Standalone export leaves website alone');
  assert.equal(JSON.parse(readFileSync(path.join(site,'pdf/pdf.json'))).revision,book.revision);
  run(['build',site,'--no-pdf']);
  assert(!readFileSync(path.join(output,'index.html'),'utf8').includes('Download the complete book'));
  assert(!readdirSync(path.join(output,'_zellij')).includes('pdf'),'Plain rebuild removes old PDF/link together');
  for (const [paper,size,dimensions] of [['a4',12,/595\.\d+ x 841\.\d+/],['letter',10,/612 x 792/]]) {
    writeFileSync(path.join(site,'zel.yaml'),config+`\npdf: {enabled: false, paper: ${paper}, font-size: ${size}}\n`);
    const custom=path.join(root,paper);
    run(['pdf',site,'--out',custom]);
    const customPdf=path.join(custom,'book.pdf');
    const metadata=execFileSync('pdfinfo',[customPdf],{encoding:'utf8'});
    assert.match(metadata,dimensions); execFileSync('qpdf',['--check',customPdf]);
    const settings=JSON.parse(readFileSync(path.join(custom,'pdf.json'))).settings;
    assert.equal(settings.paper,paper); assert.equal(settings['font-size'],size);
    cpSync(custom,path.join(artifacts,paper),{recursive:true});
    writeFileSync(path.join(artifacts,paper,'pdfinfo.txt'),metadata);
    const count=Number(metadata.match(/Pages:\s+(\d+)/)[1]);
    for(let n=1;n<=count;n++) execFileSync('pdftoppm',['-f',String(n),'-singlefile','-scale-to','1300','-png',customPdf,path.join(artifacts,paper,`page-${n}`)]);
  }
  writeFileSync(path.join(site,'zel.yaml'),config+'\npdf: {enabled: true}\n');
  const preview=spawn(binary,['view',site,'--watch','--no-open','--port','0'],{stdio:['ignore','pipe','pipe']});
  let previewLog=''; preview.stdout.on('data',chunk=>previewLog+=chunk); preview.stderr.on('data',chunk=>previewLog+=chunk);
  const closed=new Promise(resolve=>preview.once('close',(code,signal)=>resolve({code,signal})));
  try {
    await waitFor(()=>/Serving at (http:\/\/[^\s]+)/.test(previewLog),'PDF preview startup');
    const origin=previewLog.match(/Serving at (http:\/\/[^\s]+)/)[1].replace(/\/$/,'');
    const get=async p=> {const r=await fetch(origin+p,{signal:AbortSignal.timeout(5000)}); assert.equal(r.status,200,p); return r;};
    const initial=await (await get('/_zellij/book.json')).json();
    assert.equal((await (await get('/_zellij/pdf/pdf.json')).json()).revision,initial.revision);
    assert(Buffer.from(await (await get('/_zellij/pdf/book.pdf')).arrayBuffer()).subarray(0,5).equals(Buffer.from('%PDF-')));
    const manuscript=readFileSync(path.join(site,'content/practice.md'),'utf8');
    writeFileSync(path.join(site,'content/practice.md'),manuscript+'\nA revised closing observation.\n');
    await waitFor(async()=> (await (await get('/_zellij/book.json')).json()).revision!==initial.revision,'PDF watched rebuild');
    const latest=await (await get('/_zellij/book.json')).json();
    assert.equal((await (await get('/_zellij/pdf/pdf.json')).json()).revision,latest.revision);
    assert.match(await (await get('/practice.html')).text(),/A revised closing observation/);
    writeFileSync(path.join(artifacts,'preview.json'),JSON.stringify({initial:initial.revision,rebuilt:latest.revision,pdfMatchesWeb:true},null,2));
  } finally {
    preview.kill('SIGINT');
    const status=await Promise.race([closed,new Promise(resolve=>{const timer=setTimeout(()=>resolve(null),30_000);timer.unref();})]);
    if(!status) preview.kill('SIGKILL');
    writeFileSync(path.join(artifacts,'preview.log'),previewLog);
    assert(status && status.code===0,`PDF preview failed to stop: ${JSON.stringify(status)}\n${previewLog}`);
  }
  if(!process.env.CONTAINER_HOST) {
    const tag=`zel-pdf-static-${process.pid}:acceptance`, name=`zel-pdf-static-${process.pid}`;
    const docker=args=>execFileSync('docker',args,{encoding:'utf8',timeout:180_000}).trim();
    try {
      run(['si',site,'--tag',tag]);
      docker(['run','-d','--name',name,'--read-only','--tmpfs','/tmp','-p','127.0.0.1::3000',tag]);
      const container=JSON.parse(docker(['inspect',name]))[0];
      const origin=`http://127.0.0.1:${container.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
      await waitFor(async()=> {try{return (await fetch(origin+'/index.html')).ok;}catch{return false;}},'PDF static image HTTP');
      const request=async p=>{const r=await fetch(origin+p,{signal:AbortSignal.timeout(5000)});assert.equal(r.status,200,p);return r;};
      const publication=await (await request('/_zellij/book.json')).json();
      const receipt=await (await request('/_zellij/pdf/pdf.json')).json();
      const bytes=Buffer.from(await (await request('/_zellij/pdf/book.pdf')).arrayBuffer());
      assert.equal(receipt.revision,publication.revision);assert.equal(receipt.pdf_sha256,createHash('sha256').update(bytes).digest('hex'));
      assert.equal(container.Config.User,'nginx');
      const absent=docker(['exec',name,'sh','-c','! command -v lualatex && ! command -v node && ! command -v cargo']); assert.equal(absent,'');
      const image=JSON.parse(docker(['image','inspect',tag]))[0];
      writeFileSync(path.join(artifacts,'static-image.json'),JSON.stringify({architecture:image.Architecture,bytes:image.Size,pdfBytes:bytes.length,revision:publication.revision,user:container.Config.User,latexAbsent:true},null,2));
    } finally { spawnSync('docker',['rm','-f',name]); spawnSync('docker',['image','rm','-f',tag]); }
  }
  console.log('PDF: actual LuaLaTeX, text/order/math, links, fonts, valid PDF, source identity, failed-build preservation and standalone/web-only behavior, configured A4/Letter, live PDF preview and configured static-output behavior passed.');
} finally { rmSync(root,{recursive:true,force:true}); }
