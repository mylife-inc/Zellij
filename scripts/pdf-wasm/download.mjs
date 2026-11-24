// Downloads only the checked-in, hashed feasibility resources. No runtime lookup.
import { mkdir, mkdtemp, rename, rm, writeFile, lstat } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { entries, sha256, verifiedBytes } from './bundle.mjs';

const transientCodes = new Set(['ECONNRESET', 'ETIMEDOUT', 'EAI_AGAIN', 'UND_ERR_SOCKET', 'UND_ERR_CONNECT_TIMEOUT',
  'UND_ERR_HEADERS_TIMEOUT', 'UND_ERR_BODY_TIMEOUT']);
const transientStatuses = new Set([408, 429, 500, 502, 503, 504]);
class RetryableHttp extends Error {}
const transient = error => error instanceof RetryableHttp || error?.name === 'TimeoutError'
  || transientCodes.has(error?.code) || transientCodes.has(error?.cause?.code);

export async function fetchResource(entry, { fetcher = fetch, sleep = delay, timeoutMs = 60_000, report = console.warn, originals } = {}) {
  if (!/^[a-f0-9]{64}$/.test(entry.sha256) || !Number.isSafeInteger(entry.bytes)
      || entry.bytes < 0 || entry.bytes > 64 * 1024 * 1024
      || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 60_000) {
    throw new Error('Invalid pinned download input or deadline');
  }
  if (entry.original !== undefined) {
    if (typeof entry.original !== 'string' || entry.url !== undefined || !originals) {
      throw new Error('Prepare the pinned original resources with originals.py and pass their directory');
    }
    const info = await lstat(originals);
    if (!info.isDirectory() || info.isSymbolicLink()) throw new Error('Original resources must be a real directory');
    return Buffer.from(verifiedBytes(originals, entry));
  }
  for (let attempt = 1; attempt <= 3; attempt++) {
    const signal = AbortSignal.timeout(timeoutMs);
    try {
      const response = await fetcher(entry.url, { signal });
      if (!response.ok) {
        await response.body?.cancel().catch(() => {});
        const ErrorType = transientStatuses.has(response.status) ? RetryableHttp : Error;
        throw new ErrorType(`${entry.name}: HTTP ${response.status}`);
      }
      const chunks = [];
      let length = 0;
      if (response.body) {
        for await (const chunk of response.body) {
          length += chunk.length;
          if (length > entry.bytes) throw new Error(`Oversized resource: ${entry.name}`);
          chunks.push(chunk);
        }
      }
      const bytes = Buffer.concat(chunks);
      if (length !== entry.bytes || sha256(bytes) !== entry.sha256) throw new Error(`Integrity failed: ${entry.name}`);
      return bytes;
    } catch (error) {
      const timedOut = signal.aborted && signal.reason?.name === 'TimeoutError'
        && ['AbortError', 'TimeoutError'].includes(error?.name);
      if ((!transient(error) && !timedOut) || attempt === 3) {
        throw new Error(`${entry.name}: download failed after ${attempt} attempt(s): ${error.message}`, { cause: error });
      }
      report(`${entry.name}: transient download failure; retry ${attempt + 1}/3`);
      await sleep(attempt * 1000);
    }
  }
}

async function absent(target) {
  try { await lstat(target); throw new Error('Bundle destination already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
}

export async function downloadBundle(target, options, inventory = entries) {
  target = path.resolve(target);
  await absent(target);
  await mkdir(path.dirname(target), { recursive: true });
  const stage = await mkdtemp(path.join(path.dirname(target), '.pdf-probe-'));
  try {
    for (const entry of inventory) {
      const bytes = await fetchResource(entry, options);
      // Distinct logical resources may have identical verified content hashes.
      await writeFile(path.join(stage, entry.sha256), bytes);
    }
    await absent(target);
    await rename(stage, target);
  } finally { await rm(stage, { recursive: true, force: true }); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (![3, 4].includes(process.argv.length)) throw new Error('Usage: node scripts/pdf-wasm/download.mjs NEW_BUNDLE_DIRECTORY [PREPARED_ORIGINALS]');
  await downloadBundle(process.argv[2], { originals: process.argv[3] });
  console.log(`Verified ${entries.length} PDF feasibility resources in ${path.resolve(process.argv[2])}`);
}
