import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { nativeTargets } from './release-policy.mjs';
import { downloadRelease, releaseAssets } from './download-release.mjs';

const version = 'v0.2.0-rc.1';
const installers = { 'install.sh': Buffer.from('unix installer\n'), 'install.ps1': Buffer.from('windows installer\n') };
async function fixture(t, target = nativeTargets[0]) {
  const root = await mkdtemp(path.join(tmpdir(), 'zel-release-download-test-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const archive = releaseAssets(version, target)[0];
  const bytes = Buffer.from(`archive bytes for ${target}`);
  const hash = createHash('sha256').update(bytes).digest('hex');
  const files = { ...installers, [archive]: bytes, [`${archive}.sha256`]: Buffer.from(`${hash}  ${archive}\n`),
    SHA256SUMS: Buffer.from(`${hash}  ./${archive}\n`) };
  const calls = [];
  const options = { version, target, repository: 'example/Zellij', destination: path.join(root, 'download'), installers,
    fetchImpl: async (url, init) => {
      calls.push({ url, init });
      const name = new URL(url).pathname.split('/').at(-1);
      return new Response(files[name] ?? 'missing', { status: files[name] ? 200 : 404 });
    } };
  return { root, archive, files, options, calls };
}

for (const target of nativeTargets) {
  test(`downloads and verifies the published ${target} archive and installers`, async t => {
    const f = await fixture(t, target);
    const result = await downloadRelease(f.options);
    assert.equal(result.archive.endsWith('.zip'), target.endsWith('windows-msvc'));
    for (const [name, bytes] of Object.entries(f.files)) assert.deepEqual(await readFile(path.join(f.options.destination, name)), bytes);
    assert.equal(f.calls.length, 5);
    assert.ok(f.calls.every(call => call.url.startsWith(`https://github.com/example/Zellij/releases/download/${version}/`)));
    assert.ok(f.calls.every(call => !call.init.headers && call.init.signal));
  });
}

test('shared release scope selects scoped asset URLs', async t => {
  const f = await fixture(t);
  const result = await downloadRelease({ ...f.options, repository: 'mylife-inc/releases', scope: 'zellij' });
  assert.equal(result.base, `https://github.com/mylife-inc/releases/releases/download/zellij-${version}`);
  assert.ok(f.calls.every(call => call.url.startsWith(`${result.base}/`)));
});

test('recovers a reset connection and a partial body without mixing archive bytes', async t => {
  const f = await fixture(t);
  let attempts = 0;
  const result = await downloadRelease({ ...f.options, fetchImpl: async (url, init) => {
    if (url.endsWith(`/${f.archive}`)) {
      attempts++;
      if (attempts === 1) throw new TypeError('fetch failed', { cause: Object.assign(new Error('read reset'), { code: 'ECONNRESET' }) });
      if (attempts === 2) return { ok: true, body: (async function* () {
        yield Buffer.from('incomplete bytes must be discarded');
        throw Object.assign(new Error('other side closed'), { code: 'UND_ERR_SOCKET' });
      })() };
    }
    return f.options.fetchImpl(url, init);
  } });
  assert.equal(attempts, 3);
  assert.deepEqual(result.contents[f.archive], f.files[f.archive]);
  assert.deepEqual(await readFile(path.join(f.options.destination, f.archive)), f.files[f.archive]);
});

test('bounds network retries and saves nothing when the connection never recovers', async t => {
  const f = await fixture(t);
  let attempts = 0;
  await assert.rejects(downloadRelease({ ...f.options, fetchImpl: async () => {
    attempts++;
    throw new DOMException('Request timed out', 'TimeoutError');
  } }), error => {
    assert.match(error.message, new RegExp(f.archive.replaceAll('.', '\\.')));
    assert.match(error.message, /example\/Zellij/);
    assert.equal(error.cause.name, 'TimeoutError');
    return true;
  });
  assert.equal(attempts, 3);
  await assert.rejects(access(f.options.destination));
});

test('does not retry missing assets, oversized assets or integrity failures', async t => {
  for (const [change, expectedCalls] of [
    [f => { delete f.files[f.archive]; }, 1],
    [f => { f.files['install.sh'] = Buffer.alloc(1024 * 1024 + 1); }, 4],
    [f => { f.files[f.archive] = Buffer.from('corrupt'); }, 5],
    [f => { f.files['install.ps1'] = Buffer.from('changed installer'); }, 5],
  ]) {
    const f = await fixture(t);
    change(f);
    await assert.rejects(downloadRelease(f.options));
    assert.equal(f.calls.length, expectedCalls);
    await assert.rejects(access(f.options.destination));
  }
});

test('rejects corrupt, missing, ambiguous and mismatched assets without saving them', async t => {
  const cases = [
    f => { f.files[f.archive] = Buffer.from('corrupt archive'); },
    f => { delete f.files[`${f.archive}.sha256`]; },
    f => { f.files.SHA256SUMS = Buffer.from('malformed'); },
    f => { f.files.SHA256SUMS = Buffer.concat([f.files.SHA256SUMS, f.files.SHA256SUMS]); },
    f => { f.files.SHA256SUMS = Buffer.from(`${'0'.repeat(64)}  ${f.archive}\n`); },
    f => { f.files['install.sh'] = Buffer.from('different installer'); },
    f => { f.files['install.ps1'] = Buffer.from('different installer'); },
    f => { f.files['install.sh'] = Buffer.alloc(1024 * 1024 + 1); },
  ];
  for (const change of cases) {
    const f = await fixture(t);
    change(f);
    await assert.rejects(downloadRelease(f.options));
    await assert.rejects(access(f.options.destination));
  }
});

test('invalid references fail before downloading and existing directories remain untouched', async t => {
  const f = await fixture(t);
  for (const invalid of [{ version: '../bad' }, { target: 'unknown' }, { repository: 'example/repo\n' }, { scope: '../bad' }]) {
    await assert.rejects(downloadRelease({ ...f.options, ...invalid }));
  }
  assert.equal(f.calls.length, 0);
  await mkdir(f.options.destination);
  await writeFile(path.join(f.options.destination, 'keep'), 'authored');
  await assert.rejects(downloadRelease(f.options), { code: 'EEXIST' });
  assert.equal(await readFile(path.join(f.options.destination, 'keep'), 'utf8'), 'authored');
});

test('published image acceptance pulls and executes exactly four pinned variants', async t => {
  const f = await fixture(t);
  const bin = path.join(f.root, 'bin'); await mkdir(bin);
  const log = path.join(f.root, 'docker.jsonl');
  const engine = JSON.parse(await readFile('engine/packages/zellij/package.json', 'utf8'));
  const manifest = await readFile('cli/Cargo.toml', 'utf8');
  const currentVersion = `v${manifest.match(/\[workspace\.package\][\s\S]*?version = "([^"]+)"/)[1]}`;
  const program = `#!${process.execPath}
const fs = require('node:fs');
const args = process.argv.slice(2);
fs.appendFileSync(process.env.DOCKER_TEST_LOG, JSON.stringify(args)+'\\n');
if (args[0] === 'image') console.log(process.arch === 'arm64' ? 'arm64' : 'amd64');
if (args[0] === 'run') {
  const tag = args[3];
  const features = tag.endsWith('-all') ? ['maths','diagrams'] : tag.endsWith('-maths') ? ['maths'] : tag.endsWith('-diagrams') ? ['diagrams'] : [];
  console.log(JSON.stringify({engine: ${JSON.stringify(engine)}, wrapper: {zellij: {features}}}));
}
`;
  await writeFile(path.join(bin, 'docker'), program, { mode: 0o755 });
  execFileSync(process.execPath, ['scripts/check-published-images.mjs'], { env: { ...process.env,
    PATH: `${bin}:${path.dirname(process.execPath)}:${process.env.PATH}`, DOCKER_TEST_LOG: log,
    VERSION: currentVersion, RELEASE_IMAGE: 'ghcr.io/example/zellij-base' } });
  const calls = (await readFile(log, 'utf8')).trim().split('\n').map(line => JSON.parse(line));
  const expected = ['', '-maths', '-diagrams', '-all'].map(suffix => `ghcr.io/example/zellij-base:${currentVersion.slice(1)}${suffix}`);
  assert.deepEqual(calls.filter(args => args[0] === 'pull').map(args => args[1]), expected);
  assert.deepEqual(calls.filter(args => args[0] === 'run').map(args => args.slice(0, 4)), expected.map(image => ['run', '--rm', '--network=none', image]));
});
