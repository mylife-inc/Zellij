// Public pulls on native amd64/arm64 runners; no registry login or publication.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { imageTags } from './release-policy.mjs';

const version = process.env.VERSION;
const repository = process.env.RELEASE_IMAGE;
execFileSync(process.execPath, ['scripts/package-native.mjs', version, '--check-version'], { stdio: 'inherit' });
const expectedEngine = JSON.parse(readFileSync('engine/packages/zellij/package.json', 'utf8'));
const architecture = process.arch === 'arm64' ? 'arm64' : 'amd64';
for (const features of ['none', 'maths', 'diagrams', 'all']) {
  const image = imageTags(version, repository, features)[0];
  execFileSync('docker', ['pull', image], { stdio: 'inherit', timeout: 180000 });
  const actualArchitecture = execFileSync('docker', ['image', 'inspect', '--format', '{{.Architecture}}', image], { encoding: 'utf8' }).trim();
  assert.equal(actualArchitecture, architecture, `${image}: native architecture`);
  const script = "console.log(JSON.stringify({engine:require('/opt/zellij/package.json'),wrapper:require('/app/package.json')}))";
  const actual = JSON.parse(execFileSync('docker', ['run', '--rm', '--network=none', image, 'node', '-e', script], { encoding: 'utf8', timeout: 30000 }));
  assert.equal(actual.engine.version, expectedEngine.version);
  assert.deepEqual(actual.engine.zellij, expectedEngine.zellij);
  assert.deepEqual(actual.wrapper.zellij.features, features === 'none' ? [] : features === 'all' ? ['maths', 'diagrams'] : [features]);
  console.log(`${image}: native ${architecture} execution, engine contract and ${features} selection passed`);
}
