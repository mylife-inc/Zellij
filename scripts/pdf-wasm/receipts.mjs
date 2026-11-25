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
let reference;
const platforms = [];
for (const [runner, platform] of matrix) {
  const dir = path.join(root, `pdf-format-receipts-${runner}`);
  const toolchain = await json(path.join(dir, 'pdf-toolchain/verification.json'));
  const cache = await json(path.join(dir, 'pdf-cache/verification.json'));
  const installed = await json(path.join(dir, 'pdf-installed/verification.json'));
  if (reference) assert.deepEqual(toolchain, reference, `Package/source bytes differ on ${platform}`);
  else reference = toolchain;
  assert.equal(toolchain.experimental, true);
  assert.equal(toolchain.redistributionReviewComplete, false);
  assert.equal(cache.platform, platform);
  assert.equal(cache.assertions, 13);
  assert.equal(cache.emptyPath, true);
  assert.equal(cache.coldAndOfflineCompiled, true);
  assert.equal(cache.manifestSha256, toolchain.manifestSha256);
  assert.deepEqual(cache.archive, toolchain.archive);
  assert.deepEqual(cache.sourceArchive, toolchain.sourceArchive);
  assert.equal(installed.platform, platform);
  assert.equal(installed.fixtureOnly, true);
  assert.equal(installed.releaseApproved, false);
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
const evidence = { schemaVersion: 1, experimental: true, redistributionReviewComplete: false,
  platforms, cacheScenarios: 78, compilations: 12, installedCliScenarios: 24, archive: reference.archive,
  manifestSha256: reference.manifestSha256, sourceArchive: reference.sourceArchive,
  sourceManifestSha256: reference.sourceManifestSha256 };
await writeFile('test-results/pdf-package-parity.json', JSON.stringify(evidence, null, 2) + '\n');
console.log('Six identical runtime/source packages; 78 cache scenarios, 12 cache compilations and 24 installed CLI scenarios passed.');
