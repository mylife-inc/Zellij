import assert from 'node:assert/strict';
import { test } from 'node:test';
import { review, refs } from './review-release-retention.mjs';
import { packages, validateLive } from './retire-release-images.mjs';

function fixture() {
  let id = 1;
  const graphs = {}, inventory = { errors: [], sourceCommit: 'synthetic', observedAt: '2026-11-26', releases: [], packages: [] };
  const versions = [11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 24].map(n => `0.2.0-rc.${n}`);
  for (const repository of ['mylife-inc/Zellij', 'mylife-inc/releases']) inventory.releases.push({ repository, versions: versions.map(v => ({ id: id++, tag: `${repository.endsWith('/Zellij') ? 'v' : 'zellij-v'}${v}`, assets: [{ id: id++, name: 'artifact', size: 10, digest: 'hash' }] })) });
  for (const name of packages) {
    const graph = graphs[name] = {}, rows = [];
    const add = (tags, children = []) => {
      const row = { id: id++, digest: `sha256:${id.toString(16).padStart(64, '0')}`, tags };
      rows.push(row); graph[row.digest] = { children }; return row;
    };
    const tags = name === 'zellij-base' ? [...versions, '0.2.0-rc.22', '0.2.0-rc.23', 'main'].flatMap(v => ['', '-maths', '-diagrams', '-all'].map(s => v + s)) : [...versions, 'future-user-tag'];
    for (const tag of tags) add([tag]);
    const shared = add([]);
    for (const tag of ['0.2.0-rc.16', '0.2.0-rc.19']) graph[rows.find(v => v.tags.includes(tag)).digest].children.push(shared.digest);
    add([]); // Unreferenced historical builds are outside this review's scope.
    inventory.packages.push({ name, versions: rows });
  }
  return { inventory, graphs };
}

test('proposes only rc.16–rc.18 and preserves shared manifests and unreferenced history', () => {
  const f = fixture(), plan = review(f.inventory, f.graphs);
  assert.equal(plan.executable, false);
  assert.deepEqual(plan.releases.map(r => r.targets.length), [3, 3]);
  for (const r of plan.releases) for (const v of [11, 13, 14, 15, 19, 20, 21, 24]) assert(r.protectedReleases.some(p => p.tag.endsWith(`rc.${v}`)));
  for (const p of plan.images) {
    assert.equal(p.targets.length, p.name === 'zellij-base' ? 12 : 3);
    assert.equal(p.unreferencedBuildHistoryRetained, 1);
    assert(p.targets.every(t => !p.protectedDigests.includes(t.digest)));
  }
  assert.throws(() => validateLive(plan, f.inventory)); // Not an executable legacy deletion plan.
});

test('an unknown tag on an old manifest protects its entire shared graph', () => {
  const f = fixture(), p = f.inventory.packages[0], row = p.versions.find(v => v.tags.includes('0.2.0-rc.16'));
  row.tags.push('keep-this-build');
  const result = review(f.inventory, f.graphs).images[0];
  assert(result.protectedDigests.includes(row.digest));
  assert(!result.targets.some(v => v.id === row.id));
});

test('missing retained manifests, duplicate identities, incomplete inventory and missing releases fail closed', () => {
  for (const mutate of [
    f => { const p = f.inventory.packages[0]; delete f.graphs[p.name][p.versions[0].digest]; },
    f => f.inventory.packages[0].versions.push(f.inventory.packages[0].versions[0]),
    f => f.inventory.errors.push('HTTP 403'),
    f => f.inventory.releases[0].versions.pop(),
    f => f.inventory.packages.pop(),
  ]) {
    const f = fixture(); mutate(f); assert.throws(() => review(f.inventory, f.graphs));
  }
});

test('future candidate and main tags are protected, never swept by prefix', () => {
  const f = fixture(), p = f.inventory.packages[0];
  const old = p.versions.find(v => v.tags.includes('0.2.0-rc.17'));
  old.tags.push('0.2.0-rc.170');
  const result = review(f.inventory, f.graphs).images[0];
  assert(result.keepTags.includes('main') && result.keepTags.includes('0.2.0-rc.170'));
  assert(!result.targets.some(v => v.id === old.id));
});

test('remaining-identity fingerprints are order independent and notice unrelated mutation', () => {
  const f = fixture(), before = review(f.inventory, f.graphs);
  for (const p of f.inventory.packages) p.versions.reverse();
  assert.deepEqual(review(f.inventory, f.graphs).images.map(p => p.expectedRemainingFingerprint), before.images.map(p => p.expectedRemainingFingerprint));
  f.inventory.packages[0].versions[0].id += 10000;
  assert.notEqual(review(f.inventory, f.graphs).images[0].expectedRemainingFingerprint, before.images[0].expectedRemainingFingerprint);
});

test('ambiguous tags cannot hide a second manifest', () => {
  const f = fixture(), rows = f.inventory.packages[0].versions;
  rows[1].tags.push(rows[0].tags[0]);
  assert.throws(() => refs(rows), /Duplicate/);
});
