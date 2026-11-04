// Fake native clients only: argument, ordering and preservation acceptance.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const root = await mkdtemp(path.join(tmpdir(), 'zel-multi-'));
const bin = path.join(root, 'bin');
const exe = name => path.join(bin, name + (process.platform === 'win32' ? '.exe' : ''));
const env = { ...process.env, ZELLIJ_BUILD_FIXTURE: root };
const selectors = ['DOCKER_CONTEXT', 'DOCKER_HOST', 'CONTAINER_CONNECTION', 'CONTAINER_HOST', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE'];
for (const key of Object.keys(env)) if (['PATH', 'NETLIFY_SITE_ID', 'NETLIFY_AUTH_TOKEN', ...selectors].includes(key.toUpperCase())) delete env[key];
env.PATH = bin;
const run = (args, code = 0, extra = {}) => {
  const result = spawnSync(exe('zel'), args, { cwd: root, env: { ...env, DOCKER_CONTEXT: 'fixture', ...extra }, encoding: 'utf8', timeout: 30000 });
  assert.equal(result.status, code, `${args}\n${result.stdout}\n${result.stderr}\n${result.error ?? ''}`);
  return result;
};
const write = async (name, text) => { const file = path.join(root, name); await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, text); };
const read = name => readFile(path.join(root, name), 'utf8');
const reset = () => write('calls', '');
const calls = async () => (await read('calls')).trimEnd().split('\n').filter(Boolean).map(line => line.split('\0'));
const args = ['--build-arg', 'LABEL=日本語 $(literal) =', '--features', 'none'];
try {
  await mkdir(bin);
  await copyFile(path.resolve(`cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`), exe('zel'));
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/build-engine.rs', '-o', exe('docker')], { stdio: 'pipe' });
  for (const name of ['podman', 'nerdctl']) await copyFile(exe('docker'), exe(name));
  for (const renderer of ['portable', 'hosted']) {
    await write(`${renderer}/content/${renderer === 'portable' ? 'index.md' : 'site.yaml'}`, renderer === 'portable' ? '# Multi\n' : 'name: Multi\n');
    for (const kind of ['static', 'next']) {
      const config = `version: 1\nrenderer: ${renderer}\nimage: {name: registry.example/docs:test, platform: all}\npublish: {action: push, image: ${kind}}\n`;
      await write(`${renderer}/zel.yaml`, config);
      for (const [runtime, selector] of [['docker', 'DOCKER_CONTEXT'], ['podman', 'CONTAINER_CONNECTION'], ['nerdctl', 'CONTAINERD_NAMESPACE']]) {
        await reset();
        run(['pub', renderer, ...args, '--dry-run']); assert.deepEqual(await calls(), []);
        run(['pub', renderer, ...args], 0, { DOCKER_CONTEXT: '', [selector]: 'fixture' });
        const log = await calls();
        const build = log.find(call => call.includes('build'));
        assert.equal(build[build.indexOf('--platform') + 1], 'linux/amd64,linux/arm64');
        assert.equal(build[build.indexOf('--build-arg') + 1], 'LABEL=日本語 $(literal) =');
        assert(!log.some(call => call[0] === 'push'), 'No second, host-only push');
        if (runtime === 'docker') {
          assert.deepEqual(build.slice(0, 3), ['buildx', 'build', '--push']);
          assert(log.findIndex(call => call[1] === 'version') < log.indexOf(build));
        } else if (runtime === 'nerdctl') {
          assert.equal(build[build.indexOf('--output') + 1], 'type=image,name=registry.example/docs:test,push=true');
        } else {
          const manifest = build[build.indexOf('--manifest') + 1];
          assert(/^zel-multi-[a-f0-9]+$/.test(manifest) && !build.includes('-t'));
          assert.deepEqual(log.slice(-2), [['manifest', 'push', '--all', manifest, 'docker://registry.example/docs:test'], ['manifest', 'rm', manifest]]);
        }
        assert.equal(await read(`${renderer}/zel.yaml`), config);
      }
      // Saved "all" is deliberately overridden by an ordinary local target.
      run(['si', renderer, '--platform', 'linux/amd64', ...args]);
      assert(!(await read('build-args')).includes('buildx'));
      await reset();
      for (const command of [['si'], ['ni'], ...(renderer === 'hosted' ? [['image', 'build'], ['build', '--target', 'image']] : [])]) {
        assert.match(run([...command, renderer], 2).stderr, /requires registry publication/);
      }
      assert.deepEqual(await calls(), []);
    }
  }
  for (const platform of ['all', 'linux/amd64,linux/arm64', 'linux/arm64,linux/amd64']) run(['pub', 'hosted', '--platform', platform]);
  await reset();
  assert.match(run(['pub', 'hosted'], 2, { ZELLIJ_NO_BUILDX: 'yes' }).stderr, /requires Docker Buildx/);
  assert(!(await calls()).some(call => call.includes('build')));
  for (const selector of ['DOCKER_CONTEXT', 'CONTAINERD_NAMESPACE']) {
    await reset();
    const failed = run(['pub', 'hosted'], 3, { DOCKER_CONTEXT: '', [selector]: 'fixture', ZELLIJ_BUILD_FAIL: 'yes' });
    assert.match(failed.stderr, /may have uploaded layers/);
    assert(!(await calls()).some(call => call[0] === 'push'));
  }
  for (const failure of ['ZELLIJ_BUILD_FAIL', 'ZELLIJ_PUSH_FAIL']) {
    await reset();
    run(['pub', 'hosted'], 3, { DOCKER_CONTEXT: '', CONTAINER_CONNECTION: 'fixture', [failure]: 'yes' });
    const log = await calls();
    assert.equal(log.at(-1)[1], 'rm');
    assert.equal(log.some(call => call[0] === 'manifest' && call[1] === 'push'), failure === 'ZELLIJ_PUSH_FAIL');
  }
  await write('source/mkdocs.yml', 'site_name: Imported\nnav: [index.md]\n');
  await write('source/docs/index.md', '# Imported\n');
  for (const kind of ['static', 'next']) {
    const command = ['ship', 'source', '--out', `shipped-${kind}`, '--image', kind, '--tag', 'registry.example/docs:ship', '--platform', 'all'];
    run([...command, '--dry-run']); await assert.rejects(stat(path.join(root, `shipped-${kind}`)), { code: 'ENOENT' });
    run(command);
    assert.match(await read(`shipped-${kind}/zel.yaml`), /platform: all/);
  }
  run(['ship', 'source', '--out', 'retained', '--image', 'next', '--tag', 'registry.example/docs:failed', '--platform', 'all'], 3, { ZELLIJ_BUILD_FAIL: 'yes' });
  assert.match(await read('retained/zel.yaml'), /platform: all/);
  await reset();
  run(['netlify', 'hosted', '--site-id', 'example.netlify.app', '--platform', 'all'], 2);
  assert.deepEqual(await calls(), []);
  assert.equal(await read('source/docs/index.md'), '# Imported\n');
  console.log('Combined image publication: all runtimes, renderers, image kinds, aliases, failures and source preservation passed.');
} finally { await rm(root, { recursive: true, force: true }); }
