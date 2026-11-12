import test from 'node:test';
import assert from 'node:assert/strict';
import { packages, buildPlan, validateLive, targetMatches, closure } from './retire-release-images.mjs';
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
    const currentTags = name === 'zellij-base' ? ['', '-maths', '-diagrams', '-all'].flatMap(s => ['0.2.0-rc.4' + s, 'main' + s]) : ['0.2.0-rc.4'];
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
    f => f.inventory.packages[0].versions[0].tags.push('0.2.0-rc.4'),
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
    p => { p.keepVersion = '0.2.0-rc.3'; },
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
