// Install public bytes and execute the PDF driver from the immutable release.
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyPublishedEngine } from './check-published-engine.mjs';

export function verifyPublishedNextPdf({ captures = path.resolve('test-results/next-pdf'), ...options }, hooks) {
  return verifyPublishedEngine({ ...options, captures, driver: 'scripts/check-next-pdf.mjs', label: 'Next PDF' }, hooks);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.platform, 'linux', 'Run public Next PDF acceptance on a native Linux runner');
  await verifyPublishedNextPdf({ version: process.env.VERSION, repository: process.env.RELEASE_REPOSITORY,
    imageRepository: process.env.RELEASE_IMAGE });
}
