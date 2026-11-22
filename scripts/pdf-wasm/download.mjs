// Downloads only the checked-in, hashed feasibility resources. No runtime lookup.
import { mkdir, mkdtemp, rename, rm, writeFile, lstat } from 'node:fs/promises';
import path from 'node:path';
import { entries, sha256 } from './bundle.mjs';

const target = process.argv[2] && path.resolve(process.argv[2]);
if (!target || process.argv.length !== 3) throw new Error('Usage: node scripts/pdf-wasm/download.mjs NEW_BUNDLE_DIRECTORY');
try { await lstat(target); throw new Error('Bundle destination already exists'); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
await mkdir(path.dirname(target), { recursive: true });
const stage = await mkdtemp(path.join(path.dirname(target), '.pdf-probe-'));
try {
  for (const entry of entries) {
    const response = await fetch(entry.url, { signal: AbortSignal.timeout(60_000) });
    if (!response.ok) throw new Error(`${entry.name}: HTTP ${response.status}`);
    const chunks = [];
    let length = 0;
    for await (const chunk of response.body) {
      length += chunk.length;
      if (length > entry.bytes) throw new Error(`Oversized resource: ${entry.name}`);
      chunks.push(chunk);
    }
    const bytes = Buffer.concat(chunks);
    if (length !== entry.bytes || sha256(bytes) !== entry.sha256) throw new Error(`Integrity failed: ${entry.name}`);
    await writeFile(path.join(stage, entry.sha256), bytes);
  }
  await rename(stage, target);
  console.log(`Verified ${entries.length} PDF feasibility resources in ${target}`);
} finally { await rm(stage, { recursive: true, force: true }); }
