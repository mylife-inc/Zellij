// Immutable, reproducible template assets. Uses only Node and Python standard libraries.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstat, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { releasePolicy } from './release-policy.mjs';
const repo = fileURLToPath(new URL('../', import.meta.url));
const sha = b => createHash('sha256').update(b).digest('hex');
const python = String.raw`
import sys, json, io, gzip, tarfile, pathlib
request=json.load(sys.stdin); root=pathlib.Path(request['root']); output=io.BytesIO()
with gzip.GzipFile(fileobj=output,mode='wb',filename='',mtime=0) as compressed:
 with tarfile.open(fileobj=compressed,mode='w',format=tarfile.USTAR_FORMAT) as archive:
  for name in request['files']:
   data=(root/name).read_bytes(); info=tarfile.TarInfo(name)
   info.size=len(data); info.mode=0o644; info.mtime=0
   archive.addfile(info,io.BytesIO(data))
sys.stdout.buffer.write(output.getvalue())
`;
async function files(root, dir = root) {
  const result = [];
  for (const name of (await readdir(dir)).sort()) {
    const file = path.join(dir,name), stat = await lstat(file);
    assert(!stat.isSymbolicLink(), 'Template source cannot contain links');
    if (stat.isDirectory()) result.push(...await files(root,file));
    else { assert(stat.isFile(), 'Template source must contain only regular files'); result.push(path.relative(root,file).split(path.sep).join('/')); }
  }
  return result.sort();
}
export async function packageTemplates(binary, tag, destination) {
  const { version } = releasePolicy(tag), zel = path.resolve(binary);
  const invoke = args => execFileSync(zel,args,{encoding:'utf8',timeout:30000,stdio:'pipe'});
  assert.equal(invoke(['--version']).trim(), `zel ${version}`, 'Template packager needs the matching CLI build');
  const catalog = JSON.parse(invoke(['templates','--json','--offline']));
  assert.equal(catalog.schema_version,1); assert(catalog.templates.length > 0);
  const temp = await mkdtemp(path.join(tmpdir(),'zel-template-package-'));
  const assets = new Map(), templates = [];
  try {
    for (const manifest of catalog.templates) {
      assert(/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(manifest.id));
      releasePolicy(`v${manifest.version}`);
      const source = path.join(repo,'templates',manifest.id);
      for (const [kind, selection] of [['bundled',manifest.id],['local',source]]) invoke(['create',path.join(temp,kind,manifest.id),'-t',selection,'--offline']);
      const receipts = await Promise.all(['bundled','local'].map(async kind => JSON.parse(await readFile(path.join(temp,kind,manifest.id,'.zel-template.json'),'utf8'))));
      assert.equal(receipts[0].source_sha256,receipts[1].source_sha256,'Rebuild the CLI: authored template bytes differ from its bundled copy');
      const archive = execFileSync(process.platform==='win32'?'python':'python3',['-X','utf8','-c',python],{input:JSON.stringify({root:source,files:await files(source)}),maxBuffer:16*1024*1024,timeout:30000});
      const name = `template-${manifest.id}-${manifest.version}.tar.gz`;
      assets.set(name,archive); templates.push({manifest,archive:{url:name,sha256:sha(archive)}});
    }
    assets.set('templates-catalog.json',Buffer.from(JSON.stringify({schema:1,release:tag,templates},null,2)+'\n'));
    await mkdir(destination,{recursive:true});
    for (const name of assets.keys()) await assert.rejects(lstat(path.join(destination,name)),{code:'ENOENT'},`Refusing existing template asset ${name}`);
    for (const [name,body] of assets) await writeFile(path.join(destination,name),body,{flag:'wx'});
    return [...assets].map(([name,body])=>({name,size:body.length,sha256:sha(body)}));
  } finally { await rm(temp,{recursive:true,force:true}); }
}
if (process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length,5,'Usage: package-templates.mjs <zel> <version> <asset-directory>');
  console.log(JSON.stringify(await packageTemplates(...process.argv.slice(2)),null,2));
}
