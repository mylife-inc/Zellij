// Download one native target from a public release. No API token or npm dependencies.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { nativeTargets, releasePolicy } from './release-policy.mjs';

export function releaseAssets(version, target) {
  releasePolicy(version);
  if (!nativeTargets.includes(target)) throw new Error(`Unsupported release target: ${target}`);
  const archive = `zel-${version}-${target}.${target.endsWith('windows-msvc') ? 'zip' : 'tar.gz'}`;
  return [archive, `${archive}.sha256`, 'SHA256SUMS', 'install.sh', 'install.ps1'];
}

export async function downloadRelease({ version, target, repository, scope = '', destination, installers, fetchImpl = fetch }) {
  const names = releaseAssets(version, target);
  if (/\s/.test(repository + scope) || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository) ||
      (scope && !/^[A-Za-z0-9-]+$/.test(scope))) throw new Error('Invalid release repository or scope');
  const tag = `${scope ? `${scope}-` : ''}${version}`;
  const base = `https://github.com/${repository}/releases/download/${tag}`;
  const contents = {};
  for (const name of names) {
    const response = await fetchImpl(`${base}/${name}`, { signal: AbortSignal.timeout(60000) });
    if (!response.ok || !response.body) throw new Error(`Download failed (${response.status}): ${name} from ${repository}`);
    const limit = name === names[0] ? 128 * 1024 * 1024 : 1024 * 1024;
    let size = 0;
    const chunks = [];
    for await (const chunk of response.body) {
      size += chunk.length;
      if (size > limit) throw new Error(`Release asset exceeds size limit: ${name}`);
      chunks.push(chunk);
    }
    contents[name] = Buffer.concat(chunks);
  }
  const archive = names[0];
  const hash = createHash('sha256').update(contents[archive]).digest('hex');
  assert.equal(contents[`${archive}.sha256`].toString().trim(), `${hash}  ${archive}`, 'Per-file checksum mismatch');
  const combined = contents.SHA256SUMS.toString().trim().split(/\r?\n/).map(line => line.match(/^([a-f0-9]{64})  (?:\.\/)?(.+)$/));
  assert(combined.every(Boolean), 'Malformed combined checksum manifest');
  const matches = combined.filter(line => line[2] === archive);
  assert.equal(matches.length, 1, 'Archive must appear exactly once in SHA256SUMS');
  assert.equal(matches[0][1], hash, 'Combined checksum mismatch');
  for (const name of ['install.sh', 'install.ps1']) {
    assert.deepEqual(contents[name], installers[name], `Published ${name} differs from the tagged source`);
  }
  // Refuse existing output. All downloaded bytes are checked before any are saved.
  await mkdir(path.dirname(destination), { recursive: true });
  await mkdir(destination);
  for (const name of names) await writeFile(path.join(destination, name), contents[name], { flag: 'wx' });
  return { base, archive, hash, contents };
}
