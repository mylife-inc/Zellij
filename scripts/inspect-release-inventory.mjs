// Read-only inventory for the owned Zellij distribution channels. Never deletes.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';

assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Run on the hosted inventory workflow');
assert.equal(process.env.GITHUB_REPOSITORY, 'mylife-inc/Zellij');
assert(process.env.GH_TOKEN, 'The workflow needs its read-only GitHub token');
assert.equal(process.argv.length, 3, 'Supply the inventory output path');
const output = process.argv[2];
const report = { observedAt: new Date().toISOString(), readOnly: true, sourceCommit: process.env.GITHUB_SHA, releases: [], packages: [], errors: [] };
function list(endpoint) {
  // Endpoints are constants below; only public metadata is retained in the report.
  const text = execFileSync('gh', ['api', endpoint, '--paginate', '--slurp'], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 120000, stdio: ['ignore', 'pipe', 'pipe'] });
  const pages = JSON.parse(text);
  assert(Array.isArray(pages) && pages.every(Array.isArray), 'Expected paginated arrays');
  return pages.flat();
}
const failureCode = error => String(error.stderr ?? '').match(/HTTP (\d{3})/)?.[1] ?? (error.signal || 'unknown');
const asset = a => ({ id: a.id, name: a.name, size: a.size, digest: a.digest ?? null });
for (const repository of ['mylife-inc/Zellij', 'mylife-inc/releases']) {
  try {
    const releases = list(`repos/${repository}/releases?per_page=100`);
    const prefix = repository.endsWith('/Zellij') ? /^v[0-9]/ : /^zellij-v[0-9]/;
    report.releases.push({ repository, versions: releases.filter(r => prefix.test(r.tag_name)).map(r => ({ id: r.id, tag: r.tag_name, draft: r.draft, prerelease: r.prerelease, assets: r.assets.map(asset) })) });
  } catch (error) {
    report.errors.push({ status: failureCode(error), channel: repository, error: 'Release inventory failed; no complete inventory is available.' });
  }
}
for (const name of ['zellij-base', 'zellij/acceptance-static', 'zellij/acceptance-next']) {
  try {
    const versions = list(`users/mylife-inc/packages/container/${encodeURIComponent(name)}/versions?per_page=100`);
    report.packages.push({ name, versions: versions.map(v => ({ id: v.id, digest: v.name, tags: v.metadata?.container?.tags ?? [], createdAt: v.created_at, updatedAt: v.updated_at })) });
  } catch (error) {
    report.errors.push({ status: failureCode(error), channel: name, error: 'Package inventory failed; check repository access to this package. No deletion was attempted.' });
  }
}
await writeFile(output, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ readOnly: true, releases: report.releases.map(r => ({ repository: r.repository, count: r.versions.length })), packages: report.packages.map(p => ({ name: p.name, count: p.versions.length })), errors: report.errors }, null, 2));
if (report.errors.length) process.exitCode = 1;
