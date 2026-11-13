import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmod, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { packages, buildPlan, validateLive, targetMatches, closure, validateExecution, parseOwnerIdentity } from './retire-release-images.mjs';
const digest = n => 'sha256:' + n.toString(16).padStart(64, '0');
function fixture() {
  let id = 1;
  const graphs = {};
  const inventory = { errors: [], sourceCommit: 'fixture', packages: packages.map(name => {
    const graph = graphs[name] = {};
    const add = (tags, children = []) => {
      const v = { id: id++, digest: digest(id), tags }; graph[v.digest] = { children }; return v;
    };
    const shared = add([]), oldChild = add([]), unknownChild = add([]), orphan = add([]);
    const currentTags = name === 'zellij-base' ? ['', '-maths', '-diagrams', '-all'].flatMap(s => ['0.2.0-rc.6' + s, 'main' + s]) : ['0.2.0-rc.6'];
    const current = add(currentTags, [shared.digest]);
    const old = add(['0.2.0-rc.2'], [oldChild.digest, shared.digest]);
    const unknown = add(['user-retained'], [unknownChild.digest]);
    return { name, versions: [shared, oldChild, unknownChild, orphan, current, old, unknown] };
  }) };
  return { inventory, graphs };
}
const clone = value => JSON.parse(JSON.stringify(value));
test('deletes only retired roots and their exclusive children; preserves shared, unknown and orphan history', () => {
  const { inventory, graphs } = fixture(), plan = buildPlan(inventory, graphs);
  validateLive(plan, inventory);
  for (let i = 0; i < packages.length; i++) {
    const p = plan.packages[i], v = inventory.packages[i].versions;
    assert.deepEqual(p.targets.map(t => t.id), [v[1].id, v[5].id]);
    for (const index of [0, 2, 4, 6]) assert(p.protectedDigests.includes(v[index].digest));
    assert.equal(p.retainedUnreferencedVersions, 1);
  }
});
test('requires every retained variant and a complete retained manifest graph', () => {
  for (const name of packages) {
    const { inventory, graphs } = fixture();
    inventory.packages.find(p => p.name === name).versions[4].tags.shift();
    assert.throws(() => buildPlan(inventory, graphs), /Missing retained tag/);
  }
  const { inventory, graphs } = fixture(), p = inventory.packages[0];
  graphs[p.name][p.versions[0].digest].missing = true;
  assert.throws(() => buildPlan(inventory, graphs), /Retained manifest is missing/);
});
test('rejects incomplete, unexpected or ambiguous inventory', () => {
  for (const mutate of [
    f => f.inventory.errors.push('incomplete'),
    f => f.inventory.packages.push({ name: 'another-product', versions: [] }),
    f => f.inventory.packages[0].versions.push(clone(f.inventory.packages[0].versions[0])),
    f => f.inventory.packages[0].versions[0].tags.push('0.2.0-rc.6'),
    f => { f.inventory.packages[0].versions[0].digest = 'invalid'; },
  ]) { const f = fixture(); mutate(f); assert.throws(() => buildPlan(f.inventory, f.graphs)); }
});
test('fails closed on new or moved tags and altered targets', () => {
  for (const mutate of [
    f => f.inventory.packages[0].versions[3].tags.push('brand-new'),
    f => { f.inventory.packages[0].versions[5].tags = []; f.inventory.packages[0].versions[3].tags = ['0.2.0-rc.2']; },
    f => f.inventory.packages[0].versions[1].tags.push('protect-me'),
    f => { f.inventory.packages[0].versions[1].digest = digest(1000); },
    f => { f.inventory.packages[0].versions[1].id = 1000; },
    f => f.inventory.packages[0].versions[4].tags.shift(),
  ]) { const f = fixture(), plan = buildPlan(f.inventory, f.graphs); mutate(f); assert.throws(() => validateLive(plan, f.inventory)); }
});
test('allows resumption only for already absent exact versions', () => {
  const f = fixture(), plan = buildPlan(f.inventory, f.graphs);
  for (const p of f.inventory.packages) {
    const targets = plan.packages.find(v => v.name === p.name).targets;
    p.versions = p.versions.filter(v => !targets.some(t => t.id === v.id));
  }
  validateLive(plan, f.inventory);
  f.inventory.packages[0].versions = f.inventory.packages[0].versions.filter(v => !v.tags.includes('main'));
  assert.throws(() => validateLive(plan, f.inventory), /Retained tag disappeared/);
});
test('rejects protected targets, duplicates, unexpected scope and version', () => {
  for (const mutate of [
    p => p.packages[0].protectedDigests.push(p.packages[0].targets[0].digest),
    p => p.packages[0].targets.push(clone(p.packages[0].targets[0])),
    p => { p.packages[0].name = 'other'; },
    p => { p.keepVersion = '0.2.0-rc.4'; },
    p => p.packages[0].targets[0].tags.push('main'),
  ]) { const f = fixture(), plan = buildPlan(f.inventory, f.graphs); mutate(plan); assert.throws(() => validateLive(plan, f.inventory)); }
});
test('identity checks require matching ids, digests and all tags', () => {
  const v = { id: 1, digest: digest(1), tags: ['a', 'b'] };
  targetMatches(v, { ...v, tags: ['b', 'a'] });
  for (const change of [{ id: 2 }, { digest: digest(2) }, { tags: ['a'] }]) assert.throws(() => targetMatches(v, { ...v, ...change }));
});
test('graph traversal handles shared nodes and cycles; missing retired leaf may be recorded', () => {
  const graph = { [digest(1)]: { children: [digest(2)] }, [digest(2)]: { children: [digest(1)] } };
  assert.equal(closure(graph, [digest(1)]).size, 2);
  graph[digest(2)] = { missing: true, children: [] };
  assert.equal(closure(graph, [digest(1)], false).size, 2);
  assert.throws(() => closure(graph, [digest(1)]), /missing/);
  delete graph[digest(2)];
  assert.throws(() => closure(graph, [digest(1)], false), /Missing graph entry/);
});

test('retires rc.1 through rc.5 while retaining rc.6 and every development variant', () => {
  for (const version of [1, 2, 3, 4, 5]) {
    const f = fixture();
    for (const p of f.inventory.packages) p.versions[5].tags = [`0.2.0-rc.${version}`];
    const plan = buildPlan(f.inventory, f.graphs);
    assert.equal(plan.keepVersion, '0.2.0-rc.6');
    validateLive(plan, f.inventory);
    for (const p of plan.packages) {
      assert.deepEqual(p.retiredTags, [`0.2.0-rc.${version}`]);
      assert(p.keepTags.includes('0.2.0-rc.6'));
      assert(p.targets.some(t => t.tags.includes(`0.2.0-rc.${version}`)));
    }
  }
});

test('workflow execution keeps its repository, branch and token guards', () => {
  const env = { GITHUB_ACTIONS: 'true', GITHUB_REPOSITORY: 'mylife-inc/Zellij', GITHUB_REF: 'refs/heads/main', GH_TOKEN: 'fixture' };
  validateExecution('apply', env);
  for (const patch of [{ GITHUB_ACTIONS: '' }, { GITHUB_REPOSITORY: 'other/repo' }, { GITHUB_REF: 'refs/tags/v1' }, { GH_TOKEN: '' }]) {
    assert.throws(() => validateExecution('apply', { ...env, ...patch }));
  }
});
test('owner fallback requires the named owner, package scopes and a local process', () => {
  const identity = { login: 'mylife-inc', scopes: ['read:packages', 'delete:packages'] };
  validateExecution('apply-local', {}, identity);
  validateExecution('apply-local', {}, { ...identity, scopes: ['write:packages', 'delete:packages'] });
  for (const value of [undefined, { login: 'mylife-inc' }, { ...identity, login: 'another-account' }, { ...identity, scopes: ['repo'] }, { ...identity, scopes: ['read:packages'] }, { ...identity, scopes: ['delete:packages'] }]) {
    assert.throws(() => validateExecution('apply-local', {}, value));
  }
  assert.throws(() => validateExecution('apply-local', { GITHUB_ACTIONS: 'true' }, identity));
  assert.throws(() => validateExecution('unknown', {}, identity));
});
test('owner preflight parses identity and scope metadata without any credential', () => {
  for (const newline of ['\n', '\r\n']) {
    const raw = ['HTTP/2.0 200 OK', 'X-Oauth-Scopes: repo, read:packages, delete:packages', '', '{"login":"mylife-inc"}'].join(newline);
    assert.deepEqual(parseOwnerIdentity(raw), { login: 'mylife-inc', scopes: ['repo', 'read:packages', 'delete:packages'] });
  }
  assert.deepEqual(parseOwnerIdentity('HTTP/2.0 200 OK\n\n{"login":"mylife-inc"}').scopes, []);
  assert.deepEqual(parseOwnerIdentity('HTTP/2.0 200 OK\nx-oauth-scopes: \nX-Other: read:packages, delete:packages\n\n{"login":"mylife-inc"}').scopes, []);
  assert.throws(() => parseOwnerIdentity('{"login":"mylife-inc"}'));
});
test('local entrypoint enforces plan hash and journals a failed replacement gate before any deletion', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'zellij-retire-guards-'));
  try {
    const fixture = '{}\n', input = path.join(directory, 'plan.json'), output = path.join(directory, 'report.json');
    const calls = path.join(directory, 'calls.jsonl'), gh = path.join(directory, 'gh');
    await writeFile(input, fixture);
    await writeFile(gh, `#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
const args = process.argv.slice(2);
appendFileSync(process.env.RETIRE_FIXTURE_CALLS, JSON.stringify(args) + '\\n');
if (args.at(-1) === 'user') {
  process.stdout.write('HTTP/2.0 200 OK\\nX-Oauth-Scopes: read:packages, delete:packages\\n\\n{"login":"mylife-inc"}');
} else {
  process.stdout.write('{"status":"completed","conclusion":"failure"}');
}
`);
    await chmod(gh, 0o700);
    const env = { ...process.env, GITHUB_ACTIONS: '', PATH: `${directory}${path.delimiter}${process.env.PATH}`, RETIRE_FIXTURE_CALLS: calls };
    const args = [fileURLToPath(new URL('./retire-release-images.mjs', import.meta.url)), 'apply-local', input, output];
    const badHash = spawnSync(process.execPath, args, { env: { ...env, RETIRE_PLAN_SHA256: 'wrong' }, encoding: 'utf8' });
    assert.equal(badHash.status, 1);
    assert.match(badHash.stderr, /Reviewed plan hash does not match/);
    await assert.rejects(readFile(output), { code: 'ENOENT' });
    const failedGate = spawnSync(process.execPath, args, { env: { ...env, RETIRE_PLAN_SHA256: createHash('sha256').update(fixture).digest('hex') }, encoding: 'utf8' });
    assert.equal(failedGate.status, 1);
    const journal = JSON.parse(await readFile(output, 'utf8'));
    assert.equal(journal.execution, 'local-owner');
    assert.equal(journal.authenticatedOwner, 'mylife-inc');
    assert.equal(journal.complete, false);
    assert.deepEqual(journal.events, []);
    assert.match(journal.error, /Replacement acceptance gate changed/);
    const requests = (await readFile(calls, 'utf8')).trim().split('\n').map(line => JSON.parse(line));
    assert.equal(requests.length, 3);
    for (const request of requests) {
      assert(request.includes('--hostname') && request.includes('github.com'));
      assert(!request.includes('DELETE'));
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});
