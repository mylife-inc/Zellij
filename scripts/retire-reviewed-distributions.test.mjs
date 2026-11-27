import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile, mkdtemp, writeFile, chmod, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validatePlan, validateSnapshot, execute, qualification, gates, sha, releaseIdentity, knownTargets } from './retire-reviewed-distributions.mjs';

const baseReview = JSON.parse(await readFile(new URL('../specs/evidence/zellij-retention-review-2026-11-26.json', import.meta.url)));
const planFile = fileURLToPath(new URL('../specs/evidence/zellij-rc16-rc18-execution-plan-2026-11-26.json', import.meta.url));
const basePlan = JSON.parse(await readFile(planFile));
const clone = value => structuredClone(value);
function fixture() {
  const review = clone(baseReview), plan = clone(basePlan), graphs = {}, state = { heads: clone(plan.repositoryHeads), refs: [...clone(plan.protectedRefs), ...plan.targets.map(t => ({ repository: t.repository, ...clone(t.ref) }))], releases: [], packages: [] };
  let id = 9000000;
  for (const repo of review.releases) {
    const templates = [...new Map(repo.targets.flatMap(t => t.assets.filter(a => a.name.startsWith('template-')).map(a => [a.name, a]))).values()];
    const kept = repo.protectedReleases.map(r => ({ id: r.id, tag_name: r.tag, draft: false, prerelease: true, immutable: false, assets: templates.map(a => ({...a, id:id++})) }));
    for (const r of repo.protectedReleases) r.fingerprint = sha(JSON.stringify(releaseIdentity(kept.find(k => k.id === r.id))));
    state.releases.push({ repository: repo.repository, versions: [...repo.targets.map(t => ({ ...releaseIdentity(t), tag_name: t.tag, immutable: false })), ...kept, ...review.protectedUnrelatedReleases.filter(r => r.repository === repo.repository).map(r => ({ ...clone(r), tag_name: r.tag, immutable: false }))] });
  }
  plan.protectedReleases = state.releases.flatMap(repo => repo.versions.filter(r => !knownTargets.some(([name,,id]) => name === repo.repository && id === r.id)).map(r => ({ repository: repo.repository, id: r.id, tag: r.tag_name, fingerprint: sha(JSON.stringify(releaseIdentity(r))) })));
  for (const p of review.images) {
    const kept = p.protectedDigests.map(digest => ({ id: id++, digest, tags: Object.keys(p.expectedRefs).filter(t => p.expectedRefs[t] === digest) }));
    kept.push({ id: id++, digest:'sha256:'+'f'.repeat(64), tags:[] });
    p.expectedRemainingVersions = kept.length;
    p.expectedRemainingFingerprint = sha(JSON.stringify([...kept].sort((a,b) => a.id-b.id).map(r => ({...r,tags:[...r.tags].sort()}))));
    state.packages.push({ name:p.name, versions:[...clone(p.targets), ...kept] });
    const graph = graphs[p.name] = Object.fromEntries(p.protectedDigests.map(d => [d,{children:[]}]));
    const root = p.expectedRefs[p.keepTags[0]]; graph[root].children = p.protectedDigests.filter(d => d !== root);
  }
  const bytes = JSON.stringify(review); plan.reviewSha256 = sha(bytes);
  return {plan,review,bytes,state,graphs};
}
const validate = f => { validatePlan(f.plan,f.review,f.bytes); validateSnapshot(f.plan,f.review,f.state,f.graphs); };
function fakeApi(f, onDelete = () => {}) {
  const deletes = [];
  const call = (endpoint, method = 'GET') => {
    if (method === 'DELETE') { onDelete(endpoint, deletes.length); deletes.push(endpoint); }
    for (const [repo,head] of Object.entries(f.state.heads)) if (endpoint === `repos/${repo}/commits/main`) return {sha:head};
    for (const repo of f.state.releases) {
      const base = `repos/${repo.repository}/releases/`;
      if (endpoint.startsWith(base)) {
        const suffix = endpoint.slice(base.length), r = repo.versions.find(r => suffix === String(r.id) || suffix === `tags/${encodeURIComponent(r.tag_name)}`);
        if (method === 'DELETE' && r) repo.versions = repo.versions.filter(v => v !== r);
        return r ? clone(r) : null;
      }
      for (const prefix of ['ref','refs']) {
        const base = `repos/${repo.repository}/git/${prefix}/tags/`;
        if (endpoint.startsWith(base)) {
          const tag = decodeURIComponent(endpoint.slice(base.length)), r = f.state.refs.find(r => r.repository === repo.repository && r.ref === `refs/tags/${tag}`);
          if (method === 'DELETE' && r) f.state.refs = f.state.refs.filter(v => v !== r);
          return r ? clone(r) : null;
        }
      }
    }
    for (const p of f.state.packages) {
      const prefix = `users/mylife-inc/packages/container/${encodeURIComponent(p.name)}/versions/`;
      if (endpoint.startsWith(prefix)) {
        const r = p.versions.find(r => String(r.id) === endpoint.slice(prefix.length));
        if (method === 'DELETE' && r) p.versions = p.versions.filter(v => v !== r);
        return r ? {id:r.id,name:r.digest,metadata:{container:{tags:clone(r.tags)}}} : null;
      }
    }
    throw new Error(`Unexpected ${method} ${endpoint}`);
  };
  return {call,deletes};
}

test('exact review is executable only through the distinct hashed plan', () => {
  const f=fixture(); validate(f);
  for (const mutate of [
    f=>{f.plan.schema=2;}, f=>{f.plan.reviewSha256='bad';}, f=>f.plan.targets.pop(),
    f=>{f.plan.targets[0].id++;}, f=>f.plan.protectedReleases.pop(), f=>f.plan.protectedRefs.pop(),
    f=>f.review.images[0].protectedDigests.push(f.review.images[0].targets[0].digest),
    f=>f.review.images[0].targets[0].tags.push('main'), f=>f.plan.gates.pop(),
  ]) { const x=fixture(); mutate(x); assert.throws(()=>validate(x)); }
});
test('release assets, immutable state, tag identity and retained products fail closed', () => {
  for (const mutate of [
    f=>{f.state.releases[0].versions[0].assets[0].digest='changed';},
    f=>{f.state.releases[0].versions[0].immutable=true;},
    f=>{delete f.state.releases[0].versions[0].immutable;},
    f=>{f.state.releases[0].versions[0].id++;},
    f=>{f.state.releases[1].versions.at(-1).assets.pop();},
    f=>{f.state.refs[0].object.sha='f'.repeat(40);},
    f=>f.state.refs.pop(), f=>{f.state.heads['mylife-inc/homebrew-tap']='a'.repeat(40);},
    f=>f.state.releases[0].versions.push({...clone(f.state.releases[0].versions[3]),id:42,tag_name:'future'}),
  ]) { const f=fixture(); mutate(f); assert.throws(()=>validate(f)); }
});
test('shared graphs, unreferenced history, missing protected images and new tags fail closed', () => {
  for (const mutate of [
    f=>f.state.packages[0].versions[0].tags.push('keep-me'),
    f=>f.state.packages[0].versions.at(-1).tags.push('0.2.0-rc.160'),
    f=>{f.state.packages[0].versions.at(-1).id++;},
    f=>{f.state.packages[0].versions[0].digest='sha256:'+'e'.repeat(64);},
    f=>f.state.packages[0].versions.splice(-2,1),
    f=>f.state.packages[0].versions.push(clone(f.state.packages[0].versions[0])),
    f=>{delete f.graphs['zellij-base'][f.review.images[0].protectedDigests[0]];},
  ]) {const f=fixture();mutate(f);assert.throws(()=>validate(f));}
});
test('all exact deletes complete with protected state intact, and a second execution makes no changes', async () => {
  const f=fixture();validate(f);const api=fakeApi(f), journal={events:[]};let saves=0;
  await execute(f.plan,f.review,journal,async()=>{saves++;},api.call);
  assert.equal(api.deletes.length,84);assert.equal(saves,168);
  assert.equal(new Set(api.deletes).size,84);
  validateSnapshot(f.plan,f.review,f.state,f.graphs,true);
  await execute(f.plan,f.review,journal,async()=>{},api.call);assert.equal(api.deletes.length,84);
});
test('journal failure prevents the first remote mutation', async () => {
  const f=fixture(),api=fakeApi(f);
  await assert.rejects(execute(f.plan,f.review,{events:[]},async()=>{throw Error('disk full');},api.call),/disk full/);
  assert.equal(api.deletes.length,0);
});
test('partial remote failure is resumable without deleting anything outside the original plan', async () => {
  const f=fixture(),journal={events:[]},first=fakeApi(f,(_,count)=>{if(count===17)throw Error('HTTP 403');});
  await assert.rejects(execute(f.plan,f.review,journal,async()=>{},first.call),/403/);
  assert.equal(first.deletes.length,17);validate(f);
  const second=fakeApi(f);await execute(f.plan,f.review,journal,async()=>{},second.call);
  assert.equal(first.deletes.length+second.deletes.length,84);validateSnapshot(f.plan,f.review,f.state,f.graphs,true);
});
test('an acknowledged deletion followed by lost response can resume safely', async () => {
  const f=fixture(),journal={events:[]},base=fakeApi(f);let lose=true;
  const call=(endpoint,method,...rest)=>{const result=base.call(endpoint,method,...rest);if(method==='DELETE'&&lose){lose=false;throw Error('response lost');}return result;};
  await assert.rejects(execute(f.plan,f.review,journal,async()=>{},call),/response lost/);validate(f);
  await execute(f.plan,f.review,journal,async()=>{},call);assert.equal(base.deletes.length,84);validateSnapshot(f.plan,f.review,f.state,f.graphs,true);
});
test('an image identity reused after partial deletion cannot be resumed', async () => {
  const f=fixture(),api=fakeApi(f);await execute(f.plan,f.review,{events:[]},async()=>{},api.call);
  f.state.packages[0].versions.push({...clone(f.review.images[0].targets[0]),id:123});
  assert.throws(()=>validate(f));
});
test('last-moment release or tag changes prevent deletion', async () => {
  for (const change of ['asset','ref']) {
    const f=fixture();validate(f);const t=f.review.releases[0].targets[0];
    if(change==='asset')f.state.releases[0].versions.find(r=>r.id===t.id).assets[0].size++;
    else f.state.refs.find(r=>r.repository==='mylife-inc/Zellij'&&r.ref===`refs/tags/${t.tag}`).object.sha='f'.repeat(40);
    const api=fakeApi(f);await assert.rejects(execute(f.plan,f.review,{events:[]},async()=>{},api.call));assert.equal(api.deletes.length,0);
  }
});
test('a target acquiring a new tag is rejected immediately before its deletion', async () => {
  const f=fixture(),api=fakeApi(f),target=f.review.images[0].targets[0];validate(f);
  f.state.packages[0].versions.find(r=>r.id===target.id).tags.push('new-retained-tag');
  await assert.rejects(execute(f.plan,f.review,{events:[]},async()=>{},api.call),/Image changed/);
  assert.equal(api.deletes.length,12); // Only the already-verified native releases/tags.
});
test('replacement gates and executor qualification require successful exact-source runs', () => {
  const f=fixture();gates(f.plan, endpoint=>({status:'completed',conclusion:'success',head_sha:f.plan.gates.find(g=>endpoint.endsWith('/'+g.run)).head}));
  assert.throws(()=>gates(f.plan,()=>({status:'completed',conclusion:'failure',head_sha:'a'.repeat(40)})));
  const run={status:'completed',conclusion:'success',head_sha:'a'.repeat(40),path:'.github/workflows/retirement-inventory.yml',event:'workflow_dispatch'};const jobs=[{name:'inventory',conclusion:'success',steps:['Check exact-scope deletion safeguards','Review exact superseded releases and manifest dependencies without deleting','Check the execution plan without deleting'].map(name=>({name,conclusion:'success'}))}];qualification(run,run.head_sha,jobs);
  assert.throws(()=>qualification(run,run.head_sha,[]));const missing=clone(jobs);missing[0].steps.pop();assert.throws(()=>qualification(run,run.head_sha,missing));
  for(const change of [{head_sha:'b'.repeat(40)},{status:'in_progress'},{conclusion:'failure'},{path:'other.yml'},{event:'push'}])assert.throws(()=>qualification({...run,...change},run.head_sha,jobs));
});
test('CLI rejects a wrong plan hash before calling GitHub', async t => {
  const dir=await mkdtemp(path.join(tmpdir(),'zel-retire-cli-'));t.after(()=>rm(dir,{recursive:true,force:true}));
  const result=spawnSync(process.execPath,[fileURLToPath(new URL('./retire-reviewed-distributions.mjs',import.meta.url)),'apply',planFile,path.join(dir,'journal.json')],{env:{...process.env,RETIRE_PLAN_SHA256:'wrong',PATH:dir},encoding:'utf8'});
  assert.notEqual(result.status,0);assert.match(result.stderr,/Reviewed plan hash mismatch/);
});
test('CLI preflight records a failed API gate without any DELETE request', async t => {
  const dir=await mkdtemp(path.join(tmpdir(),'zel-retire-cli-'));t.after(()=>rm(dir,{recursive:true,force:true}));
  const calls=path.join(dir,'calls.txt'), gh=path.join(dir,'gh');
  await writeFile(gh,`#!${process.execPath}\nrequire('node:fs').appendFileSync(${JSON.stringify(calls)},process.argv.slice(2).join(' ')+'\\n');process.stderr.write('HTTP 403');process.exit(1);\n`);await chmod(gh,0o755);
  const output=path.join(dir,'journal.json');
  const result=spawnSync(process.execPath,[fileURLToPath(new URL('./retire-reviewed-distributions.mjs',import.meta.url)),'preflight',planFile,output],{env:{...process.env,RETIRE_PLAN_SHA256:sha(await readFile(planFile)),PATH:dir+path.delimiter+process.env.PATH},encoding:'utf8'});
  assert.notEqual(result.status,0);assert.match(result.stderr,/HTTP 403/);assert(!/DELETE/.test(await readFile(calls,'utf8')));
  const journal=JSON.parse(await readFile(output));assert.equal(journal.complete,false);assert.equal(journal.events.length,0);
});
test('CLI apply rejects wrong owner, missing deletion scopes and unqualified source before mutations', async t => {
  for (const scenario of ['owner','scopes','qualification']) {
    const dir=await mkdtemp(path.join(tmpdir(),'zel-retire-owner-'));t.after(()=>rm(dir,{recursive:true,force:true}));
    const calls=path.join(dir,'calls.txt'), output=path.join(dir,'journal.json');
    const scripts={
      git:`if(process.argv[2]==='rev-parse')console.log('c'.repeat(40));else if(process.argv[2]==='show')process.stdout.write(require('node:fs').readFileSync(${JSON.stringify(planFile)}));else if(process.argv[2]!=='status')process.exit(2);`,
      gh:`const fs=require('node:fs');fs.appendFileSync(${JSON.stringify(calls)},process.argv.slice(2).join(' ')+'\\n');if(process.argv.includes('--include'))console.log('HTTP/2.0 200 OK\\nx-oauth-scopes: ${scenario==='scopes'?'repo':'repo, read:packages, delete:packages'}\\n\\n'+JSON.stringify({login:${JSON.stringify(scenario==='owner'?'other-owner':'mylife-inc')}}));else console.log(JSON.stringify({status:'completed',conclusion:'failure',head_sha:'c'.repeat(40),path:'.github/workflows/retirement-inventory.yml',event:'workflow_dispatch'}));`,
    };
    for(const [name,body]of Object.entries(scripts)){const file=path.join(dir,name);await writeFile(file,`#!${process.execPath}\n${body}\n`);await chmod(file,0o755);}
    const result=spawnSync(process.execPath,[fileURLToPath(new URL('./retire-reviewed-distributions.mjs',import.meta.url)),'apply',planFile,output],{env:{...process.env,GITHUB_ACTIONS:'false',RETIRE_PLAN_SHA256:sha(await readFile(planFile)),RETIRE_QUALIFICATION_RUN:'123',PATH:dir},encoding:'utf8'});
    assert.notEqual(result.status,0);assert.match(result.stderr,scenario==='owner'?/mylife-inc login/:scenario==='scopes'?/delete:packages/:/failure/);
    assert(!/DELETE/.test(await readFile(calls,'utf8')));
  }
});
