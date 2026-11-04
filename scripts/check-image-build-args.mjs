// Native forwarding acceptance. Uses fake clients, with no daemon or browser.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const builtZel = path.resolve(`cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zel-image-args-'));
const bin = path.join(root, 'bin');
// Windows searches the parent's executable directory before system folders and
// PATH. Keep zel beside the fake clients so an installed Docker cannot win.
const zel = path.join(bin, process.platform === 'win32' ? 'zel.exe' : 'zel');
const env = { ...process.env, ZELLIJ_BUILD_FIXTURE: root };
const selectors = ['DOCKER_CONTEXT', 'DOCKER_HOST', 'CONTAINER_CONNECTION', 'CONTAINER_HOST', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE'];
for (const key of Object.keys(env)) if (['PATH', 'NETLIFY_SITE_ID', 'NETLIFY_AUTH_TOKEN', ...selectors].includes(key.toUpperCase())) delete env[key];
env.PATH = bin;
const read = file => readFile(path.join(root, file), 'utf8');
const write = async (file, text) => { const target = path.join(root, file); await mkdir(path.dirname(target), { recursive: true }); await writeFile(target, text); };
const run = (args, code = 0, extra = {}) => {
  const result = spawnSync(zel, args, { cwd: root, env: { ...env, CONTAINER_CONNECTION: 'fixture', ...extra }, encoding: 'utf8', timeout: 30000 });
  assert.equal(result.status, code, `${args}\n${result.stdout}\n${result.stderr}\n${result.error ?? ''}`);
  return result;
};
const absent = async file => assert.rejects(stat(path.join(root, file)), { code: 'ENOENT' });
const value = "日本語 '$(touch INJECTED)`echo literal` = ";
const flags = ['--build-arg', 'DOCS_LABEL=old', '--build-arg', `DOCS_LABEL=${value}`, '--build-arg', 'EMPTY=', '--build-arg', 'HTTP_PROXY=http://proxy.example'];
async function captured() {
  const args = (await read('build-args')).split('\0');
  assert.deepEqual(args.flatMap((arg, i) => arg === '--build-arg' ? [args[i + 1]] : []), [`DOCS_LABEL=${value}`, 'EMPTY=', 'HTTP_PROXY=http://proxy.example']);
  const recipe = await read('recipe');
  assert.equal(recipe.split('ARG DOCS_LABEL\n').length, 2);
  assert(recipe.includes('ARG EMPTY\n'));
  assert(!recipe.includes(value) && !recipe.includes('ARG HTTP_PROXY') && !recipe.includes('ENV DOCS_LABEL'));
}
try {
  await mkdir(bin);
  await copyFile(builtZel, zel);
  const exe = name => path.join(bin, name + (process.platform === 'win32' ? '.exe' : ''));
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/build-engine.rs', '-o', exe('podman')], { stdio: 'pipe' });
  for (const name of ['docker', 'nerdctl']) await copyFile(exe('podman'), exe(name));
  for (const renderer of ['portable', 'hosted']) {
    await write(`${renderer}/zel.yaml`, `version: 1\nrenderer: ${renderer}\n`);
    await write(`${renderer}/content/${renderer === 'portable' ? 'index.md' : 'site.yaml'}`, renderer === 'portable' ? '# Docs\n' : 'name: Hosted\n');
  }
  // All runtime clients receive literal argv, including empty/Unicode values.
  for (const selector of ['CONTAINER_CONNECTION', 'DOCKER_CONTEXT', 'CONTAINERD_NAMESPACE']) {
    await rm(path.join(root, 'probed'), { force: true });
    run(['si', 'portable', ...flags], 0, { CONTAINER_CONNECTION: '', [selector]: 'fixture' });
    assert.equal(await read('probed'), 'yes', 'The selected probe must reach the isolated fake client');
    await captured();
  }
  for (const [command, site] of [
    [['si'], 'hosted'], [['image', 'static'], 'portable'], [['image', 'static'], 'hosted'],
    [['ni'], 'portable'], [['ni'], 'hosted'], [['image', 'next'], 'portable'],
    [['image', 'build'], 'hosted'], [['b', '--target', 'image'], 'hosted'],
  ]) {
    run([...command, site, ...flags]); await captured();
  }
  for (const [site, kind] of [['portable', 'static'], ['hosted', 'static'], ['hosted', 'next']]) {
    for (const command of [['idf'], ['image', 'dockerfile']]) {
      await rm(path.join(root, 'probed'), { force: true });
      const result = run([...command, site, '--kind', kind, ...flags]);
      assert(result.stdout.includes('ARG DOCS_LABEL\n') && !result.stdout.includes(value));
      await absent('probed');
    }
  }
  for (const renderer of ['portable', 'hosted']) {
    for (const kind of ['static', 'next']) {
      const config = `version: 1\nrenderer: ${renderer}\nimage: {name: 'example/docs:test'}\npublish: {action: push, image: ${kind}}\n`;
      await write(`${renderer}/zel.yaml`, config);
      for (const command of [['pub', renderer], ['publish', renderer], ['deploy', '--site-root', renderer]]) {
        const plan = run([...command, ...flags, '--dry-run']).stdout;
        assert(plan.includes('Build arguments DOCS_LABEL, EMPTY, HTTP_PROXY') && !plan.includes(value));
        run([...command, ...flags]); await captured();
        assert.equal(await read('pushed'), 'push\nexample/docs:test');
        assert.equal(await read(`${renderer}/zel.yaml`), config);
      }
    }
  }
  await rm(path.join(root, 'pushed'));
  run(['pub', 'hosted', ...flags], 3, { ZELLIJ_BUILD_FAIL: 'yes' }); await absent('pushed');
  run(['ni', 'hosted']);
  assert(!(await read('recipe')).includes('ARG DOCS_LABEL'));
  assert(!(await read('build-args')).includes('--build-arg'));

  await write('source/mkdocs.yml', 'site_name: Imported\nnav: [index.md]\n');
  await write('source/docs/index.md', '# Imported\n');
  for (const kind of ['static', 'next']) {
    const args = ['ship', 'source', '--out', `shipped-${kind}`, '--image', kind, '--tag', 'example/ship:test', ...flags];
    run([...args, '--dry-run']); await absent(`shipped-${kind}`);
    assert.match(run(args).stdout, /Image build arguments were not saved/); await captured();
    assert(!(await read(`shipped-${kind}/zel.yaml`)).includes('DOCS_LABEL'));
  }
  const failure = run(['ship', 'source', '--out', 'retained', '--image', 'next', '--tag', 'example/ship:test', ...flags], 3, { ZELLIJ_BUILD_FAIL: 'yes' });
  assert.match(failure.stderr, /Image build arguments were not saved/);
  assert((await read('retained/zel.yaml')).includes('renderer: hosted'));
  await rm(path.join(root, 'probed'), { force: true });
  for (const renderer of ['portable', 'hosted']) {
    for (const command of [['build', renderer], ['netlify', renderer, '--site-id', 'example.netlify.app'], ['deploy', 'netlify', renderer, '--site-id', 'example.netlify.app', '--dry-run']]) {
      assert.match(run([...command, ...flags], 2).stderr, /requires an image/);
    }
  }
  for (const argument of ['MISSING', '1BAD=value', 'ZELLIJ_OUTPUT=export', 'TARGETARCH=arm64', 'KEY=value\nRUN nope']) run(['ni', 'hosted', '--build-arg', argument], 2);
  run(['ship', 'source', '--out', 'rejected', '--provider', 'netlify', '--site-id', 'example.netlify.app', ...flags], 2);
  await absent('rejected'); await absent('probed'); await absent('INJECTED');
  assert.equal(await read('source/docs/index.md'), '# Imported\n');
  assert.equal(await read('portable/content/index.md'), '# Docs\n');
  assert.equal(await read('hosted/content/site.yaml'), 'name: Hosted\n');
  console.log('Image build arguments: runtime argv, aliases, recipes, delivery, ship, dry plans and rejection passed.');
} finally { await rm(root, { recursive: true, force: true }); }
