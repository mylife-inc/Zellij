// Install public bytes and execute the website driver from the immutable release.
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyPublishedEngine } from './check-published-engine.mjs';

export function verifyPublishedWebsites({ captures = path.resolve('test-results/websites'), ...options }, hooks) {
  return verifyPublishedEngine({ ...options, captures, driver: 'scripts/check-website-release.mjs', label: 'website' }, hooks);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.platform, 'linux', 'Run public website acceptance on a native Linux runner');
  await verifyPublishedWebsites({ version: process.env.VERSION, repository: process.env.RELEASE_REPOSITORY,
    imageRepository: process.env.RELEASE_IMAGE });
}
