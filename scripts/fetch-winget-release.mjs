// Verify public Windows packages and the exact manifest bundle before WinGet runs.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { wingetManifests, manifestYaml } from './winget-manifest.mjs';

const [version, destination = 'dist'] = process.argv.slice(2);
assert.equal(process.env.GITHUB_ACTIONS, 'true');
assert.equal(process.platform, 'win32');
// Requires this harness's workspace version: arbitrary refs/versions cannot be
// substituted into the manifest or the subsequent ownership assertions.
execFileSync(process.execPath, ['scripts/package-native.mjs', version, '--check-version'], { stdio: 'inherit' });
execFileSync(process.execPath, ['scripts/fetch-scoop-release.mjs', version, destination], { stdio: 'inherit', timeout: 600000 });
const sums = await readFile(path.join(destination, 'x86_64-pc-windows-msvc/SHA256SUMS'), 'utf8');
const documents = wingetManifests(version, sums);
const name = `zel-winget-${version}.zip`;
const checksums = sums.trim().split(/\r?\n/).map(line => line.trim().split(/\s+/));
const entries = checksums.filter(([, file]) => file === name);
assert.equal(entries.length, 1, 'Expected exactly one manifest-bundle checksum');
const url = `https://github.com/mylife-inc/releases/releases/download/zellij-${version}/${name}`;
const response = await fetch(url, { signal: AbortSignal.timeout(60000) });
assert(response.ok && response.body, `Manifest bundle download failed: HTTP ${response.status}`);
const chunks = []; let size = 0;
for await (const chunk of response.body) {
  size += chunk.length; assert(size <= 1024 * 1024, 'Manifest bundle exceeds size limit'); chunks.push(chunk);
}
const bytes = Buffer.concat(chunks);
assert.equal(createHash('sha256').update(bytes).digest('hex'), entries[0][0].toLowerCase());
const archive = path.resolve(destination, name); await writeFile(archive, bytes, { flag: 'wx' });
const members = execFileSync('tar', ['-tf', archive], { encoding: 'utf8', timeout: 10000 }).trim().split(/\r?\n/);
assert.deepEqual([...members].sort(), Object.keys(documents).sort(), 'Unexpected manifest bundle members');
// Read members to stdout; never extract an archive path onto the filesystem.
for (const member of members) {
  const actual = execFileSync('tar', ['-xOf', archive, member], { encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024 });
  assert.equal(actual.replaceAll('\r\n', '\n'), manifestYaml(documents[member]), `Published manifest differs: ${member}`);
}
console.log('Public Windows archives and manifest bundle match the release checksums and reviewed generator.');
