// Runs against public release URLs with the installed, released binary.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
export async function verifyPublishedTemplates(binary, base, root) {
  const env={...process.env}; delete env.GITHUB_TOKEN; delete env.GH_TOKEN;
  const run=args=>execFileSync(binary,args,{env,encoding:'utf8',stdio:'pipe',timeout:180000});
  const bundled=JSON.parse(run(['templates','--json','--offline']));
  const options=['--catalog',`${base}/templates-catalog.json`,'--cache-dir',path.join(root,'cache')];
  const published=JSON.parse(run(['templates','--json',...options]));
  assert.deepEqual(published,bundled,'Published catalog must describe the templates in the released CLI');
  const records=[];
  for(const template of bundled.templates){
    for(const kind of ['bundled','published','offline'])run(['create',path.join(root,kind,template.id),'-t',`${template.id}@${template.version}`,
      ...(kind==='bundled'?['--offline']:kind==='offline'?[...options,'--offline']:options)]);
    const receipts=await Promise.all(['bundled','published','offline'].map(async kind=>JSON.parse(await readFile(path.join(root,kind,template.id,'.zel-template.json'),'utf8'))));
    assert.deepEqual(receipts[0].files,receipts[1].files);assert.deepEqual(receipts[1].files,receipts[2].files);
    if(template.print==='lualatex'){
      const site=path.join(root,'offline',template.id);
      run(['pdf',site,'--check']);
      run(['build',site,'--no-pdf']);
      const publication=JSON.parse(await readFile(path.join(site,'site/_zellij/book.json'),'utf8'));
      assert(publication.entries.length>=3,'Published print template has an actual manuscript');
      assert.equal(publication.manifest.title,template.variables.title);
    }
    records.push({template:template.id,version:template.version,archive_sha256:receipts[1].source_sha256,files:receipts[1].files});
  }
  return records;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 5, 'Usage: check-published-templates.mjs <zel> <release-base> <new-directory>');
  console.log(JSON.stringify(await verifyPublishedTemplates(...process.argv.slice(2))));
}
