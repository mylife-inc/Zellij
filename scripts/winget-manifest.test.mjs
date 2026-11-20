import assert from 'node:assert/strict';
import { test } from 'node:test';
import { wingetManifests, manifestYaml, wingetId } from './winget-manifest.mjs';
import yaml from 'js-yaml';

const sums = tag => `${'a'.repeat(64)}  zel-${tag}-x86_64-pc-windows-msvc.zip\n${'b'.repeat(64)}  zel-${tag}-aarch64-pc-windows-msvc.zip\n`;
test('WinGet pins both public archives and installs one managed portable command', () => {
  const docs = wingetManifests('v0.2.0-rc.15', sums('v0.2.0-rc.15'));
  assert.equal(Object.keys(docs).length, 3);
  for (const doc of Object.values(docs)) {
    assert.equal(doc.PackageIdentifier, 'Shebka.Zel');
    assert.equal(doc.PackageVersion, '0.2.0-rc.15');
    assert.deepEqual(yaml.load(manifestYaml(doc)), doc);
  }
  const installer = docs[`${wingetId}.installer.yaml`];
  assert.equal(installer.InstallerType, 'zip'); assert.equal(installer.NestedInstallerType, 'portable');
  assert.deepEqual(installer.NestedInstallerFiles, [{ RelativeFilePath: 'zel.exe', PortableCommandAlias: 'zel' }]);
  assert.equal(installer.Scope, 'user');
  assert.deepEqual(installer.Installers.map(i => i.Architecture), ['x64', 'arm64']);
  for (const i of installer.Installers) {
    assert.match(i.InstallerUrl, /^https:\/\/github.com\/mylife-inc\/releases\/releases\/download\/zellij-v0\.2\.0-rc\.15\/zel-v0\.2\.0-rc\.15-/);
    assert.match(i.InstallerSha256, /^[AB]{64}$/);
  }
  assert.match(docs[`${wingetId}.locale.en-US.yaml`].Description, /development preview/);
});
test('stable publication omits preview wording and malformed inventories cannot produce manifests', () => {
  assert.doesNotMatch(wingetManifests('v1.0.0', sums('v1.0.0'))[`${wingetId}.locale.en-US.yaml`].Description, /preview/);
  for (const tag of ['v0.1.0', 'v0.2.0-rc.14', 'v0.2.0-beta.20']) assert.throws(() => wingetManifests(tag, sums(tag)), /ownership protection/);
  for (const tag of ['v0.2.0-rc.16', 'v0.2.0', 'v0.3.0-rc.1']) assert.doesNotThrow(() => wingetManifests(tag, sums(tag)));
  const tag = 'v0.2.0-rc.15', inventory = sums(tag);
  for (const bad of ['', inventory.split('\n')[0], inventory + inventory, inventory.replace('a'.repeat(64), 'bad')]) assert.throws(() => wingetManifests(tag, bad));
  for (const bad of ['main', 'v1.0.0\n', 'v1.0.0+extra', 'v1.0.0-rc.01']) assert.throws(() => wingetManifests(bad, inventory));
  assert.deepEqual(wingetManifests(tag, inventory), wingetManifests(tag, inventory.replaceAll('\n', '\r\n').replaceAll('  zel-', '  ./zel-')));
});
