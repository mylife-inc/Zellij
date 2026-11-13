import assert from 'node:assert/strict';
import { test } from 'node:test';
import { scoopManifest } from './scoop-manifest.mjs';
import { nativeTargets } from './release-policy.mjs';

const inventory = (tag, prefix = '') => nativeTargets.map((target, i) =>
  `${String(i + 1).repeat(64)}  ${prefix}zel-${tag}-${target}.${target.includes('windows') ? 'zip' : 'tar.gz'}`,
).join('\n') + '\n';

test('Scoop pins both native Windows archives and preserves managed installation ownership', () => {
  const tag = 'v0.2.0-rc.6';
  const result = scoopManifest(tag, inventory(tag));
  assert.equal(result.version, '0.2.0-rc.6');
  assert.deepEqual(Object.keys(result.architecture), ['64bit', 'arm64']);
  for (const [arch, target, hash] of [['64bit', 'x86_64-pc-windows-msvc', '5'], ['arm64', 'aarch64-pc-windows-msvc', '6']]) {
    assert.deepEqual(result.architecture[arch], {
      url: `https://github.com/mylife-inc/releases/releases/download/zellij-${tag}/zel-${tag}-${target}.zip`, hash: hash.repeat(64),
    });
  }
  assert.equal(result.extract_to, 'bin');
  assert.equal(result.bin, 'bin\\zel.exe');
  assert.match(result.post_install.join('\n'), /share\\zel/);
  assert.match(result.post_install.join('\n'), /installation-manager.*"scoop`n"/);
  assert.match(result.post_install.join('\n'), /UTF8Encoding\]::new\(\$false\)/);
  assert(result.notes.includes('Update with: scoop update zel'));
  assert.match(result.notes.join('\n'), /development preview 0\.2\.0-rc\.6/);
  assert(!('autoupdate' in result)); // The owned bucket only adopts reviewed releases.
  assert(!('env_add_path' in result)); // Scoop owns its shim, not a second PATH entry.
  assert.deepEqual(result, scoopManifest(tag, inventory(tag, './').replaceAll('\n', '\r\n')));
});

test('stable Scoop manifest keeps exact pins and omits preview wording', () => {
  const result = scoopManifest('v1.0.0', inventory('v1.0.0'));
  assert(!result.notes.join('\n').includes('development preview'));
  assert.match(result.notes.join('\n'), /zellij-base:1\.0\.0/);
});

test('Scoop rejects missing, ambiguous and injected release metadata', () => {
  const tag = 'v0.2.0-rc.6', sums = inventory(tag);
  for (const input of ['', sums.split('\n').filter(l => !l.includes('aarch64-pc-windows')).join('\n'),
    sums + sums.split('\n')[4], sums.replace(/5{64}/, 'invalid'), sums.replace('x86_64-pc-windows-msvc', 'wrong-target')]) {
    assert.throws(() => scoopManifest(tag, input));
  }
  for (const badTag of ['main', 'v1.0.0\n', 'v1.0.0"', 'v1.0.0+extra', 'v1.0.0-rc.01']) {
    assert.throws(() => scoopManifest(badTag, sums));
  }
});
