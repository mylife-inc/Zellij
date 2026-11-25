// Compare receipts only; no third-party runtime/source archive leaves a runner.
import assert from 'node:assert/strict';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve('test-results/pdf-package-receipts');
const matrix = new Map([
  ['ubuntu-latest', 'linux-x64'], ['ubuntu-24.04-arm', 'linux-arm64'],
  ['macos-15-intel', 'darwin-x64'], ['macos-latest', 'darwin-arm64'],
  ['windows-latest', 'win32-x64'], ['windows-11-arm', 'win32-arm64'],
]);
const json = async file => JSON.parse(await readFile(file, 'utf8'));
assert.deepEqual((await readdir(root)).sort(), [...matrix.keys()].map(n => `pdf-format-receipts-${n}`).sort());
let reference, releasedReference, pairedReference;
const platforms = [];
for (const [runner, platform] of matrix) {
  const dir = path.join(root, `pdf-format-receipts-${runner}`);
  const toolchain = await json(path.join(dir, 'pdf-toolchain/verification.json'));
  const cache = await json(path.join(dir, 'pdf-cache/verification.json'));
  const installed = await json(path.join(dir, 'pdf-installed/verification.json'));
  const released = await json(path.join(dir, 'pdf-release/toolchain/verification.json'));
  const paired = await json(path.join(dir, 'pdf-release/verification.json'));
  if (reference) assert.deepEqual(toolchain, reference, `Package/source bytes differ on ${platform}`);
  else reference = toolchain;
  if (releasedReference) assert.deepEqual(released, releasedReference, `Reviewed package differs on ${platform}`);
  else releasedReference = released;
  if (pairedReference) assert.deepEqual(paired, pairedReference, `Runtime/source pair differs on ${platform}`);
  else pairedReference = paired;
  assert.equal(released.experimental, false);
  assert.equal(released.redistributionReviewComplete, true);
  assert.equal(paired.redistributionReviewComplete, true);
  assert.equal(paired.published, false);
  assert.deepEqual(paired.runtime, released.archive);
  assert.deepEqual(paired.source, released.sourceArchive);
  assert.equal(paired.sourceManifestSha256, released.sourceManifestSha256);
  assert.equal(toolchain.experimental, true);
  assert.equal(toolchain.redistributionReviewComplete, false);
  assert.equal(cache.platform, platform);
  assert.equal(cache.assertions, 13);
  assert.equal(cache.emptyPath, true);
  assert.equal(cache.coldAndOfflineCompiled, true);
  assert.equal(cache.manifestSha256, toolchain.manifestSha256);
  assert.deepEqual(cache.archive, toolchain.archive);
  assert.deepEqual(cache.sourceArchive, toolchain.sourceArchive);
  for (const key of ['coldMs', 'offlineMs']) {
    assert(Number.isInteger(cache.timings?.[key]) && cache.timings[key] > 0, `Missing ${key} timing on ${platform}`);
  }
  assert.equal(installed.platform, platform);
  assert.equal(installed.compilerReviewComplete, true);
  assert.equal(installed.rendererFixtureOnly, true);
  assert.equal(installed.publicDownloadVerified, false);
  assert.deepEqual(installed.archive, released.archive);
  assert.deepEqual(installed.sourceArchive, released.sourceArchive);
  assert.equal(installed.manifestSha256, released.manifestSha256);
  assert.equal(installed.cases, 4);
  assert.equal(installed.emptyPath, true);
  assert.equal(installed.privateNodeReused, true);
  assert.equal(installed.coldRequests, 4);
  assert.equal(installed.offlineWatchRebuilt, true);
  platforms.push(platform);
}
const sources = await json('test-results/pdf-package-source-receipt/verification.json');
assert.deepEqual(reference.sourceArchive, sources.sources.archive);
assert.equal(reference.sourceManifestSha256, sources.sourceManifestSha256);
const reviewedSources = await json('test-results/pdf-reviewed-package-source-receipt/verification.json');
assert.deepEqual(releasedReference.sourceArchive, reviewedSources.sources.archive);
assert.equal(releasedReference.sourceManifestSha256, reviewedSources.sourceManifestSha256);
assert.equal(pairedReference.reviewSha256, reviewedSources.reviewSha256);
const evidence = { schemaVersion: 1, publicDownloadVerified: false, redistributionReviewComplete: true,
  platforms, cacheScenarios: 78, compilations: 12, installedCliScenarios: 24, archive: reference.archive,
  manifestSha256: reference.manifestSha256, sourceArchive: reference.sourceArchive,
  sourceManifestSha256: reference.sourceManifestSha256, reviewed: pairedReference };
await writeFile('test-results/pdf-package-parity.json', JSON.stringify(evidence, null, 2) + '\n');
console.log('Six identical qualification and reviewed runtime/source pairs; 78 cache scenarios, 12 cache compilations and 24 installed CLI scenarios passed.');
