// Exact rc.16–rc.18 retirement. Default commands are read-only; apply is owner-only.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { refs, manifests } from './review-release-retention.mjs';
import { closure, packages, parseOwnerIdentity, validateExecution } from './retire-release-images.mjs';

export const repositories = ['mylife-inc/Zellij', 'mylife-inc/releases'];
export const headsRepositories = [...repositories, 'mylife-inc/homebrew-tap', 'mylife-inc/scoop-bucket'];
export const knownTargets = [
  ['mylife-inc/Zellij', 'v0.2.0-rc.16', 453531308],
  ['mylife-inc/Zellij', 'v0.2.0-rc.17', 453982711],
  ['mylife-inc/Zellij', 'v0.2.0-rc.18', 454234418],
  ['mylife-inc/releases', 'zellij-v0.2.0-rc.16', 453531361],
  ['mylife-inc/releases', 'zellij-v0.2.0-rc.17', 453982764],
  ['mylife-inc/releases', 'zellij-v0.2.0-rc.18', 454234456],
];
export const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const sort = rows => [...rows].sort((a, b) => a.id - b.id);
const imageIdentity = r => ({ id: r.id, digest: r.digest, tags: [...r.tags].sort() });
const imageHash = rows => sha(JSON.stringify(sort(rows).map(imageIdentity)));
export const releaseIdentity = r => ({ id: r.id, tag: r.tag_name ?? r.tag, draft: r.draft, prerelease: r.prerelease, assets: [...r.assets].map(a => ({ id: a.id, name: a.name, size: a.size, digest: a.digest ?? null })).sort((a, b) => a.name.localeCompare(b.name)) });
const releaseHash = r => sha(JSON.stringify(releaseIdentity(r)));
const refIdentity = r => ({ ref: r.ref, object: { sha: r.object.sha, type: r.object.type } });
const same = (a, b, message) => assert.deepEqual(a, b, message);
const reviewPath = new URL('../specs/evidence/zellij-retention-review-2026-11-26.json', import.meta.url);
const targetKeys = knownTargets.map(([repository, tag, id]) => `${repository}/${tag}/${id}`).sort();
const targetIn = (repository, row) => knownTargets.some(([repo, tag, id]) => repo === repository && row.id === id && (row.tag_name ?? row.tag) === tag);

export function validatePlan(plan, review, reviewBytes) {
  assert.equal(plan.schema, 3); assert.equal(plan.kind, 'rc16-rc18-execution'); assert.equal(plan.executable, true);
  assert.equal(plan.replacement, 'v0.2.0-rc.24'); assert.equal(plan.reviewSha256, sha(reviewBytes));
  assert.equal(review.schema, 2); assert.equal(review.executable, false);
  same(review.candidateVersions, [16, 17, 18].map(n => `0.2.0-rc.${n}`));
  same(review.releases.flatMap(r => r.targets.map(t => `${r.repository}/${t.tag}/${t.id}`)).sort(), targetKeys);
  same(plan.targets.map(t => `${t.repository}/${t.tag}/${t.id}`).sort(), targetKeys);
  same(Object.keys(plan.repositoryHeads).sort(), [...headsRepositories].sort());
  for (const head of Object.values(plan.repositoryHeads)) assert(/^[a-f0-9]{40}$/.test(head));
  same(review.images.map(p => p.name).sort(), [...packages].sort());
  assert.equal(review.releases.flatMap(r => r.targets).reduce((n, t) => n + t.assets.length, 0), 240);
  for (const p of review.images) {
    refs(p.targets); assert.equal(p.targets.length, p.name === 'zellij-base' ? 54 : 9);
    for (const t of p.targets) {
      assert(!p.protectedDigests.includes(t.digest), 'Protected deletion');
      assert(t.tags.every(tag => p.retiredTags.includes(tag)), 'Unreviewed deletion tag');
    }
  }
  assert.equal(plan.protectedReleases.length, 18); assert.equal(new Set(plan.protectedReleases.map(r => `${r.repository}/${r.id}`)).size, 18);
  for (const repo of review.releases) for (const p of repo.protectedReleases) assert(plan.protectedReleases.some(r => r.repository === repo.repository && r.tag === p.tag && r.id === p.id && r.fingerprint === p.fingerprint), 'Protected release omitted');
  for (const r of review.protectedUnrelatedReleases) assert(plan.protectedReleases.some(p => p.repository === r.repository && p.id === r.id && p.tag === r.tag && p.fingerprint === releaseHash(r)), 'Unrelated release omitted');
  assert.equal(new Set(plan.protectedRefs.map(r => `${r.repository}/${r.ref}`)).size, plan.protectedRefs.length);
  for (const r of [...review.protectedTagRefs, ...review.protectedFailureTags.map(ref => ({ repository: 'mylife-inc/Zellij', ref }))]) assert(plan.protectedRefs.some(p => p.repository === r.repository && p.ref === r.ref.ref && p.object.sha === r.ref.object.sha), 'Protected tag omitted');
  same(plan.gates.map(g => `${g.repository}/${g.run}`).sort(), [
    'mylife-inc/Zellij/45786080712', 'mylife-inc/Zellij/45786080629', 'mylife-inc/Zellij/45791545516',
    'mylife-inc/Zellij/45806880663', 'mylife-inc/Zellij/45921906069', 'mylife-inc/Zellij/45923597757',
    'mylife-inc/homebrew-tap/45800173733', 'mylife-inc/scoop-bucket/45800212328',
  ].sort(), 'Unexpected qualification gates');
  for (const g of plan.gates) assert(/^[a-f0-9]{40}$/.test(g.head));
}

export function validateSnapshot(plan, review, snapshot, graphs, post = false) {
  same(snapshot.heads, plan.repositoryHeads, 'Repository head moved; re-review');
  same(snapshot.releases.map(r => r.repository).sort(), [...repositories].sort());
  same(snapshot.packages.map(p => p.name).sort(), [...packages].sort());
  for (const repo of snapshot.releases) {
    const rows = repo.versions;
    assert.equal(new Set(rows.map(r => r.id)).size, rows.length); assert.equal(new Set(rows.map(r => r.tag_name)).size, rows.length);
    const kept = rows.filter(r => !targetIn(repo.repository, r));
    same(kept.map(r => ({ repository: repo.repository, id: r.id, tag: r.tag_name, fingerprint: releaseHash(r) })).sort((a,b) => a.id-b.id), plan.protectedReleases.filter(r => r.repository === repo.repository).sort((a,b) => a.id-b.id), 'Retained release changed');
    for (const t of review.releases.find(r => r.repository === repo.repository).targets) {
      const byId = rows.find(r => r.id === t.id), byTag = rows.find(r => r.tag_name === t.tag);
      assert.equal(byId, byTag, 'Release identity moved');
      if (byId) { assert(!post, 'Retired release remains'); same(releaseIdentity(byId), releaseIdentity(t), 'Target assets changed'); assert.equal(byId.immutable, false, 'Refuse immutable or unknown release'); }
    }
    for (const t of review.releases.find(r => r.repository === repo.repository).targets) for (const a of t.assets.filter(a => a.name.startsWith('template-'))) assert(kept.some(r => r.assets.some(b => a.name === b.name && a.digest === b.digest)), 'Template loses its retained copy');
    const allRefs = snapshot.refs.filter(r => r.repository === repo.repository);
    const keptRefs = allRefs.filter(r => !plan.targets.some(t => t.repository === r.repository && r.ref === `refs/tags/${t.tag}`));
    same(keptRefs.sort((a,b) => a.ref.localeCompare(b.ref)), plan.protectedRefs.filter(r => r.repository === repo.repository).sort((a,b) => a.ref.localeCompare(b.ref)), 'Retained tags changed');
    for (const t of plan.targets.filter(t => t.repository === repo.repository)) {
      const ref = allRefs.find(r => r.ref === `refs/tags/${t.tag}`);
      if (ref) { assert(!post, 'Retired tag remains'); same(refIdentity(ref), t.ref, 'Target tag changed'); }
      else assert(!rows.some(r => r.id === t.id), 'Live target release lost its tag');
    }
  }
  for (const p of review.images) {
    const rows = snapshot.packages.find(q => q.name === p.name).versions, liveRefs = refs(rows);
    for (const [tag, digest] of Object.entries(liveRefs)) assert.equal(digest, p.expectedRefs[tag], 'New or moved image tag');
    for (const tag of p.keepTags) assert.equal(liveRefs[tag], p.expectedRefs[tag], 'Retained image tag missing');
    const remaining = rows.filter(r => !p.targets.some(t => t.id === r.id));
    assert.equal(remaining.length, p.expectedRemainingVersions); assert.equal(imageHash(remaining), p.expectedRemainingFingerprint, 'Excluded image identity changed');
    for (const t of p.targets) { const r = rows.find(r => r.id === t.id); if (r) { assert(!post, 'Retired image remains'); same(imageIdentity(r), t, 'Image target changed'); } }
    same([...closure(graphs[p.name], p.keepTags.map(t => p.expectedRefs[t]))].sort(), p.protectedDigests, 'Retained manifest graph changed');
  }
}

export function api(endpoint, method = 'GET', missing = false, pages = false) {
  assert(['GET', 'DELETE'].includes(method));
  try {
    const result = execFileSync('gh', ['api', '--hostname', 'github.com', '--method', method, '-H', 'X-GitHub-Api-Version: 2026-03-10', endpoint, ...(pages ? ['--paginate', '--slurp'] : [])], { env: { ...process.env, GH_DEBUG: '' }, encoding: 'utf8', timeout: 120000, maxBuffer: 64 * 1024 * 1024, stdio: ['ignore','pipe','pipe'] });
    return result.trim() ? JSON.parse(result) : null;
  } catch (error) {
    const status = String(error.stderr ?? '').match(/HTTP (\d{3})/)?.[1];
    if (missing && status === '404') return null;
    throw new Error(`${method} ${endpoint}: HTTP ${status ?? 'request failed'}`);
  }
}
const packageEndpoint = name => { assert(packages.includes(name)); return `users/mylife-inc/packages/container/${encodeURIComponent(name)}/versions`; };
const imageRow = r => ({ id: r.id, digest: r.name, tags: r.metadata?.container?.tags ?? [] });
const releaseEndpoint = t => `repos/${t.repository}/releases/${t.id}`;
const tagEndpoint = t => `repos/${t.repository}/git/ref/tags/${encodeURIComponent(t.tag)}`;
export function snapshot(call = api) {
  return {
    heads: Object.fromEntries(headsRepositories.map(repo => [repo, call(`repos/${repo}/commits/main`).sha])),
    releases: repositories.map(repository => ({ repository, versions: call(`repos/${repository}/releases?per_page=100`, 'GET', false, true).flat() })),
    refs: repositories.flatMap(repository => call(`repos/${repository}/git/matching-refs/tags/`, 'GET', false, true).flat().map(r => ({ repository, ...refIdentity(r) }))),
    packages: packages.map(name => ({ name, versions: call(`${packageEndpoint(name)}?per_page=100`, 'GET', false, true).flat().map(imageRow) })),
  };
}
export async function retainedGraphs(review, fetchGraph = manifests) {
  return Object.fromEntries(await Promise.all(review.images.map(async p => [p.name, await fetchGraph(p.name, p.keepTags.map(t => p.expectedRefs[t]))])));
}
export function gates(plan, call = api) {
  for (const g of plan.gates) {
    const r = call(`repos/${g.repository}/actions/runs/${g.run}`);
    assert.equal(r.status, 'completed'); assert.equal(r.conclusion, 'success'); assert.equal(r.head_sha, g.head, 'Qualification source changed');
  }
}
function heads(plan, call) { for (const [repo, sha] of Object.entries(plan.repositoryHeads)) assert.equal(call(`repos/${repo}/commits/main`).sha, sha, 'Repository head moved'); }

export async function execute(plan, review, journal, save, call = api) {
  // The caller completes full source/owner/plan/state/graph preflight first.
  const event = async row => { journal.events.push({ time: new Date().toISOString(), ...row }); await save(journal); };
  for (const repo of review.releases) for (const t of repo.targets) {
    heads(plan, call);
    const target = { ...t, repository: repo.repository };
    const current = call(releaseEndpoint(target), 'GET', true), byTag = call(`repos/${repo.repository}/releases/tags/${encodeURIComponent(t.tag)}`, 'GET', true);
    assert.equal(current?.id, byTag?.id, 'Release lookups disagree');
    const ref = call(tagEndpoint(target), 'GET', true), expected = plan.targets.find(r => r.repository === repo.repository && r.id === t.id);
    if (ref) same(refIdentity(ref), expected.ref, 'Tag changed before deletion');
    if (current) {
      assert(ref, 'Live release lost tag'); same(releaseIdentity(current), releaseIdentity(t), 'Release changed before deletion'); assert.equal(current.immutable, false);
      await event({ action: 'deleting-release', repository: repo.repository, id: t.id, tag: t.tag });
      call(releaseEndpoint(target), 'DELETE');
      assert.equal(call(releaseEndpoint(target), 'GET', true), null, 'Release deletion not confirmed');
      await event({ action: 'deleted-release', repository: repo.repository, id: t.id, tag: t.tag });
    }
    // Re-read even when a release was already absent, including crash recovery.
    const beforeRef = call(tagEndpoint(target), 'GET', true);
    if (beforeRef) {
      same(refIdentity(beforeRef), expected.ref, 'Tag changed before deletion');
      await event({ action: 'deleting-tag', repository: repo.repository, tag: t.tag });
      call(`repos/${repo.repository}/git/refs/tags/${encodeURIComponent(t.tag)}`, 'DELETE');
      assert.equal(call(tagEndpoint(target), 'GET', true), null, 'Tag deletion not confirmed');
      await event({ action: 'deleted-tag', repository: repo.repository, tag: t.tag });
    }
  }
  for (const p of review.images) {
    heads(plan, call);
    for (const t of p.targets) {
      const endpoint = `${packageEndpoint(p.name)}/${t.id}`, current = call(endpoint, 'GET', true);
      if (!current) continue;
      same(imageIdentity(imageRow(current)), t, 'Image changed before deletion');
      await event({ action: 'deleting-image', package: p.name, ...t });
      call(endpoint, 'DELETE');
      assert.equal(call(endpoint, 'GET', true), null, 'Image deletion not confirmed');
      await event({ action: 'deleted-image', package: p.name, ...t });
    }
  }
}

export function qualification(run, head, jobs) {
  assert.equal(run.status, 'completed'); assert.equal(run.conclusion, 'success'); assert.equal(run.head_sha, head);
  assert.equal(run.path, '.github/workflows/retirement-inventory.yml'); assert.equal(run.event, 'workflow_dispatch');
  const inventory = jobs.find(j => j.name === 'inventory');
  assert.equal(inventory?.conclusion, 'success', 'Require the read-only inventory job');
  for (const name of ['Check exact-scope deletion safeguards', 'Review exact superseded releases and manifest dependencies without deleting', 'Check the execution plan without deleting']) assert.equal(inventory.steps.find(s => s.name === name)?.conclusion, 'success', `Qualification step missing: ${name}`);
}

async function main() {
  const [mode, input, output] = process.argv.slice(2);
  assert(['preflight', 'apply', 'verify'].includes(mode) && input && output && process.argv.length === 5, 'Usage: retire-reviewed-distributions.mjs preflight|apply|verify plan.json journal.json');
  const bytes = await readFile(input), plan = JSON.parse(bytes), reviewBytes = await readFile(reviewPath), review = JSON.parse(reviewBytes);
  assert.equal(process.env.RETIRE_PLAN_SHA256, sha(bytes), 'Reviewed plan hash mismatch');
  validatePlan(plan, review, reviewBytes);
  const git = args => execFileSync('git', args, { cwd: fileURLToPath(new URL('../', import.meta.url)), encoding: 'utf8' }).trim();
  const sourceCommit = git(['rev-parse','HEAD']);
  let report = { planSha256: sha(bytes), sourceCommit, mode, startedAt: new Date().toISOString(), complete: false, events: [] };
  if (mode === 'apply') {
    assert.equal(git(['status','--porcelain']), '', 'Apply requires a clean committed checkout');
    assert.equal(await readFile(input, 'utf8'), execFileSync('git', ['show', `HEAD:${path.relative(fileURLToPath(new URL('../', import.meta.url)), path.resolve(input))}`], { encoding: 'utf8' }), 'Plan must be committed');
    const raw = execFileSync('gh', ['api','--hostname','github.com','--include','user'], { env: {...process.env, GH_DEBUG:''}, encoding:'utf8', stdio:['ignore','pipe','pipe'] });
    const owner = parseOwnerIdentity(raw); validateExecution('apply-local', process.env, owner); report.authenticatedOwner = owner.login;
    assert(/^\d+$/.test(process.env.RETIRE_QUALIFICATION_RUN ?? ''), 'Require the successful executor test run');
    qualification(api(`repos/mylife-inc/Zellij/actions/runs/${process.env.RETIRE_QUALIFICATION_RUN}`), sourceCommit, api(`repos/mylife-inc/Zellij/actions/runs/${process.env.RETIRE_QUALIFICATION_RUN}/jobs?per_page=100`).jobs);
    report.qualificationRun = Number(process.env.RETIRE_QUALIFICATION_RUN);
    for (const repo of headsRepositories) for (const status of ['queued','in_progress']) {
      const active = api(`repos/${repo}/actions/runs?status=${status}&per_page=100`);
      assert.equal(active.total_count, 0, `Active workflow in ${repo}; wait before cleanup`);
    }
    try { const existing = JSON.parse(await readFile(output)); assert.equal(existing.planSha256, report.planSha256, 'Journal belongs to another plan'); assert.equal(existing.sourceCommit, sourceCommit, 'Resume from the same qualified source'); report.events = existing.events; } catch (e) { if (e.code !== 'ENOENT') throw e; }
  }
  const save = r => writeFile(output, JSON.stringify(r, null, 2) + '\n');
  await save(report); // Must succeed before the first mutation.
  try {
    gates(plan); const before = snapshot(); const graphs = await retainedGraphs(review);
    validateSnapshot(plan, review, before, graphs, mode === 'verify');
    report.preflightPassed = true; await save(report);
    if (mode === 'apply') {
      await execute(plan, review, report, save);
      const after = snapshot(); validateSnapshot(plan, review, after, await retainedGraphs(review), true);
      report.postCounts = { releases: after.releases.map(r => ({ repository: r.repository, count: r.versions.length })), packages: after.packages.map(p => ({ name: p.name, count: p.versions.length })) };
    }
    report.complete = true; report.finishedAt = new Date().toISOString(); await save(report);
    console.log(JSON.stringify({ mode, complete: true, deletions: report.events.filter(e => e.action.startsWith('deleted-')).length, ...(report.postCounts ?? {}) }));
  } catch (e) { report.error = e.message; await save(report); throw e; }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
