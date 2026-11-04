// Native CLI forwarding and rejection acceptance. No daemon or GUI browser.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const zel = path.resolve(`cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-build-options-'));
const bin = path.join(root, 'bin');
const env = { ...process.env, ZELLIJ_BUILD_FIXTURE: root };
const replaced = new Set(['PATH', 'DOCKER_CONTEXT', 'DOCKER_HOST', 'CONTAINER_CONNECTION', 'CONTAINER_HOST', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE']);
for (const key of Object.keys(env)) if (replaced.has(key.toUpperCase())) delete env[key];
Object.assign(env, { PATH: bin, CONTAINER_CONNECTION: 'build-fixture' });
const read = name => readFile(path.join(root, name), 'utf8');
const write = async (name, value) => { const file = path.join(root, name); await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, value); };
const run = (args, status = 0, extra = {}) => {
  const result = spawnSync(zel, args, { cwd: root, env: { ...env, ...extra }, encoding: 'utf8', timeout: 30000 });
  assert.equal(result.status, status, `${args.join(' ')}\n${result.stdout}\n${result.stderr}\n${result.error ?? ''}`);
  return result;
};
const decode = text => [...text.matchAll(/(?:^|\n)(?:ENV )?ZELLIJ_BUILD_OVERRIDES=([0-9a-f]+)(?:\n|$)/g)].map(match => JSON.parse(Buffer.from(match[1], 'hex').toString('utf8')));
const settings = { version: 1, basePath: "/日本語/'$(literal)`value`", siteUrl: 'https://docs.example', noCheck: true };
const flags = ['--base-path', settings.basePath + '/', '--site-url', 'https://DOCS.example:443/', '--no-check'];
const authored = 'name: Hosted\nbasePath: /authored\nsiteUrl: https://old.example\n';
try {
  await mkdir(bin);
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/build-engine.rs', '-o', path.join(bin, process.platform === 'win32' ? 'podman.exe' : 'podman')], { stdio: 'pipe' });
  await write('hosted/zel.yaml', 'version: 1\nrenderer: hosted\n');
  await write('hosted/content/site.yaml', authored);
  await write('portable/zel.yaml', 'version: 1\nrenderer: portable\n');
  await write('portable/content/index.md', '# Documentation\n');
  for (const command of ['build', 'b']) {
    run([command, 'hosted', '--out', 'export', '--features', 'none', ...flags]);
    const args = await read('args');
    assert.deepEqual(decode(args), [settings]);
    assert(args.includes(':/app/content:ro'));
    assert(args.includes('features "none" /app'));
    assert(!args.includes(' check /app/content'));
    assert(args.indexOf('buildOverridesVersion') < args.indexOf('npm install'));
    assert(args.includes('|| exit 95; '));
  }
  run(['build', 'hosted', '--out', 'export', '--base-path', '/']);
  let args = await read('args');
  assert.deepEqual(decode(args), [{ version: 1, basePath: '', noCheck: false }]);
  assert(args.includes(' check /app/content'));
  const previous = await read('export/index.html');
  run(['build', 'hosted', '--out', 'export', ...flags], 3, { ZELLIJ_BUILD_FAIL: 'yes' });
  assert.equal(await read('export/index.html'), previous);
  run(['build', 'hosted', '--out', 'export']);
  args = await read('args');
  assert.deepEqual(decode(args), []);
  assert(!args.includes('buildOverridesVersion'));
  assert(args.includes(' check /app/content'));

  for (const [command, target, copies] of [
    [['image', 'static'], 'hosted', 1], [['si'], 'hosted', 1],
    [['image', 'next'], 'hosted', 2], [['ni'], 'hosted', 2],
    [['image', 'build'], 'hosted', 2], [['ni'], 'portable', 2],
    [['build'], 'hosted', 2],
  ]) {
    run([...command, target, '--tag', 'example/site:test', ...(command[0] === 'build' ? ['--target', 'image'] : []), ...flags]);
    const recipe = await read('recipe');
    assert.deepEqual(decode(recipe), Array(copies).fill(settings));
    assert(!recipe.includes(settings.basePath), 'Values stay encoded in Dockerfiles');
    assert(!recipe.includes('RUN npx zellij check content'));
    assert(recipe.includes('RUN npm run build'));
    assert(recipe.indexOf('buildOverridesVersion') < recipe.indexOf('npm install'));
  }
  for (const command of [['image', 'dockerfile'], ['idf']]) {
    for (const kind of ['static', 'next']) {
      const result = run([...command, 'hosted', '--kind', kind, ...flags]);
      assert.deepEqual(decode(result.stdout), Array(kind === 'next' ? 2 : 1).fill(settings));
    }
  }
  const defaults = run(['idf', 'hosted', '--kind', 'next']).stdout;
  assert(defaults.includes('RUN npx zellij check content'));
  assert.deepEqual(decode(defaults), []);
  await rm(path.join(root, 'args')); await rm(path.join(root, 'recipe'));
  await rm(path.join(root, 'probed'));
  for (const command of [['build'], ['si'], ['idf', '--kind', 'static']]) {
    for (const option of [['--base-path', '/docs'], ['--site-url', 'https://docs.example'], ['--no-check']]) {
      const result = run([...command, 'portable', ...option], 2);
      assert.match(result.stderr, /hosted/);
    }
  }
  for (const option of [['--base-path', '/../docs'], ['--site-url', 'https://docs.example/path'], ['--site-url', 'https://user:password@docs.example']]) {
    run(['build', 'hosted', ...option], 2);
  }
  for (const file of ['args', 'recipe', 'probed', 'portable/site', 'hosted/site']) await assert.rejects(stat(path.join(root, file)), { code: 'ENOENT' });
  assert.equal(await read('hosted/content/site.yaml'), authored);
  assert.equal(await read('portable/content/index.md'), '# Documentation\n');
  console.log('Hosted build overrides: native aliases, recipes, root reset, preflight selection, rejection and preservation passed.');
} finally { await rm(root, { recursive: true, force: true }); }
