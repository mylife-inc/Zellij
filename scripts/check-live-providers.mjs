// Explicit, operator-provisioned disposable destinations only. Builds and
// provider calls run in CI; this harness does not discover or create resources.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {checkDeliverySite} from './check-delivery-http.mjs';

export function validateLiveTargets(value) {
  const uuid = /^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i;
  assert(value && /^zellij-live-\d{8}-[0-9a-f]{6}$/.test(value.session), 'Expected a disposable test session');
  const {cloudflare:cf, github:gh, railway:rw} = value;
  assert(cf?.project === value.session && /^[0-9a-f]{32}$/i.test(cf.account), 'Expected the disposable Cloudflare project and account');
  assert(gh?.repository === `mylife-inc/${value.session}` && Number.isSafeInteger(gh.repositoryId) && gh.repositoryId > 0, 'Expected the separate disposable GitHub repository');
  assert(gh.url === `https://mylife-inc.github.io/${value.session}/`, 'Unexpected GitHub Pages URL');
  assert([rw?.project,rw?.environment,rw?.service].every(id => typeof id === 'string' && uuid.test(id)), 'Expected explicit Railway target UUIDs');
  const url = new URL(rw.url);
  assert(url.protocol === 'https:' && /^[a-z0-9-]+\.up\.railway\.app$/.test(url.hostname) && !url.port && !url.username && !url.password && url.pathname === '/' && !url.search && !url.hash, 'Expected a Railway-generated test URL');
  return value;
}
async function waitForHttp(url, renderer) {
  const deadline = Date.now() + 180_000;
  for (;;) {
    try { await checkDeliverySite(url, renderer); return; }
    catch(error) {
      if (Date.now() >= deadline) throw error;
      console.log(`Waiting for public HTTP acceptance at ${url}: ${error.message}`);
      await new Promise(resolve => setTimeout(resolve, 5_000));
    }
  }
}
async function github(path, token, method = 'GET') {
  const r = await fetch(`https://api.github.com/${path}`, {method,headers:{Authorization:`Bearer ${token}`,Accept:'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28'},signal:AbortSignal.timeout(30_000)});
  assert(r.ok, `GitHub ${method} ${path}: HTTP ${r.status}`);
  return r.status === 204 ? null : r.json();
}
async function main(binary, output) {
  assert(binary && output, 'Pass the current zel binary and an evidence output path');
  const targets = validateLiveTargets(JSON.parse(process.env.ZEL_LIVE_TARGETS));
  const selection = process.env.ZEL_LIVE_PROVIDERS ?? 'all';
  assert(['all','static','railway'].includes(selection), 'Expected all, static or railway providers');
  const required = selection === 'static' ? ['CLOUDFLARE_API_TOKEN','GITHUB_PAGES_TOKEN'] : selection === 'railway' ? ['RAILWAY_TOKEN','GITHUB_TOKEN'] : ['CLOUDFLARE_API_TOKEN','GITHUB_PAGES_TOKEN','RAILWAY_TOKEN','GITHUB_TOKEN'];
  for (const name of required) assert(process.env[name], `Missing ${name}`);
  if (selection !== 'railway') {
    const repo = await github(`repos/${targets.github.repository}`,process.env.GITHUB_PAGES_TOKEN);
    assert(repo.id === targets.github.repositoryId && repo.description?.startsWith('Disposable Zellij CLI live acceptance fixture'), 'GitHub fixture identity changed');
  }
  const token = process.env.GITHUB_TOKEN;
  const run = process.env.GITHUB_RUN_ID;
  assert(/^\d+$/.test(run??''), 'Run live acceptance in GitHub Actions');
  const evidence = {session:targets.session,source:process.env.GITHUB_SHA,run,selection,startedAt:new Date().toISOString(),cases:[],imageCleanup:[]};
  const tags = [];
  function zel(args, env = {}) {
    const result = spawnSync(binary,args,{env:{...process.env,NO_COLOR:'1',...env},encoding:'utf8',maxBuffer:32*1024*1024,timeout:20*60*1000});
    process.stdout.write(result.stdout??''); process.stderr.write(result.stderr??'');
    assert.equal(result.status,0, `zel ${args[0]} failed`);
    return result.stdout;
  }
  async function record(name,url,renderer,details = {}) {
    await waitForHttp(url,renderer);
    evidence.cases.push({name,url,renderer,status:'passed',...details});
    await writeFile(output,JSON.stringify(evidence,null,2)+'\n');
  }
  try {
    for (const renderer of selection === 'railway' ? [] : ['portable','hosted']) {
      const config = `scripts/fixtures/release-delivery/${renderer}/${renderer}.yaml`;
      const cf = zel(['cf','--config',config,'--site',targets.cloudflare.project],{CLOUDFLARE_ACCOUNT_ID:targets.cloudflare.account});
      const url = cf.match(/https:\/\/[a-z0-9.-]+\.pages\.dev\/[^\s]*/)?.[0];
      assert(url,'Cloudflare did not report its deployment URL');
      await record(`cloudflare-${renderer}`,url,renderer);
      const gh = zel(['ghp','--config',config,'--site',targets.github.repository,'--prod']);
      const commit = gh.match(/Published commit ([0-9a-f]{40})/)?.[1];
      assert(commit,'GitHub Pages did not report the published commit');
      await record(`github-pages-${renderer}`,targets.github.url,renderer,{commit});
    }
    for (const kind of selection === 'static' ? [] : ['static','next']) {
      const image = `ghcr.io/mylife-inc/zellij/acceptance-${kind}:live-${run}-${kind}`;
      tags.push({package:`zellij/acceptance-${kind}`,tag:`live-${run}-${kind}`});
      const stdout = zel(['rw','--config','scripts/fixtures/release-delivery/portable/portable.yaml','--site',targets.railway.service,'--kind',kind,'--tag',image,'--prod'],{RAILWAY_PROJECT_ID:targets.railway.project,RAILWAY_ENVIRONMENT_ID:targets.railway.environment});
      await record(`railway-${kind}`,targets.railway.url,'portable',{image,receipt:stdout.trim().split('\n').slice(-5)});
    }
  } finally {
    // Remove only versions exclusively tagged by this run. Never delete a
    // release image or a shared digest carrying any other tag.
    for (const target of tags) {
      try {
        const path = `users/mylife-inc/packages/container/${encodeURIComponent(target.package)}/versions`;
        const versions = await github(`${path}?per_page=100`,token);
        const matches = versions.filter(v => v.metadata?.container?.tags?.includes(target.tag));
        for (const version of matches) {
          const owned = version.metadata.container.tags.every(tag => tags.some(t => t.package === target.package && t.tag === tag));
          if (owned) await github(`${path}/${version.id}`,token,'DELETE');
          evidence.imageCleanup.push({package:target.package,version:version.id,tag:target.tag,status:owned?'deleted':'retained-shared-version'});
        }
      } catch(error) { evidence.imageCleanup.push({package:target.package,tag:target.tag,status:'cleanup-failed',error:error.message}); }
    }
    evidence.finishedAt = new Date().toISOString();
    await writeFile(output,JSON.stringify(evidence,null,2)+'\n');
  }
}
if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) await main(...process.argv.slice(2));
