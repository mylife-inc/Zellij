// Verify actual CLI output against engine routes, MDX rendering and asset bytes.
// With --export, build both imports and inspect their generated HTML as well.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
import { build } from 'esbuild';
import { renderToStaticMarkup } from 'react-dom/server';
import { compileMDX } from 'next-mdx-remote/rsc';

const repo = fileURLToPath(new URL('../', import.meta.url));
// The published module graph is consumed by Next's bundler. Bundle the same
// source for this Node-only integration check without changing package exports.
const cache = await mkdtemp(path.join(repo, 'node_modules/.conversion-check-'));
const helper = path.join(cache, 'engine.mjs');
const require = createRequire(import.meta.url);
await build({
  stdin: { contents: `
    export { FolderContentSource } from './engine/packages/zellij/src/content/folder-source';
    export { loadSite } from './engine/packages/zellij/src/content/site';
    export { mdxOptions } from './engine/packages/zellij/src/mdx/pipeline';
    export { mdxComponents } from './engine/packages/zellij/src/mdx/components';
  `, resolveDir: repo },
  bundle: true, packages: 'external', platform: 'node', format: 'esm', jsx: 'automatic', outfile: helper,
  plugins: [{ name: 'next-node-paths', setup(builder) {
    builder.onResolve({ filter: /^next\// }, ({ path: name }) => ({ path: require.resolve(name), external: true }));
  } }],
});
const { FolderContentSource, loadSite, mdxOptions, mdxComponents } = await import(pathToFileURL(helper));
await rm(cache, { recursive: true });
const [binary, target, mode] = process.argv.slice(2);
assert(binary && target, 'Pass a zel binary and a new fixture directory');
const root = path.resolve(target);
await mkdir(root);
const zel = path.resolve(binary);
const asset = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" /></svg>\n';
const pdf = Buffer.from([0x25, 0x50, 0x44, 0x46, 0, 255, 254]);
for (const format of ['mkdocs', 'docusaurus']) {
  const source = path.join(root, `${format}-source`);
  const content = path.join(root, `${format}-content`);
  async function write(file, body) {
    await mkdir(path.dirname(path.join(source, file)), { recursive: true });
    await writeFile(path.join(source, file), body);
  }
  if (format === 'mkdocs') {
    await write('mkdocs.yml', 'site_name: Converted docs\nnav:\n  - index.md\n  - 02-Setup/APIReference.md\n  - 03-Café/index.md\n');
  } else {
    await write('docusaurus.config.js', "module.exports = {title: 'Converted docs'};");
    await write('sidebars.js', "module.exports = {docs: ['index', '02-Setup/APIReference', '03-Café/index']};");
    await write('static/img/static.svg', asset);
  }
  await write(`docs/02-Setup/APIReference.${format === 'mkdocs' ? 'md' : 'mdx'}`, '# Reference\n\n## Install\n\n[Home](../index.md)\n');
  await write('docs/03-Café/index.md', '# Café\n\n[Reference](../02-Setup/APIReference.md)\n');
  await write('docs/media/logo.svg', asset);
  await write('docs/media/Q3 Report.pdf', pdf);
  await write('docs/index.md', `# Welcome\n\n[Install **now**][install]\n\n[Café](03-Caf%C3%A9/index.md)\n\n![Diagram][logo]\n\n[Download](<media/Q3 Report.pdf>)\n\n[install]: 02-Setup/APIReference.md?view=all#install "Install guide"\n[logo]: media/logo.svg\n\n${format === 'docusaurus' ? '![Static](/img/static.svg)\n' : ''}`);
  execFileSync(zel, ['convert', source, '--out', content, '--from', format], { stdio: 'inherit' });
  // A non-empty deployment prefix exercises the ordinary Markdown image path.
  await writeFile(path.join(content, 'site.yaml'), '\nbasePath: /products/docs\n', { flag: 'a' });
  execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'check', content], { stdio: 'inherit' });
  const site = await loadSite(new FolderContentSource({ contentDir: content }));
  assert.deepEqual([...site.routes].sort(), ['/', '/guide', '/guide/cafe', '/guide/setup/api-reference']);
  const rendered = new Map();
  for (const page of site.guidePages) {
    const result = await compileMDX({ source: page.body, options: await mdxOptions(), components: mdxComponents(site) });
    rendered.set(page.route, renderToStaticMarkup(result.content));
  }
  const home = rendered.get('/guide');
  assert(home.includes('href="/guide/setup/api-reference?view=all#install"'));
  assert(home.includes('href="/guide/cafe"'));
  assert(rendered.get('/guide/setup/api-reference').includes('id="install"'));
  assert(home.includes('src="/products/docs/_zellij/assets/imported/docs/media/logo.svg"'));
  assert(home.includes('href="/_zellij/assets/imported/docs/media/Q3%20Report.pdf"'));
  assert.deepEqual(await readFile(path.join(content, 'assets/imported/docs/media/Q3 Report.pdf')), pdf);
  if (mode === '--export') {
    const output = path.join(root, `${format}-site`);
    execFileSync(zel, ['build', content, '--out', output], { stdio: 'inherit' });
    const html = await readFile(path.join(output, 'guide.html'), 'utf8');
    for (const reference of [
      'href="/products/docs/guide/setup/api-reference?view=all#install"',
      'href="/products/docs/guide/cafe"',
      'src="/products/docs/_zellij/assets/imported/docs/media/logo.svg"',
      'href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"',
    ]) assert(html.includes(reference), `Export missing ${reference}`);
    const installed = await readFile(path.join(output, 'guide/setup/api-reference.html'), 'utf8');
    assert(installed.includes('id="install"'));
    assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/media/logo.svg'), 'utf8'), asset);
    assert.deepEqual(await readFile(path.join(output, '_zellij/assets/imported/docs/media/Q3 Report.pdf')), pdf);
    if (format === 'docusaurus') {
      assert(html.includes('src="/products/docs/_zellij/assets/imported/static/img/static.svg"'));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/static/img/static.svg'), 'utf8'), asset);
    }
  }
  console.log(`${format}: converted routes, heading fragment, rendered URLs and asset bytes verified.`);
}
