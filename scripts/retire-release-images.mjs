// Reviewed, exact-scope rc.8 image retirement. Unreferenced build history is not swept.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const packages = ['zellij-base', 'zellij/acceptance-static', 'zellij/acceptance-next'];
const keepVersion = '0.2.0-rc.8';
const suffixes = ['', '-maths', '-diagrams', '-all'];
const digestPattern = /^sha256:[a-f0-9]{64}$/;
const isRetired = (name, tag) => name === 'zellij-base'
  ? /^(?:(?:0\.1\.0-rc\.[2-6]|0\.1\.3|0\.2\.0-rc\.[1-7])(?:-(?:maths|diagrams|all))?|latest(?:-(?:maths|diagrams|all))?)$/.test(tag)
  : /^0\.2\.0-rc\.[1-7]$/.test(tag);
const sorted = values => [...values].sort();
const requiredTags = name => name === 'zellij-base' ? [keepVersion, 'main'].flatMap(v => suffixes.map(s => v + s)) : [keepVersion];

export function closure(graph, roots, strict = true) {
  const visited = new Set(), pending = [...roots];
  while (pending.length) {
    const digest = pending.pop();
    assert(digestPattern.test(digest), 'Invalid manifest digest');
    if (visited.has(digest)) continue;
    visited.add(digest);
    const node = graph[digest];
    assert(node && Array.isArray(node.children), `Missing graph entry: ${digest}`);
    if (node.missing) { assert(!strict, `Retained manifest is missing: ${digest}`); continue; }
    pending.push(...node.children);
  }
  return visited;
}
function refs(versions) {
  const result = Object.create(null), ids = new Set(), digests = new Set();
  for (const v of versions) {
    assert(Number.isSafeInteger(v.id) && v.id > 0 && digestPattern.test(v.digest));
    assert(!ids.has(v.id) && !digests.has(v.digest), 'Ambiguous package inventory');
    ids.add(v.id); digests.add(v.digest);
    assert(Array.isArray(v.tags) && v.tags.every(t => typeof t === 'string'));
    for (const tag of v.tags) { assert(!Object.hasOwn(result, tag), 'Duplicate image tag'); result[tag] = v.digest; }
  }
  return result;
}
export function buildPlan(inventory, graphs) {
  assert.deepEqual(inventory.errors, [], 'Incomplete inventory');
  assert.deepEqual(sorted(inventory.packages.map(p => p.name)), sorted(packages), 'Unexpected package scope');
  return { schema: 1, keepVersion, createdAt: new Date().toISOString(), inventorySourceCommit: inventory.sourceCommit, packages: inventory.packages.map(p => {
    const expectedRefs = refs(p.versions), graph = graphs[p.name];
    for (const tag of requiredTags(p.name)) assert(expectedRefs[tag], `Missing retained tag ${p.name}:${tag}`);
    const keepTags = Object.keys(expectedRefs).filter(tag => !isRetired(p.name, tag));
    const retiredTags = Object.keys(expectedRefs).filter(tag => isRetired(p.name, tag));
    const protectedDigests = closure(graph, keepTags.map(tag => expectedRefs[tag]));
    const retiredDigests = closure(graph, retiredTags.map(tag => expectedRefs[tag]), false);
    const targets = p.versions.filter(v => retiredDigests.has(v.digest) && !protectedDigests.has(v.digest)).map(v => {
      assert(v.tags.every(tag => isRetired(p.name, tag)), 'A target has an unretired tag');
      return { id: v.id, digest: v.digest, tags: sorted(v.tags) };
    });
    return { name: p.name, observedVersions: p.versions.length, expectedRefs, keepTags: sorted(keepTags), retiredTags: sorted(retiredTags), protectedDigests: sorted(protectedDigests), targets, retainedUnreferencedVersions: p.versions.filter(v => !retiredDigests.has(v.digest) && !protectedDigests.has(v.digest)).length };
  }) };
}
export function targetMatches(expected, current) {
  assert.equal(current.id, expected.id, 'Version identity changed');
  assert.equal(current.digest, expected.digest, 'Version digest changed');
  assert.deepEqual(sorted(current.tags), expected.tags, 'Version acquired or lost tags; replan');
}
export function validateLive(plan, inventory) {
  assert.equal(plan.schema, 1); assert.equal(plan.keepVersion, keepVersion);
  assert.deepEqual(inventory.errors, []);
  assert.deepEqual(sorted(plan.packages.map(p => p.name)), sorted(packages));
  assert.deepEqual(sorted(inventory.packages.map(p => p.name)), sorted(packages));
  for (const p of plan.packages) {
    const current = inventory.packages.find(v => v.name === p.name), liveRefs = refs(current.versions);
    assert.deepEqual(p.keepTags, sorted(Object.keys(p.expectedRefs).filter(tag => !isRetired(p.name, tag))));
    assert.deepEqual(p.retiredTags, sorted(Object.keys(p.expectedRefs).filter(tag => isRetired(p.name, tag))));
    for (const tag of requiredTags(p.name)) assert(p.keepTags.includes(tag), 'Retained release omitted');
    for (const [tag, digest] of Object.entries(liveRefs)) assert.equal(digest, p.expectedRefs[tag], `New or moved tag ${p.name}:${tag}`);
    for (const tag of p.keepTags) assert.equal(liveRefs[tag], p.expectedRefs[tag], `Retained tag disappeared: ${p.name}:${tag}`);
    const ids = new Set(), digests = new Set();
    for (const target of p.targets) {
      assert(Number.isSafeInteger(target.id) && target.id > 0 && digestPattern.test(target.digest));
      assert(!ids.has(target.id) && !digests.has(target.digest), 'Duplicate deletion'); ids.add(target.id); digests.add(target.digest);
      assert(!p.protectedDigests.includes(target.digest), 'A deletion targets a retained manifest');
      assert(target.tags.every(tag => p.retiredTags.includes(tag)), 'A deletion targets an unretired tag');
      const sameDigest = current.versions.find(v => v.digest === target.digest);
      assert(!sameDigest || sameDigest.id === target.id, 'Retired digest acquired a new version identity; replan');
      const v = current.versions.find(v => v.id === target.id);
      if (v) targetMatches(target, v); // Missing exact versions support safe resumption.
    }
  }
}
async function graphFor(name, roots) {
  assert(packages.includes(name));
  const repository = `mylife-inc/${name}`;
  const auth = await fetch(`https://ghcr.io/token?service=ghcr.io&scope=repository:${repository}:pull`, { signal: AbortSignal.timeout(30000) });
  assert(auth.ok, `Registry read token HTTP ${auth.status}`);
  const { token } = await auth.json(), graph = {}, pending = [...roots];
  while (pending.length) {
    const digest = pending.pop(); assert(digestPattern.test(digest));
    if (graph[digest]) continue;
    const r = await fetch(`https://ghcr.io/v2/${repository}/manifests/${digest}`, { headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json' }, signal: AbortSignal.timeout(30000) });
    if (r.status === 404) { graph[digest] = { missing: true, children: [] }; continue; }
    assert(r.ok, `${name}/${digest}: HTTP ${r.status}`);
    const body = Buffer.from(await r.arrayBuffer()); assert(body.length <= 4 * 1024 * 1024, 'Oversized manifest');
    assert.equal('sha256:' + createHash('sha256').update(body).digest('hex'), digest, 'Registry digest mismatch');
    const manifest = JSON.parse(body);
    assert(['application/vnd.oci.image.index.v1+json', 'application/vnd.docker.distribution.manifest.list.v2+json', 'application/vnd.oci.image.manifest.v1+json', 'application/vnd.docker.distribution.manifest.v2+json'].includes(manifest.mediaType));
    const children = (manifest.manifests ?? []).map(m => m.digest);
    if (manifest.subject) children.push(manifest.subject.digest);
    graph[digest] = { children }; pending.push(...children);
  }
  return graph;
}
function api(endpoint, method = 'GET', missing = false, paginated = false) {
  try {
    const text = execFileSync('gh', ['api', '--hostname', 'github.com', '-H', 'Accept: application/vnd.github+json', '-H', 'X-GitHub-Api-Version: 2026-03-10', '--method', method, endpoint, ...(paginated ? ['--paginate', '--slurp'] : [])], { env: { ...process.env, GH_DEBUG: '' }, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 120000, stdio: ['ignore', 'pipe', 'pipe'] });
    return text.trim() ? JSON.parse(text) : null;
  } catch (error) {
    const status = String(error.stderr ?? '').match(/HTTP (\d{3})/)?.[1];
    if (missing && status === '404') return null;
    let message = '';
    try { message = String(JSON.parse(String(error.stdout ?? '')).message ?? '').slice(0, 200); } catch {}
    throw new Error(`${method} ${endpoint}: HTTP ${status ?? 'request failed'}${message ? ': ' + message : ''}`);
  }
}
const endpoint = name => { assert(packages.includes(name)); return `users/mylife-inc/packages/container/${encodeURIComponent(name)}/versions`; };
function liveInventory() {
  return { errors: [], packages: packages.map(name => ({ name, versions: api(`${endpoint(name)}?per_page=100`, 'GET', false, true).flat().map(v => ({ id: v.id, digest: v.name, tags: v.metadata?.container?.tags ?? [] })) })) };
}
export function validateExecution(mode, env, identity) {
  if (mode === 'apply') {
    assert.equal(env.GITHUB_ACTIONS, 'true', 'Workflow apply requires GitHub Actions');
    assert.equal(env.GITHUB_REPOSITORY, 'mylife-inc/Zellij');
    assert(env.GITHUB_REF?.startsWith('refs/heads/'), 'Use a reviewed branch');
    assert(env.GH_TOKEN, 'Missing workflow token');
  } else {
    assert.equal(mode, 'apply-local');
    assert.notEqual(env.GITHUB_ACTIONS, 'true', 'Owner fallback is local only');
    assert.equal(identity?.login, 'mylife-inc', 'Owner fallback requires the mylife-inc login');
    assert(Array.isArray(identity.scopes) && identity.scopes.includes('delete:packages') &&
      identity.scopes.some(scope => ['read:packages', 'write:packages'].includes(scope)),
    'Owner login requires read:packages and delete:packages; refresh gh authorization');
  }
}
export function parseOwnerIdentity(raw) {
  const normalized = raw.replaceAll('\r\n', '\n');
  const end = normalized.indexOf('\n\n');
  assert(end > 0, 'Missing authenticated API response headers');
  const scopes = normalized.slice(0, end).match(/^x-oauth-scopes:[ \t]*([^\n]*)/im)?.[1]
    .split(',').map(value => value.trim()).filter(Boolean) ?? [];
  return { login: JSON.parse(normalized.slice(end + 2)).login, scopes };
}
function ownerIdentity() {
  // Inspect identity and scope names, never read or print the credential itself.
  const raw = execFileSync('gh', ['api', '--hostname', 'github.com', '--include', 'user'], {
    env: { ...process.env, GH_DEBUG: '' }, encoding: 'utf8', timeout: 30000,
    maxBuffer: 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'],
  });
  return parseOwnerIdentity(raw);
}
async function main(mode, input, output) {
  assert(['plan', 'apply', 'apply-local'].includes(mode) && input && output && process.argv.length === 5, 'Usage: retire-release-images.mjs plan|apply|apply-local input.json output.json');
  const bytes = await readFile(input), data = JSON.parse(bytes);
  if (mode === 'plan') {
    assert.deepEqual(data.errors, []);
    const graphs = {};
    for (const p of data.packages) graphs[p.name] = await graphFor(p.name, Object.values(refs(p.versions)));
    const plan = buildPlan(data, graphs);
    await writeFile(output, JSON.stringify(plan, null, 2) + '\n');
    console.log(JSON.stringify(plan.packages.map(p => ({ name: p.name, delete: p.targets.length, protected: p.protectedDigests.length, unreferencedBuildHistoryRetained: p.retainedUnreferencedVersions })), null, 2));
    return;
  }
  const identity = mode === 'apply-local' ? ownerIdentity() : undefined;
  validateExecution(mode, process.env, identity);
  assert.equal(process.env.RETIRE_PLAN_SHA256, createHash('sha256').update(bytes).digest('hex'), 'Reviewed plan hash does not match');
  const sourceCommit = mode === 'apply' ? process.env.GITHUB_SHA : execFileSync('git', ['rev-parse', 'HEAD'], {
    encoding: 'utf8', cwd: fileURLToPath(new URL('../', import.meta.url)),
  }).trim();
  const report = {
    planSha256: process.env.RETIRE_PLAN_SHA256, sourceCommit,
    execution: mode === 'apply' ? 'github-actions' : 'local-owner',
    ...(identity ? { authenticatedOwner: identity.login } : {}),
    keepVersion, startedAt: new Date().toISOString(), complete: false, events: [],
  };
  // Confirm the journal can be written before any remote mutation.
  await writeFile(output, JSON.stringify(report, null, 2) + '\n');
  try {
    // Exact rc.8 source, public, managed upgrade, consumer and image gates.
    for (const [repository, run, head] of [
      ['mylife-inc/Zellij', 43698716955, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43697118925, 'd70e10f983aefeb67474ea209a5caf928ca328f2'],
      ['mylife-inc/homebrew-tap', 43698752235, '064be17acfd53fb506f4bf9db5276c1226cfbd84'],
      ['mylife-inc/homebrew-tap', 43699148795, '064be17acfd53fb506f4bf9db5276c1226cfbd84'],
      ['mylife-inc/scoop-bucket', 43698756686, '356856f2e35dc8dab70177633a05ce464af3fa1d'],
      ['mylife-inc/scoop-bucket', 43699445710, '356856f2e35dc8dab70177633a05ce464af3fa1d'],
      ['mylife-inc/Zellij', 43698717007, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43698716897, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43698716926, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43698716979, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43698717067, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43698716940, '9c61a2a59abc5843e4b0268e9ae0ef2a9020614a'],
      ['mylife-inc/Zellij', 43691586062, '5ed3aeb4ffdbace64026f3c84dccefff8a09fc94'],
      ['mylife-inc/Zellij', 43701099730, '47539533fd1b48e1262e4eb08662e565f3edcc3e'],
      ['mylife-inc/Zellij', 43701102406, '47539533fd1b48e1262e4eb08662e565f3edcc3e'],
      ['mylife-inc/Zellij', 43701261554, '47539533fd1b48e1262e4eb08662e565f3edcc3e']
    ]) {
      const r = api(`repos/${repository}/actions/runs/${run}`);
      assert(r.status === 'completed' && r.conclusion === 'success' && r.head_sha === head, 'Replacement acceptance gate changed');
    }
    validateLive(data, liveInventory());
    for (const p of data.packages) {
      const graph = await graphFor(p.name, p.keepTags.map(tag => p.expectedRefs[tag]));
      const protectedNow = closure(graph, p.keepTags.map(tag => p.expectedRefs[tag]));
      assert.deepEqual(sorted(protectedNow), p.protectedDigests, 'Retained manifest graph changed');
      assert(p.targets.every(t => !protectedNow.has(t.digest)), 'A deletion would break a retained image');
    }
    const remaining = api('rate_limit').resources.core.remaining;
    assert(remaining > data.packages.reduce((sum, p) => sum + p.targets.length * 2, 0) + 100, 'Insufficient API budget; rerun after rate-limit reset');
    for (const p of data.packages) for (const target of p.targets) {
      const current = api(`${endpoint(p.name)}/${target.id}`, 'GET', true);
      if (current) {
        targetMatches(target, { id: current.id, digest: current.name, tags: current.metadata?.container?.tags ?? [] });
        api(`${endpoint(p.name)}/${target.id}`, 'DELETE');
      }
      report.events.push({ package: p.name, ...target, status: current ? 'deleted' : 'already-absent' });
      await writeFile(output, JSON.stringify(report, null, 2) + '\n');
    }
    const after = liveInventory(); validateLive(data, after);
    for (const p of data.packages) {
      const live = after.packages.find(v => v.name === p.name);
      assert(p.targets.every(t => !live.versions.some(v => v.id === t.id)), 'A retired version remains');
      const graph = await graphFor(p.name, p.keepTags.map(tag => p.expectedRefs[tag]));
      assert.deepEqual(sorted(closure(graph, p.keepTags.map(tag => p.expectedRefs[tag]))), p.protectedDigests);
    }
    report.complete = true; report.finishedAt = new Date().toISOString();
  } catch (error) { report.error = error.message; throw error; }
  finally { await writeFile(output, JSON.stringify(report, null, 2) + '\n'); }
  console.log(`Retired ${report.events.length} exact image versions; every retained tag and manifest dependency remains intact.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main(...process.argv.slice(2));
