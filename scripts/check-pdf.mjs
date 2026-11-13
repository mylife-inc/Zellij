// Real toolchain acceptance runs only on hosted native Linux runners.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, cpSync, readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
const binary=path.resolve(process.argv[2] || 'cli/target/debug/zel');
const root=mkdtempSync(path.join(tmpdir(),'zel-pdf-'));
const site=path.join(root,'book');
const artifacts=path.resolve('test-results/pdf'); mkdirSync(artifacts,{recursive:true});
const run=(args)=>execFileSync(binary,args,{encoding:'utf8',stdio:['ignore','pipe','pipe']});
try {
  cpSync('scripts/fixtures/pdf-book',site,{recursive:true});
  run(['pdf',site,'--check']);
  run(['build',site,'--pdf']);
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
  assert(Number(info.match(/Pages:\s+(\d+)/)[1])>=7);
  const text=execFileSync('pdftotext',['-layout',pdf,'-'],{encoding:'utf8'});
  for(const required of ['A small atlas of proofs','Contents','Before the first proof','Foundations','Principles','Practice','Reference','A proof establishes every case','\\end{document}','Direct','Induction']) assert(text.includes(required),`Missing PDF text: ${required}`);
  assert(text.indexOf('Principles',text.indexOf('Contents')+8)<text.lastIndexOf('Practice'));
  const fonts=execFileSync('pdffonts',[pdf],{encoding:'utf8'}); assert.match(fonts,/yes\s+yes\s+yes/);
  execFileSync('qpdf',['--check',pdf]);
  const object=JSON.parse(execFileSync('qpdf',['--json',pdf],{encoding:'utf8',maxBuffer:8*1024*1024}));
  assert(JSON.stringify(object).includes('/GoTo'),'Internal PDF links/bookmarks survive');
  for(const n of [1,2,4,5,6]) execFileSync('pdftoppm',['-f',String(n),'-singlefile','-scale-to','1300','-png',pdf,path.join(artifacts,`page-${n}`)]);
  cpSync(printed,path.join(artifacts,'publication'),{recursive:true});
  writeFileSync(path.join(artifacts,'pdfinfo.txt'),info); writeFileSync(path.join(artifacts,'text.txt'),text); writeFileSync(path.join(artifacts,'fonts.txt'),fonts);
  const first=readFileSync(pdf); const home=readFileSync(path.join(output,'index.html'));
  writeFileSync(path.join(site,'content/practice.md'),'# Practice\n\n'+'unbreakable'.repeat(80));
  const failed=spawnSync(binary,['build',site,'--pdf'],{encoding:'utf8'});
  assert.notEqual(failed.status,0,'Overfull print content must fail');
  assert.match(failed.stderr,/Overfull/); assert.deepEqual(readFileSync(pdf),first); assert.deepEqual(readFileSync(path.join(output,'index.html')),home);
  cpSync('scripts/fixtures/pdf-book/content/practice.md',path.join(site,'content/practice.md'));
  run(['export','pdf',site]);
  assert.deepEqual(readFileSync(path.join(output,'index.html')),home,'Standalone export leaves website alone');
  assert.equal(JSON.parse(readFileSync(path.join(site,'pdf/pdf.json'))).revision,book.revision);
  run(['build',site]);
  assert(!readFileSync(path.join(output,'index.html'),'utf8').includes('Download the complete book'));
  assert(!readdirSync(path.join(output,'_zellij')).includes('pdf'),'Plain rebuild removes old PDF/link together');
  console.log('PDF: actual LuaLaTeX, text/order/math, links, fonts, valid PDF, source identity, failed-build preservation and standalone/ordinary-build behavior passed.');
} finally { rmSync(root,{recursive:true,force:true}); }
