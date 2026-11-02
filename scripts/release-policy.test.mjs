import assert from 'node:assert/strict';
import { test } from 'node:test';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import yaml from 'js-yaml';
import { releasePolicy, imageTags, releaseNotes } from './release-policy.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const image = 'ghcr.io/mylife-inc/zellij-base';
const workflow = name => yaml.load(readFileSync(path.join(root, `.github/workflows/${name}.yml`), 'utf8'));
const step = (job, name) => job.steps.find(item => item.name === name);

// Every command boundary is replaced. These tests cannot reach Docker or GitHub.
function fixture(t) {
  const dir = mkdtempSync(path.join(tmpdir(), 'zellij-release-test-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const bin = path.join(dir, 'bin');
  mkdirSync(bin);
  const calls = path.join(dir, 'calls.jsonl');
  for (const tool of ['docker', 'gh', 'git']) {
    writeFileSync(path.join(bin, tool), `#!${process.execPath}
const fs = require('node:fs');
const args = process.argv.slice(2);
const record = { tool: ${JSON.stringify(tool)}, args };
const notesIndex = args.indexOf('--notes-file');
if (notesIndex !== -1) record.notes = fs.readFileSync(args[notesIndex + 1], 'utf8');
fs.appendFileSync(process.env.RECORDED_CALLS, JSON.stringify(record) + '\\n');
if (${JSON.stringify(tool)} === 'gh' && args[0] === 'release' && args[1] === 'view') process.exit(1);
if (${JSON.stringify(tool)} === 'git') process.exit(99);
`, { mode: 0o755 });
  }
  const env = { ...process.env, PATH: `${bin}:${path.dirname(process.execPath)}:${process.env.PATH}`, RECORDED_CALLS: calls,
    DOCKER_HOST: 'unused://test', ZELLIJ_IMAGE: image, ZELLIJ_RELEASES_REPO: 'example/releases', ZELLIJ_RELEASE_SCOPE: 'zellij',
    RUNNER_TEMP: dir, REGISTRY: 'ghcr.io', IMAGE: 'mylife-inc/zellij-base', GITHUB_OUTPUT: path.join(dir, 'outputs') };
  const records = () => {
    try { return readFileSync(calls, 'utf8').trim().split('\n').filter(Boolean).map(line => JSON.parse(line)); }
    catch (error) { if (error.code === 'ENOENT') return []; throw error; }
  };
  const shell = (command, extra = {}) => execFileSync('bash', ['-e', '-o', 'pipefail', '-c', command], { cwd: root, env, encoding: 'utf8', ...extra });
  return { dir, env, records, shell };
}

test('release versions reject malformed tags before they become shell output', () => {
  assert.deepEqual(releasePolicy('v0.2.0'), { version: '0.2.0', series: '0.2', stable: true, prerelease: false });
  for (const version of ['v0.2.0-rc.1', 'v1.0.0-alpha-beta.0', 'v1.0.0-0']) assert.equal(releasePolicy(version).prerelease, true);
  for (const version of [undefined, '', 'main', '0.2.0', 'v01.2.0', 'v0.02.0', 'v0.2.00', 'v0.2.0-rc.01',
    'v0.2.0-', 'v0.2.0-rc..1', 'v0.2.0+build', 'v0.2.0\n', 'v0.2.0-rc\nlatest=true', 'v0.2.0-$(id)', `v0.2.0-${'x'.repeat(128)}`]) {
    assert.throws(() => releasePolicy(version), undefined, String(version));
  }
});

test('all four variants have exact candidate tags and stable-only aliases', () => {
  for (const features of ['none', 'maths', 'diagrams', 'all']) {
    const suffix = features === 'none' ? '' : `-${features}`;
    assert.deepEqual(imageTags('v0.2.0-rc.1', image, features), [`${image}:0.2.0-rc.1${suffix}`]);
    assert.deepEqual(imageTags('v0.2.0', image, features), ['0.2.0', '0.2', 'latest'].map(version => `${image}:${version}${suffix}`));
  }
  for (const invalid of [`${image}:latest`, `${image}\n`, '-t evil', '$(id)/repo']) assert.throws(() => imageTags('v1.0.0', invalid));
  assert.throws(() => imageTags('v1.0.0', image, 'invalid'));
});

test('both release repositories use their own pinned installers and correct image tags', () => {
  for (const version of ['v0.2.0', 'v0.2.0-rc.1']) {
    for (const scope of ['', 'zellij']) {
      const notes = releaseNotes(version, 'example/releases', scope);
      const base = `https://github.com/example/releases/releases/download/${scope ? `${scope}-` : ''}${version}`;
      assert.ok(notes.includes(`curl -fsSL '${base}/install.sh'`));
      assert.ok(notes.includes(`ZEL_VERSION='${version}' ZEL_RELEASE_BASE_URL='${base}' sh ./install.sh`));
      assert.ok(notes.includes(`.\\install.ps1 -Version '${version}' -ReleaseBaseUrl '${base}'`));
      assert.ok(notes.includes(`${image}:${version.slice(1)}-all`));
      assert.doesNotMatch(notes, /raw\.githubusercontent|:latest|:v0/);
    }
  }
  assert.throws(() => releaseNotes('v1.0.0', 'example/releases\n'));
  assert.throws(() => releaseNotes('v1.0.0', 'example/releases', "evil'"));
});

for (const version of ['v0.2.0-rc.1', 'v0.2.0']) {
  test(`local image publisher sends the exact ${version} tag set to Docker`, t => {
    const f = fixture(t);
    execFileSync('bash', ['scripts/publish.sh', 'image', version, 'linux/amd64,linux/arm64'], { cwd: root, env: f.env });
    const calls = f.records();
    assert.equal(calls.length, 2);
    assert.deepEqual(calls[0].args, ['info']);
    const args = calls[1].args;
    assert.deepEqual(args.slice(0, 2), ['buildx', 'build']);
    assert.deepEqual(args.flatMap((arg, i) => arg === '--tag' ? [args[i + 1]] : []),
      (version.includes('-') ? ['0.2.0-rc.1'] : ['0.2.0', '0.2', 'latest']).map(tag => `${image}:${tag}`));
    assert.equal(args[args.indexOf('--platform') + 1], 'linux/amd64,linux/arm64');
  });

  test(`local native publisher pins ${version} and sets prerelease status`, t => {
    const f = fixture(t);
    const dist = path.join(f.dir, 'release assets');
    mkdirSync(dist);
    writeFileSync(path.join(dist, `zel-${version}-fixture.tar.gz`), 'test archive');
    execFileSync('bash', ['scripts/publish.sh', 'release', version, dist], { cwd: root, env: f.env });
    const calls = f.records();
    assert.equal(calls.length, 2);
    const created = calls[1];
    assert.deepEqual(created.args.slice(0, 3), ['release', 'create', `zellij-${version}`]);
    assert.equal(created.args.includes('--prerelease'), version.includes('-'));
    assert.equal(created.args.includes('--latest=false'), version.includes('-'));
    assert.ok(created.notes.includes(`ZEL_VERSION='${version}'`));
    assert.ok(created.notes.includes(`/releases/download/zellij-${version}/install.sh`));
    assert.ok(created.args.includes('install.ps1'));
    assert.ok(created.args.includes('SHA256SUMS'));
    assert.doesNotMatch(created.notes, /raw\.githubusercontent/);
    assert.equal(readFileSync(path.join(dist, 'install.sh'), 'utf8'), readFileSync(path.join(root, 'scripts/install.sh'), 'utf8'));
  });
}

test('malformed local versions fail without contacting either publishing tool', t => {
  const f = fixture(t);
  for (const command of ['image', 'release']) {
    const result = spawnSync('bash', ['scripts/publish.sh', command, 'v0.2.0\n'], { cwd: root, env: f.env });
    assert.notEqual(result.status, 0);
  }
  assert.deepEqual(f.records(), []);
});

test('release workflow selects and publishes the expected manifest tags', t => {
  const f = fixture(t);
  const job = workflow('release').jobs.manifests;
  const digests = path.join(f.dir, 'digests');
  mkdirSync(digests);
  const hashes = ['a'.repeat(64), 'b'.repeat(64)];
  for (const hash of hashes) writeFileSync(path.join(digests, hash), '');
  for (const version of ['v0.2.0-rc.1', 'v0.2.0']) {
    for (const features of ['none', 'maths', 'diagrams', 'all']) {
      const env = { ...f.env, VERSION: version, FEATURES: features };
      f.shell(step(job, 'Select release image tags').run, { env });
      f.shell(step(job, 'Stitch the architectures into one manifest').run, { env, cwd: digests });
      const args = f.records().at(-1).args;
      const suffix = features === 'none' ? '' : `-${features}`;
      const tags = (version.includes('-') ? ['0.2.0-rc.1'] : ['0.2.0', '0.2', 'latest']).map(tag => `${image}:${tag}${suffix}`);
      assert.deepEqual(args, ['buildx', 'imagetools', 'create', ...tags.flatMap(tag => ['-t', tag]), ...hashes.map(hash => `${image}@sha256:${hash}`)]);
    }
  }
  // Missing or unexpected architecture artifacts must fail before publication.
  writeFileSync(path.join(digests, 'unexpected'), '');
  const count = f.records().length;
  assert.throws(() => f.shell(step(job, 'Stitch the architectures into one manifest').run, { cwd: digests, stdio: 'pipe' }));
  assert.equal(f.records().length, count);
});

test('workflow effects are gated by the checked release policy', () => {
  const jobs = workflow('release').jobs;
  assert.equal(jobs.verify.outputs.stable, '${{ steps.policy.outputs.stable }}');
  assert.match(jobs.verify.steps.find(s => s.id === 'policy').run, /package-native.*--check-version[\s\S]*release-policy\.mjs outputs/);
  for (const job of [jobs.docs, jobs.publish, jobs.manifests]) assert.ok(job.needs.includes('verify'));
  assert.equal(step(jobs.docs, 'Deploy to Netlify').if, "needs.verify.outputs.stable == 'true' && env.DEPLOY_DOCS == 'true' && env.HAS_NETLIFY == 'true'");
  assert.equal(step(jobs.publish, 'Keep the installer current in the releases repository').if, "needs.verify.outputs.stable == 'true' && env.HAS_RELEASES_TOKEN == 'true'");
  for (const release of jobs.publish.steps.filter(s => s.uses?.startsWith('softprops/action-gh-release'))) {
    assert.equal(release.with.prerelease, '${{ needs.verify.outputs.prerelease }}');
    assert.equal(release.with.make_latest, '${{ needs.verify.outputs.stable }}');
    assert.ok(release.with.body_path.endsWith('release-notes.md'));
  }
  const base = workflow('base-image');
  assert.equal(base.jobs.build.if, "github.ref == 'refs/heads/main'");
  for (const job of Object.values(base.jobs)) {
    for (const meta of job.steps.filter(s => s.uses?.startsWith('docker/metadata-action'))) {
      assert.equal(meta.with.tags.trim(), 'type=ref,event=branch');
    }
  }
});

test('candidate helper preflight never reads or enables production docs settings', t => {
  const f = fixture(t);
  f.shell(`source scripts/app.sh
_zrepo() { echo example/project; }
_zvar() { echo 'unexpected variable read' >&2; exit 99; }
_zsecret_exists() { echo 'unexpected secret read' >&2; exit 99; }
_zrelease_preflight v0.2.0-rc.1
`);
  assert.deepEqual(f.records().map(call => call.args), [['repo', 'view', 'example/project', '--json', 'name']]);
});

test('normal and emergency release helpers reject a mismatched workspace version before publishing', t => {
  const f = fixture(t);
  // The emergency helper must not reach its build or publishing commands.
  for (const helper of ['zrelease', 'zrelease-local']) {
    const result = spawnSync('bash', ['-c', `source scripts/app.sh; ${helper} v999.999.999-rc.1`], { cwd: root, env: f.env });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr.toString(), /must match the committed workspace version/);
  }
  assert.deepEqual(f.records(), []);
});
