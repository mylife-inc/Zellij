// Verify actual CLI output against engine routes, MDX rendering and asset bytes.
// With --export, use the conversion.Dockerfile fixture image via ZELLIJ_IMAGE
// to build both imports with matching Next basePath and inspect generated HTML.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
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
    export { extractToc } from './engine/packages/zellij/src/guide/toc';
    export { indexGuidePage } from './engine/packages/zellij/src/search/build-index';
    export { deepCheck } from './engine/packages/zellij/src/cli/deep-check';
  `, resolveDir: repo },
  bundle: true, packages: 'external', platform: 'node', format: 'esm', jsx: 'automatic', outfile: helper,
  // Shared CJS config uses Node builtins; provide require in this ESM bundle,
  // just as the packaged Node CLI does.
  banner: { js: "import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);" },
  plugins: [{ name: 'next-node-paths', setup(builder) {
    builder.onResolve({ filter: /^next\// }, ({ path: name }) => ({ path: require.resolve(name), external: true }));
  } }],
});
const { FolderContentSource, loadSite, mdxOptions, mdxComponents, extractToc, indexGuidePage, deepCheck } = await import(pathToFileURL(helper));
await rm(cache, { recursive: true });
const [binary, target, mode] = process.argv.slice(2);
assert(binary && target, 'Pass a zel binary and a new fixture directory');
const root = path.resolve(target);
await mkdir(root);
const zel = path.resolve(binary);
const asset = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" /></svg>\n';
const assetHash = createHash('sha256').update(asset).digest('hex').slice(0, 8);
const pdf = Buffer.from([0x25, 0x50, 0x44, 0x46, 0, 255, 254]);
for (const format of ['mkdocs', 'docusaurus']) {
  const source = path.join(root, `${format}-source`);
  const destination = path.join(root, `${format}-content`);
  const configured = format === 'docusaurus';
  const content = configured ? path.join(destination, 'content') : destination;
  async function write(file, body) {
    await mkdir(path.dirname(path.join(source, file)), { recursive: true });
    await writeFile(path.join(source, file), body);
  }
  if (format === 'mkdocs') {
    await write('mkdocs.yml', 'site_name: Converted docs\nnav:\n  - index.md\n  - API navigation: 02-Setup/APIReference.md\n  - 03-Café/index.md\n');
  } else {
    await write('docusaurus.config.js', "module.exports = {title: 'Converted docs'};");
    await write('sidebars.js', "module.exports = {docs: ['index', {type: 'doc', id: 'Setup/custom', label: 'Overridden'}, 'Café/index']};");
    await write('static/img/static.svg', asset);
  }
  const titleId = format === 'mkdocs' ? '{#reference}' : '{/* #reference */}';
  const installId = format === 'mkdocs' ? '{#stable-install}' : '<!-- #stable-install -->';
  await write(`docs/02-Setup/APIReference.${format === 'mkdocs' ? 'md' : 'mdx'}`, `\uFEFF---\r\nid: custom\r\ntitle: Authored reference title\r\ndescription: Authored reference description\r\nsidebar_label: API navigation\r\nslug: /source-only-slug\r\n...\r\n# Reference ${titleId}\n\n## Install\n\n[Home](../index.md#welcome)\n\n## Install **again** ${installId}\n\nA searchable installation passage.\n\nSetext [home](../index.md#welcome) {#setext}\n------\n\nA searchable setext passage.\n\n## Café {#café}\n\nUnicode passage.\n\n[Self](#stable-install)\n\n[Unicode](#caf%C3%A9)\n`);
  await write('docs/03-Café/index.md', '# Café\n\n[Reference](../02-Setup/APIReference.md)\n');
  await write('docs/media/logo.svg', asset);
  await write('docs/media/Q3 Report.pdf', pdf);
  await write('docs/index.md', `# Welcome {#welcome}\n\n[Install **now**][install]\n\n[Stable install](02-Setup/APIReference.md#stable-install)\n\n[Reference title](02-Setup/APIReference.md#reference)\n\n[Setext](02-Setup/APIReference.md#setext)\n\n[Café](03-Caf%C3%A9/index.md)\n\n![Diagram][logo]\n\n[Download](<media/Q3 Report.pdf>)\n\n[install]: 02-Setup/APIReference.md?view=all#install "Install guide"\n[logo]: media/logo.svg\n\n${format === 'docusaurus' ? '![Static](/img/static.svg)\n' : ''}\n\n\`\`\`\`md\n\`\`\`\n# Literal {#literal}\n!!! note\n\`\`\`\`\n`);
  execFileSync(zel, [configured && mode === '--export' ? 'migrate' : 'convert', source, '--out', destination, '--from', format, '--layout', configured ? 'site' : 'content'], { stdio: 'inherit' });
  // A non-empty deployment prefix exercises the ordinary Markdown image path.
  await writeFile(path.join(content, 'site.yaml'), '\nbasePath: /products/docs\n', { flag: 'a' });
  execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'check', content], { stdio: 'inherit' });
  const site = await loadSite(new FolderContentSource({ contentDir: content }));
  assert.deepEqual(await deepCheck(site, content), { errors: [], warnings: [] });
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
  assert(home.includes('id="welcome"'));
  assert(!home.includes('<h1'), 'The imported title is rendered only by the layout');
  const reference = site.guidePages.find(page => page.route === '/guide/setup/api-reference');
  assert.equal(reference.title, 'Authored reference title');
  assert.equal(reference.description, 'Authored reference description');
  assert.deepEqual(site.guides[0].sidebar.find(node => node.route === reference.route),
    { kind: 'link', label: 'API navigation', route: reference.route });
  const headingOptions = { remarkPlugins: (await mdxOptions()).mdxOptions.remarkPlugins };
  assert.deepEqual(extractToc(reference.body, headingOptions).map(heading => heading.id), ['install', 'stable-install', 'setext', 'café']);
  assert(indexGuidePage(reference, headingOptions).some(entry => entry.route.endsWith('#stable-install') && entry.text.includes('searchable installation')));
  for (const id of ['reference', 'stable-install', 'setext', 'café']) {
    assert.equal(rendered.get(reference.route).split(`id="${id}"`).length - 1, 1, `Unique rendered ID ${id}`);
  }
  assert(home.includes('src="/products/docs/_zellij/assets/imported/docs/media/logo.svg"'));
  assert(home.includes('href="/_zellij/assets/imported/docs/media/Q3%20Report.pdf"'));
  assert.deepEqual(await readFile(path.join(content, 'assets/imported/docs/media/Q3 Report.pdf')), pdf);
  if (mode === '--export') {
    const output = path.join(root, `${format}-site`);
    execFileSync(zel, ['build', destination, '--out', output], { stdio: 'inherit' });
    const html = await readFile(path.join(output, 'guide.html'), 'utf8');
    for (const reference of [
      'href="/products/docs/guide/setup/api-reference?view=all#install"',
      'href="/products/docs/guide/cafe"',
      `src="/products/docs/_zellij/assets/imported/docs/media/logo.svg?v=${assetHash}"`,
      'href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"',
    ]) assert(html.includes(reference), `Export missing ${reference}`);
    const installed = await readFile(path.join(output, 'guide/setup/api-reference.html'), 'utf8');
    assert(installed.includes('id="install"'));
    for (const id of ['reference', 'stable-install', 'setext', 'café']) assert(installed.includes(`id="${id}"`));
    assert.equal((installed.match(/<h1(?:\s|>)/g) ?? []).length, 1, 'One page title after full export');
    assert(installed.includes('Authored reference title'), 'Authored page title survives export');
    assert(installed.includes('name="description" content="Authored reference description"'), 'Authored description survives export');
    assert(installed.includes('API navigation'), 'Distinct navigation label survives export');
    assert(html.includes('href="/products/docs/guide/setup/api-reference#stable-install"'));
    assert(installed.includes('href="#stable-install"'), 'TOC links to the preserved heading');
    assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/media/logo.svg'), 'utf8'), asset);
    assert.deepEqual(await readFile(path.join(output, '_zellij/assets/imported/docs/media/Q3 Report.pdf')), pdf);
    if (format === 'docusaurus') {
      assert(html.includes(`src="/products/docs/_zellij/assets/imported/static/img/static.svg?v=${assetHash}"`));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/static/img/static.svg'), 'utf8'), asset);
    }
  }
  console.log(`${format}: converted routes, heading fragment, rendered URLs and asset bytes verified.`);
}
