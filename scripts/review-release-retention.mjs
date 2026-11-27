// Read-only rc.16–rc.18 review. Deliberately has no apply mode or DELETE request.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { closure, packages } from './retire-release-images.mjs';

const versions = ['0.2.0-rc.16', '0.2.0-rc.17', '0.2.0-rc.18'];
const repositories = ['mylife-inc/Zellij', 'mylife-inc/releases'];
const protectedVersions = [11, 13, 14, 15, 19, 20, 21, 24].map(n => `0.2.0-rc.${n}`);
const sha = value => createHash('sha256').update(value).digest('hex');
const sorted = rows => [...rows].sort((a, b) => a.id - b.id);
const identity = row => ({ id: row.id, digest: row.digest, tags: [...row.tags].sort() });
const packageHash = rows => sha(JSON.stringify(sorted(rows).map(identity)));
const tagFor = (repo, version) => `${repo.endsWith('/Zellij') ? 'v' : 'zellij-v'}${version}`;
const retiredTag = (name, tag) => versions.some(v => tag === v || (name === 'zellij-base' && ['maths', 'diagrams', 'all'].some(f => tag === `${v}-${f}`)));

export function refs(rows) {
  const tags = {}, ids = new Set(), digests = new Set();
  for (const row of rows) {
    assert(Number.isSafeInteger(row.id) && row.id > 0 && /^sha256:[a-f0-9]{64}$/.test(row.digest), 'Invalid version identity');
    assert(!ids.has(row.id) && !digests.has(row.digest), 'Duplicate version identity');
    ids.add(row.id); digests.add(row.digest);
    assert(Array.isArray(row.tags));
    for (const tag of row.tags) {
      assert(typeof tag === 'string' && !Object.hasOwn(tags, tag), 'Duplicate or invalid tag');
      Object.defineProperty(tags, tag, { value: row.digest, enumerable: true });
    }
  }
  return tags;
}

export function review(inventory, graphs) {
  assert.deepEqual(inventory.errors, [], 'Incomplete inventory');
  assert.deepEqual(inventory.packages.map(p => p.name).sort(), [...packages].sort(), 'Unexpected package scope');
  assert.deepEqual(inventory.releases.map(r => r.repository).sort(), [...repositories].sort(), 'Unexpected repository scope');
  const releases = inventory.releases.map(repo => {
    assert.equal(new Set(repo.versions.map(r => r.tag)).size, repo.versions.length, 'Duplicate release tag');
    for (const version of [...versions, ...protectedVersions]) assert(repo.versions.some(r => r.tag === tagFor(repo.repository, version)), `Missing release ${version}`);
    const targets = repo.versions.filter(r => versions.some(v => r.tag === tagFor(repo.repository, v)));
    const protectedReleases = repo.versions.filter(r => !targets.includes(r));
    for (const r of repo.versions) assert(Number.isSafeInteger(r.id) && r.id > 0 && r.assets.length > 0, 'Invalid release inventory');
    const fingerprint = r => sha(JSON.stringify({ ...r, assets: [...r.assets].sort((a, b) => a.name.localeCompare(b.name)) }));
    return { repository: repo.repository, targets, protectedReleases: protectedReleases.map(r => ({ tag: r.tag, id: r.id, assets: r.assets.length, fingerprint: fingerprint(r) })) };
  });
  const images = inventory.packages.map(p => {
    const expectedRefs = refs(p.versions), graph = graphs[p.name];
    const retiredTags = Object.keys(expectedRefs).filter(tag => retiredTag(p.name, tag));
    const keepTags = Object.keys(expectedRefs).filter(tag => !retiredTag(p.name, tag));
    for (const v of versions) assert(retiredTags.includes(v), `Missing candidate image ${p.name}:${v}`);
    const required = p.name === 'zellij-base'
      ? [...protectedVersions, '0.2.0-rc.22', '0.2.0-rc.23', 'main'].flatMap(v => ['', '-maths', '-diagrams', '-all'].map(s => v + s))
      : ['0.2.0-rc.19'];
    for (const tag of required) assert(keepTags.includes(tag), `Missing retained image ${p.name}:${tag}`);
    const protectedDigests = closure(graph, keepTags.map(t => expectedRefs[t]));
    const retiredDigests = closure(graph, retiredTags.map(t => expectedRefs[t]));
    const targets = p.versions.filter(v => retiredDigests.has(v.digest) && !protectedDigests.has(v.digest));
    for (const v of targets) assert(v.tags.every(t => retiredTag(p.name, t)), 'Unknown tag cannot be retired');
    const remaining = p.versions.filter(v => !targets.includes(v));
    return { name: p.name, observedVersions: p.versions.length, expectedRefs, retiredTags: retiredTags.sort(), keepTags: keepTags.sort(), protectedDigests: [...protectedDigests].sort(), targets: sorted(targets).map(identity), expectedRemainingVersions: remaining.length, expectedRemainingFingerprint: packageHash(remaining), unreferencedBuildHistoryRetained: p.versions.filter(v => !protectedDigests.has(v.digest) && !retiredDigests.has(v.digest)).length };
  });
  return { schema: 2, kind: 'read-only-retention-review', executable: false, sourceCommit: inventory.sourceCommit, observedAt: inventory.observedAt, candidateVersions: versions, releases, images };
}

async function manifests(name, roots) {
  assert(packages.includes(name));
  const repository = `mylife-inc/${name}`;
  const auth = await fetch(`https://ghcr.io/token?service=ghcr.io&scope=repository:${repository}:pull`, { signal: AbortSignal.timeout(30000) });
  assert(auth.ok, `Read-token HTTP ${auth.status}`);
  const { token } = await auth.json();
  const graph = {}, pending = [...roots];
  while (pending.length) {
    const digest = pending.pop();
    assert(/^sha256:[a-f0-9]{64}$/.test(digest));
    if (graph[digest]) continue;
    const response = await fetch(`https://ghcr.io/v2/${repository}/manifests/${digest}`, { headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json' }, signal: AbortSignal.timeout(30000) });
    assert(response.ok, `Manifest HTTP ${response.status}: ${name}/${digest}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    assert(bytes.length <= 4 * 1024 * 1024 && `sha256:${sha(bytes)}` === digest, 'Manifest size or digest mismatch');
    const body = JSON.parse(bytes);
    assert(['application/vnd.oci.image.index.v1+json', 'application/vnd.docker.distribution.manifest.list.v2+json', 'application/vnd.oci.image.manifest.v1+json', 'application/vnd.docker.distribution.manifest.v2+json'].includes(body.mediaType));
    const children = (body.manifests ?? []).map(m => m.digest);
    if (body.subject) children.push(body.subject.digest);
    graph[digest] = { children }; pending.push(...children);
  }
  return graph;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [input, output] = process.argv.slice(2);
  assert(input && output && process.argv.length === 4, 'Usage: review-release-retention.mjs inventory.json review.json (read only)');
  const bytes = await readFile(input), inventory = JSON.parse(bytes);
  assert.deepEqual(inventory.errors, []);
  const graphs = Object.fromEntries(await Promise.all(inventory.packages.map(async p => [p.name, await manifests(p.name, Object.values(refs(p.versions)))])));
  const result = { ...review(inventory, graphs), inventorySha256: sha(bytes) };
  await writeFile(output, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify({ executable: false, releases: result.releases.map(r => ({ repository: r.repository, releases: r.targets.length, assets: r.targets.reduce((n, r) => n + r.assets.length, 0) })), images: result.images.map(p => ({ package: p.name, candidates: p.targets.length, protectedManifests: p.protectedDigests.length, unreferencedRetained: p.unreferencedBuildHistoryRetained })) }, null, 2));
}
