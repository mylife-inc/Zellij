// Check real Rust/Node scaffolds with engine schemas and the rendered Hero.
// --container-check also exercises default CLI discovery against the CI image.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { build } from 'esbuild';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';

const [binary, target, mode] = process.argv.slice(2);
assert(binary && target, 'Pass a zel binary and a new fixture directory');
const zel = path.resolve(binary);
const root = path.resolve(target);
await mkdir(root);
const repo = fileURLToPath(new URL('../', import.meta.url));
const nodeCli = path.join(repo, 'engine/packages/zellij/bin/zellij.mjs');
const require = createRequire(import.meta.url);
const cache = await mkdtemp(path.join(repo, 'node_modules/.scaffold-check-'));
let engine;
try {
  const helper = path.join(cache, 'engine.mjs');
  await build({
    stdin: { contents: `
      export { FolderContentSource } from './engine/packages/zellij/src/content/folder-source';
      export { loadSite } from './engine/packages/zellij/src/content/site';
      export { Hero } from './engine/packages/zellij/src/sections/components/hero';
    `, resolveDir: repo },
    bundle: true, packages: 'external', platform: 'node', format: 'esm', jsx: 'automatic', outfile: helper,
    plugins: [{ name: 'next-node-paths', setup(builder) {
      builder.onResolve({ filter: /^next\// }, ({ path: name }) => ({ path: require.resolve(name), external: true }));
    } }],
  });
  engine = await import(pathToFileURL(helper));
} finally {
  await rm(cache, { recursive: true });
}
const { FolderContentSource, loadSite, Hero } = engine;

async function load(content, allowEmpty = false) {
  const warnings = [];
  const site = await loadSite(new FolderContentSource({ contentDir: content, onWarning: (w) => warnings.push(w) }));
  assert.deepEqual(warnings, [], content);
  if (!allowEmpty) assert.deepEqual(site.warnings, [], content);
  return site;
}

async function verify(content, name, guide) {
  const site = await load(content);
  assert.equal(site.config.name, name);
  assert.equal(site.config.footer.legal, `© ${name}`);
  assert.equal(site.pages[0].title, name);
  const hero = site.pages[0].sections.find((section) => section.type === 'hero');
  assert.equal(hero.headline, name);
  assert.deepEqual([...site.routes].sort(), guide ? ['/', '/guide'] : ['/']);
  const html = renderToStaticMarkup(createElement(Hero, { section: hero, site, defaultMotion: 'none' }));
  const links = [...html.matchAll(/href="([^"]+)"/g)].map((match) => match[1]);
  assert.deepEqual(links, guide ? ['/guide'] : [], content);
  for (const link of links) assert(site.routes.has(link), `Broken starter link: ${link}`);
}

const names = ['Acme: Docs', 'A # comment', '"quoted" \\ path', 'TRUE', 'Null', '42', 'العربية Café', 'line\nnext\r\ntab\tend', 'a\u0085b\u2028c\u2029d'];
const variants = [[], ['--guide'], ['--minimal'], ['--minimal', '--guide', '--theme', 'cupertino']];
for (const [index, name] of names.entries()) {
  for (const [variant, flags] of variants.entries()) {
    const content = path.join(root, `rust-${index}-${variant}`);
    execFileSync(zel, ['new', 'docs', content, '--name', name, ...flags]);
    await verify(content, name, flags.includes('--guide'));
  }
  const content = path.join(root, `node-${index}`);
  execFileSync(process.execPath, [nodeCli, 'init', content, '--name', name, '--content-only']);
  await verify(content, name, true);

  const guide = path.join(root, `guide-${index}`);
  execFileSync(zel, ['new', 'guide', 'manual', '--dir', guide, '--title', name]);
  const site = await load(guide);
  assert.equal(site.guides[0].title, name);
  assert.equal(site.guidePages[0].title, name);
}
console.log('36 Rust starters, 9 Node starters and 9 guide titles: exact metadata and rendered links verified.');

if (mode === '--container-check') {
  for (const location of ['', 'content', 'docs', 'documentation']) {
    const cwd = path.join(root, `discover-${location || 'root'}`);
    const content = path.join(cwd, location);
    execFileSync(zel, ['new', 'guide', 'manual', '--dir', content, '--title', 'Guide: "Start"']);
    const output = execFileSync(zel, ['check'], { cwd, encoding: 'utf8' });
    assert.match(output, /guides\s+1/);
    console.log(`CLI discovery + container check: ${location || 'root'} guide-only site passed.`);
  }
  const menuOnly = path.join(root, 'menu-only');
  await mkdir(menuOnly);
  await writeFile(path.join(menuOnly, 'menu.yaml'), '[]\n');
  await load(menuOnly, true);
  const output = spawnSync(zel, ['check'], { cwd: menuOnly, encoding: 'utf8' });
  assert.equal(output.status, 0, output.stderr);
  assert.match(output.stdout + output.stderr, /no front page/); // Valid, but deliberately has nothing to render.
  console.log('CLI discovery + container check: menu-only root accepted with the expected empty-site warning.');
}
