// Exercise the packaged CLI, not just imported helpers. Optional container
// export proves every indexed URL/anchor exists in the generated site.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = fileURLToPath(new URL('../', import.meta.url));
const root = await mkdtemp(path.join(tmpdir(), 'zellij-preparation-'));
const app = path.join(root, 'wrapper');
const content = path.join(root, 'content = external');
const packaged = path.join(root, 'node_modules/@shebka/zellij');
const cli = path.join(packaged, 'bin/zellij.mjs');
const [binary, mode] = process.argv.slice(2);
const body = '# Install\n\n## Install\n\nFirst passage.\n\n## Install\n\nSecond passage.\n\n## [Use the CLI](/manual)\n\nThird passage.\n';
const files = {
  'site.yaml': 'name: Preparation fixture\nhome: 01-guide\n',
  '01-guide/_sidebar.yaml': 'route: /manual\nnav: [index, setup]\n',
  '01-guide/index.mdx': body,
  '01-guide/setup.mdx': '---\nroute: /custom/start\n---\n# Setup\n\nInstructions.\n',
  'assets/logo.svg': '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>',
};
const output = path.join(app, 'public/_zellij');
const run = (args) => execFileSync(process.execPath, [cli, ...args], { cwd: app, stdio: 'inherit' });
try {
  await mkdir(app);
  for (const [name, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(content, name)), { recursive: true });
    await writeFile(path.join(content, name), text);
  }
  // No engine dependency tree: these are the self-contained artifacts used by
  // content-only/container commands. Optional math gracefully stays unavailable.
  await mkdir(path.join(packaged, 'bin'), { recursive: true });
  await mkdir(path.join(packaged, 'dist/cli'), { recursive: true });
  await writeFile(path.join(packaged, 'package.json'), '{"type":"module"}');
  await copyFile(path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), cli);
  for (const name of ['check', 'sync-assets', 'write-search-index']) {
    await copyFile(path.join(repo, `engine/packages/zellij/dist/cli/${name}-bundle.js`), path.join(packaged, `dist/cli/${name}-bundle.js`));
  }
  run(['prepare', '--content=../content = external']);
  const entries = JSON.parse(await readFile(path.join(output, 'search-index.json'), 'utf8'));
  assert.deepEqual(entries.map((entry) => entry.route), [
    '/manual', '/manual#install', '/manual#install-1', '/manual#use-the-cli', '/custom/start',
  ]);
  assert.equal(await readFile(path.join(output, 'assets/logo.svg'), 'utf8'), files['assets/logo.svg']);
  run(['check', content]);

  // The installed engine must use the same optional math parser as rendering.
  // Braced LaTeX would be an invalid MDX expression without that parser.
  const mathBody = body + '\n## $\\sum_{k=1}^{n} k$\n\nA mathematical passage.\n';
  await writeFile(path.join(content, '01-guide/index.mdx'), mathBody);
  execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'search-index', '--content', content], { cwd: app, stdio: 'inherit' });
  const withMath = JSON.parse(await readFile(path.join(output, 'search-index.json'), 'utf8'));
  assert(withMath.some((entry) => entry.text === 'A mathematical passage.'));
  await writeFile(path.join(content, '01-guide/index.mdx'), body);
  run(['search-index', '--content', content]);
  await writeFile(path.join(app, 'legacy.yaml'), 'product: { name: Fixture }\ncontent: "../content = external"\n');
  run(['search-index', '--config', 'legacy.yaml']);
  assert.deepEqual(JSON.parse(await readFile(path.join(output, 'search-index.json'), 'utf8')), entries);
  for (const args of [['prepare', '--content'], ['search-index', '--unknown', 'value']]) {
    assert.notEqual(spawnSync(process.execPath, [cli, ...args], { cwd: app }).status, 0);
  }

  if (mode === '--export') {
    assert(binary, 'Pass a zel binary before --export');
    const exported = path.join(root, 'exported');
    execFileSync(path.resolve(binary), ['build', content, '--out', exported], { stdio: 'inherit' });
    const actual = JSON.parse(await readFile(path.join(exported, '_zellij/search-index.json'), 'utf8'));
    assert.deepEqual(actual, entries);
    for (const entry of actual) {
      const [route, anchor] = entry.route.split('#');
      const html = await readFile(path.join(exported, `${route.slice(1)}.html`), 'utf8');
      if (anchor) assert(html.includes(`id="${anchor}"`), `Missing exported anchor ${entry.route}`);
    }
    assert((await readFile(path.join(exported, 'index.html'), 'utf8')).includes('/manual'));
  }

  await rm(path.join(content, 'assets'), { recursive: true });
  run(['sync-assets', '--content', content]);
  await assert.rejects(readFile(path.join(output, 'assets/logo.svg')), { code: 'ENOENT' });
  assert.deepEqual(JSON.parse(await readFile(path.join(output, 'asset-manifest.json'), 'utf8')), {});
  console.log('Packaged preparation: canonical routes/anchors, custom paths, legacy config, errors and asset deletion passed.');
} finally {
  await rm(root, { recursive: true, force: true });
}
