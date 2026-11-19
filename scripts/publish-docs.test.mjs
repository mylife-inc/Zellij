import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm, chmod, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import yaml from 'js-yaml';
import { publish, assertContext, apiClient, checkPublished, routes, siteId, domain, repository } from './publish-docs.mjs';

const source = 'a'.repeat(40), newer = 'b'.repeat(40);
const previous = '1'.repeat(24), draftId = '2'.repeat(24);
function fixture() {
  const env = { GITHUB_REPOSITORY: repository, GITHUB_REF: 'refs/heads/main', GITHUB_EVENT_NAME: 'push',
    GITHUB_SHA: source, GH_TOKEN: 'github-test-token', NETLIFY_AUTH_TOKEN: 'netlify-test-token',
    DOCS_OUTPUT: '/checked-artifact', NETLIFY_CLI: '/unused-client' };
  const calls = [], receipts = [], checks = [];
  let promoted = false, siteReads = 0;
  const site = { id: siteId, name: 'zellij-docs', custom_domain: domain, published_deploy: { id: previous } };
  const f = { env, calls, receipts, checks, site, candidate: { site_id: siteId, state: 'ready', title: `Zellij documentation ${source}` },
    relation: 'identical', changeProduction: false, failCheck: false, failPromotion: false };
  f.github = async route => {
    calls.push(['github', route]);
    if (route === '/git/ref/heads/main') return { object: { sha: source } };
    if (route.startsWith(`/compare/${newer}...`)) return { status: f.relation };
    assert.equal(route, `/compare/${source}...${source}`);
    return { status: f.mainRelation ?? 'identical' };
  };
  f.netlify = async (route, method = 'GET') => {
    calls.push(['netlify', method, route]);
    if (route === `/sites/${siteId}`) {
      siteReads++;
      return { ...site, published_deploy: { ...site.published_deploy,
        id: promoted ? draftId : f.changeProduction && siteReads > 1 ? '3'.repeat(24) : site.published_deploy.id } };
    }
    if (route === `/deploys/${draftId}`) return f.candidate;
    assert.equal(route, `/sites/${siteId}/deploys/${draftId}/restore`);
    assert.equal(method, 'POST');
    if (f.failPromotion) throw Error('Uncertain promotion response');
    promoted = true; return {};
  };
  f.deploy = async () => { calls.push(['upload']); return { site_id: siteId, deploy_id: draftId,
    deploy_url: `https://${draftId}--zellij-docs.netlify.app` }; };
  f.check = async origin => { checks.push(origin); if (f.failCheck) throw Error('Draft content mismatch'); };
  f.save = async receipt => { receipts.push(structuredClone(receipt)); };
  return f;
}
const mutations = f => f.calls.filter(call => call[0] === 'upload' || call[1] === 'POST');

test('production requires the owned repository, main, a permitted event and dedicated credentials', () => {
  for (const [key, value] of [['GITHUB_REPOSITORY', 'fork/Zellij'], ['GITHUB_REF', 'refs/heads/topic'],
    ['GITHUB_EVENT_NAME', 'pull_request'], ['GITHUB_SHA', 'bad'], ['NETLIFY_AUTH_TOKEN', '']]) {
    const f = fixture(); f.env[key] = value; assert.throws(() => assertContext(f.env));
  }
});

test('a checked draft is promoted once, verified live and records the rollback ID', async () => {
  const f = fixture(); const result = await publish(f);
  assert.equal(result.state, 'verified'); assert.equal(result.previous_deploy_id, previous);
  assert.deepEqual(mutations(f).map(c => c[0]), ['upload', 'netlify']);
  assert.deepEqual(f.receipts.map(r => r.state), ['draft-verified', 'published', 'verified']);
  assert.deepEqual(f.checks, [`https://${draftId}--zellij-docs.netlify.app`, `https://${domain}`]);
});

for (const defect of ['wrong site', 'wrong domain', 'locked', 'not on main', 'newer already live', 'diverged publication']) {
  test(`${defect} prevents any upload or promotion`, async () => {
    const f = fixture();
    if (defect === 'wrong site') f.site.id = 'wrong';
    if (defect === 'wrong domain') f.site.custom_domain = 'zel-stage.netlify.app';
    if (defect === 'locked') f.site.published_deploy.locked = true;
    if (defect === 'not on main') f.mainRelation = 'diverged';
    if (defect.includes('already') || defect.includes('diverged publication')) {
      f.site.published_deploy.title = `Zellij documentation ${newer}`;
      f.relation = defect.includes('already') ? 'behind' : 'diverged';
    }
    if (defect.includes('already')) assert.equal((await publish(f)).skipped, true);
    else await assert.rejects(publish(f));
    assert.deepEqual(mutations(f), []);
  });
}

for (const defect of ['draft mismatch', 'not ready', 'wrong candidate site', 'wrong receipt', 'production changed']) {
  test(`${defect} leaves production untouched`, async () => {
    const f = fixture();
    if (defect === 'draft mismatch') f.failCheck = true;
    if (defect === 'not ready') f.candidate.state = 'uploading';
    if (defect === 'wrong candidate site') f.candidate.site_id = 'other';
    if (defect === 'wrong receipt') f.candidate.title = 'unexpected source';
    if (defect === 'production changed') f.changeProduction = true;
    await assert.rejects(publish(f));
    assert.deepEqual(mutations(f), [['upload']]);
  });
}

test('an uncertain promotion is not retried and retains the recovery receipt', async () => {
  const f = fixture(); f.failPromotion = true;
  await assert.rejects(publish(f), /Uncertain/);
  assert.equal(f.calls.filter(c => c[1] === 'POST').length, 1);
  assert.equal(f.receipts[0].previous_deploy_id, previous);
  assert.equal(f.receipts[0].state, 'draft-verified');
});

test('API failures omit response bodies and do not forward credentials through redirects', async () => {
  let options;
  const api = apiClient('https://api.netlify.com/api/v1', 'private-token', async (_url, init) => {
    options = init; return { ok: false, status: 403, text: async () => 'sensitive-provider-body' };
  });
  await assert.rejects(api('/sites/example'), e => /HTTP 403/.test(e.message) && !/private-token|sensitive-provider-body/.test(e.message));
  assert.equal(options.redirect, 'error');
});

test('hosted checks tolerate head annotations but reject changed article bytes and missing icons', async t => {
  const output = await mkdtemp(path.join(tmpdir(), 'zellij-docs-publication-'));
  t.after(() => rm(output, { recursive: true, force: true }));
  const html = '<html><head></head><body><main><h1>Documentation</h1></main></body></html>';
  for (const route of routes) {
    const file = path.join(output, route === '/' ? 'index.html' : `${route.slice(1)}.html`);
    await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, html);
  }
  for (const file of ['favicon.ico', '_zellij/assets/logo.png']) {
    await mkdir(path.dirname(path.join(output, file)), { recursive: true }); await writeFile(path.join(output, file), 'image-bytes');
  }
  const fetcher = async url => new Response(url.endsWith('.png') || url.endsWith('.ico') ? 'image-bytes' : html.replace('</head>', '\n<!-- Hosting annotation -->\n</head>'));
  await checkPublished('https://example.test', output, fetcher);
  await assert.rejects(checkPublished('https://example.test', output, async () => new Response(html.replace('Documentation', 'Changed'))));
  await assert.rejects(checkPublished('https://example.test', output, async url => url.endsWith('.ico') ? new Response('missing', { status: 404 }) : fetcher(url)));
});

test('workflow exposes credentials only after validated build and never publishes PRs or release tags', async () => {
  const workflow = yaml.load(await readFile(new URL('../.github/workflows/docs.yml', import.meta.url), 'utf8'));
  assert.deepEqual(workflow.on.push.branches, ['main']);
  assert(workflow.on.push.paths.includes('engine/packages/zellij/**'));
  assert.deepEqual(workflow.on.pull_request.paths, workflow.on.push.paths);
  assert('workflow_dispatch' in workflow.on);
  assert.deepEqual(workflow.permissions, { contents: 'read' });
  assert.doesNotMatch(JSON.stringify(workflow.jobs.build), /secrets\./);
  const job = workflow.jobs.publish;
  assert.equal(job.needs, 'build');
  assert.equal(job.if, "github.repository == 'mylife-inc/Zellij' && github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')");
  assert.equal(job.concurrency['cancel-in-progress'], false);
  const secretSteps = job.steps.filter(s => JSON.stringify(s).includes('secrets.'));
  assert.equal(secretSteps.length, 1);
  assert.equal(secretSteps[0].env.NETLIFY_AUTH_TOKEN, '${{ secrets.NETLIFY_DOCS_PRODUCTION_TOKEN }}');
  assert.doesNotMatch(JSON.stringify(job), /secrets\.NETLIFY_AUTH_TOKEN|secrets\.NETLIFY_SITE_ID|@latest/);
  const build = workflow.jobs.build.steps.map(s => s.run ?? '').join('\n');
  assert.match(build, /check examples\/zellij-docs\/content --deep --strict/);
  assert.match(build, /next build --webpack/);
  assert.match(build, /check-docs-preservation\.mjs/);
});


test('the actual child-process boundary uploads only a draft outside the checkout', async t => {
  const dir = await mkdtemp(path.join(tmpdir(), 'zellij-docs-client-'));
  t.after(() => rm(dir, { recursive: true, force: true }));
  const client = path.join(dir, 'netlify');
  await writeFile(client, `#!${process.execPath}
const fs = require('node:fs');
fs.writeFileSync(${JSON.stringify(path.join(dir, 'arguments.json'))}, JSON.stringify({args:process.argv.slice(2),cwd:process.cwd()}));
console.log(JSON.stringify({site_id:${JSON.stringify(siteId)},deploy_id:${JSON.stringify(draftId)},deploy_url:${JSON.stringify(`https://${draftId}--zellij-docs.netlify.app`)}}));
`);
  await chmod(client, 0o755);
  const f = fixture(); f.deploy = undefined; f.env.NETLIFY_CLI = client; f.env.RUNNER_TEMP = dir;
  await publish(f);
  const recorded = JSON.parse(await readFile(path.join(dir, 'arguments.json'), 'utf8'));
  assert.equal(recorded.cwd, await realpath(dir));
  assert(recorded.args.includes('--draft'));
  assert(recorded.args.includes('--no-build'));
  assert(!recorded.args.includes('--prod'));
  assert.equal(recorded.args[recorded.args.indexOf('--site') + 1], siteId);
  assert.equal(recorded.args[recorded.args.indexOf('--message') + 1], `Zellij documentation ${source}`);
  assert(!recorded.args.includes(f.env.NETLIFY_AUTH_TOKEN));
});

test('a failed live check retains a published receipt and never silently retries or rolls back', async () => {
  const f = fixture();
  f.check = async origin => { if (origin === `https://${domain}`) throw Error('Live HTTP check failed'); };
  await assert.rejects(publish(f), /Live HTTP check failed/);
  assert.deepEqual(f.receipts.map(r => r.state), ['draft-verified', 'published']);
  assert.equal(f.receipts[1].previous_deploy_id, previous);
  assert.equal(f.calls.filter(c => c[1] === 'POST').length, 1);
});
