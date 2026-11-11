// Verify actual CLI output against engine routes, MDX rendering and asset bytes.
// With --export, use the matching engine image via ZELLIJ_IMAGE
// to build all imports with matching Next basePath and inspect generated HTML.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
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
// Use the same source and real renderer for the link policy. These checks do
// not fetch public assets; hosted browser acceptance supplies a separate server.
async function checkLinkedAssets(format, source, copiedContent, copiedSite, variant = format) {
  const destination = path.join(root, `${variant}-linked`);
  const content = path.join(destination, 'content');
  const base = 'https://cdn.example.test/Manual%20assets/';
  execFileSync(zel, ['cv', source, '--out', destination, '--from', format, '--layout', 'site', '--assets', 'link', '--asset-base-url', base], {stdio:'inherit'});
  await writeFile(path.join(content, 'site.yaml'), '\nbasePath: /products/docs\n', {flag:'a'});
  const site = await loadSite(new FolderContentSource({contentDir:content}));
  assert.deepEqual([...site.routes].sort(), [...copiedSite.routes].sort());
  assert.deepEqual(await deepCheck(site, content), {errors:[],warnings:[]});
  let html = '';
  for (const page of site.guidePages) {
    const rendered = await compileMDX({source:page.body, options:await mdxOptions(), components:mdxComponents(site)});
    html += renderToStaticMarkup(rendered.content);
  }
  const assets = path.join(copiedContent, 'assets/imported');
  const entries = await readdir(assets, {recursive:true, withFileTypes:true});
  for (const entry of entries.filter(entry => entry.isFile())) {
    // Drop the copy-only docs/static/public namespace, preserving subdirectories.
    const relative = path.relative(assets, path.join(entry.parentPath, entry.name)).split(path.sep).slice(1).map(encodeURIComponent).join('/');
    assert(html.includes(base + relative), `${format}: linked asset missing ${relative}`);
  }
  assert(!html.includes('/_zellij/assets/imported/'), `${format}: link mode emitted a copied asset URL`);
  await assert.rejects(stat(path.join(content, 'assets')), {code:'ENOENT'});
  if (mode === '--export' && format === 'mkdocs') {
    const output = path.join(root, 'linked-site');
    execFileSync(zel, ['build', destination, '--out', output], {stdio:'inherit'});
    const exported = await readFile(path.join(output, 'guide.html'), 'utf8');
    assert(exported.includes(`src="${base}media/logo.svg"`));
    assert(exported.includes(`href="${base}media/Q3%20Report.pdf"`));
    assert(exported.includes('href="/products/docs/guide/setup/api-reference?view=all#install"'));
    await assert.rejects(stat(path.join(output, '_zellij/assets/imported')), {code:'ENOENT'});
  }
  console.log(`${format}: public asset URLs render without local copies or deployment-prefix changes.`);
}

for (const variant of ['mkdocs', 'docusaurus', 'vitepress', 'jekyll', 'jekyll-toml', 'hugo']) {
  const format = variant === 'jekyll-toml' ? 'jekyll' : variant;
  const source = path.join(root, `${variant}-source`);
  const destination = path.join(root, `${variant}-content`);
  const configured = format === 'docusaurus';
  const referenceRoute = configured ? '/guide/Public/API' : '/guide/setup/api-reference';
  const cafeRoute = configured ? '/guide/Café/Release Notes' : '/guide/cafe';
  const referenceUrl = encodeURI(referenceRoute);
  const cafeUrl = encodeURI(cafeRoute);
  const content = configured ? path.join(destination, 'content') : destination;
  async function write(file, body) {
    await mkdir(path.dirname(path.join(source, file)), { recursive: true });
    await writeFile(path.join(source, file), body);
  }
  if (format === 'mkdocs') {
    await write('mkdocs.yml', 'site_name: Converted docs\nsite_url: https://legacy.example.test/manual/\nnav:\n  - index.md\n  - API navigation: 02-Setup/APIReference.md\n  - 03-Café/index.md\n');
  } else if (configured) {
    await write('docusaurus.config.js', "module.exports = {title: 'Converted docs', url: 'https://legacy.example.test', baseUrl: '/manual/', trailingSlash: true, presets: [['classic', {docs: {routeBasePath: 'docs'}}]]};");
    await write('sidebars.js', "module.exports = {docs: [{type: 'category', label: 'Documentation', link: {type: 'generated-index', title: 'Documentation overview', slug: '/Overview', description: '  export default {safe}; <docs> & [links]'}, items: [{type: 'category', label: 'Start here', link: {type: 'doc', id: 'index'}, items: []}, {type: 'autogenerated', dirName: '.'}]}]};");
    await write('static/img/static.svg', asset);
    await write('docs/02-Setup/_category_.yaml', 'label: Setup navigation\nposition: 2.5\nlink:\n  type: generated-index\n  title: Setup overview\n  slug: Sections/Setup\n  description: Build <docs> {safely} & [quickly]\n');
  }
  if (format === 'vitepress') {
    await write('.vitepress/config.ts', "import { defineConfig } from 'vitepress'; export default defineConfig({title:'Converted docs',srcDir:'docs',base:'/manual/',cleanUrls:false,themeConfig:{sidebar:[{link:'/'},{text:'API navigation',link:'/02-Setup/APIReference'},{link:'/03-Café/'}]}});");
    await write('docs/public/static.svg', asset);
  }
  if (format === 'jekyll') {
    if (variant === 'jekyll-toml') {
      await write('_config.toml', `title = 'Converted docs'
source = 'docs'
url = 'https://legacy.example.test'
baseurl = '/manual'
[collections.reference]
output = true
[[defaults]]
scope = {type = 'reference'}
values = {description = 'Inherited collection description'}
`);
    } else {
      await write('_config.yml', 'title: Converted docs\nsource: docs\nurl: https://legacy.example.test\nbaseurl: /manual\ncollections: {reference: {output: true}}\ndefaults: [{scope: {type: reference}, values: {description: Inherited collection description}}]\n');
    }
    await write('docs/_data/navigation.yml', '- name: Welcome\n  link: /\n- name: API navigation\n  link: /02-Setup/APIReference.html\n- name: Café\n  link: /03-Café/\n- name: Extra reference\n  link: /reference/Extra.html\n');
    await write('docs/_reference/Extra.md', '---\ntitle: Extra reference\n---\n# Reference\n\n[Home](../index.md#welcome)\n\n![Chart](chart.svg)\n');
    await write('docs/_reference/chart.svg', asset);
  }
  if (format === 'hugo') {
    await write('hugo.toml', "title='Converted docs'\ncontentDir='docs'\nbaseURL='https://legacy.example.test/manual/'\ndisablePathToLower=true\n");
    await write('static/static.svg', asset);
    await write('docs/02-Setup/_index.md', '---\ntitle: Setup overview\nweight: 1\n---\n# Setup\n\n[Home](../_index.md#welcome)\n');
  }
  const titleId = configured ? '{/* #reference */}' : '{#reference}';
  const installId = configured ? '<!-- #stable-install -->' : '{#stable-install}';
  await write(`docs/02-Setup/APIReference.${configured ? 'mdx' : 'md'}`, `\uFEFF---\r\nid: custom\r\ntitle: Authored reference title\r\ndescription: Authored reference description\r\nsidebar_label: API navigation\r\nslug: /Public/API\r\n...\r\n# Reference ${titleId}\n\n## Install\n\n[Home](../index.md#welcome)\n\n## Install **again** ${installId}\n\nA searchable installation passage.\n\nSetext [home](../index.md#welcome) {#setext}\n------\n\nA searchable setext passage.\n\n## Café {#café}\n\nUnicode passage.\n\n[Self](#stable-install)\n\n[Unicode](#caf%C3%A9)\n`);
  await write('docs/03-Café/index.md', `${configured ? '---\nslug: /Caf%C3%A9/Release%20Notes\n---\n' : format === 'jekyll' ? '---\n---\n' : ''}# Café\n\n[Reference](../02-Setup/APIReference.md)\n`);
  await write('docs/media/logo.svg', asset);
  await write('docs/media/Q3 Report.pdf', pdf);
  await write('docs/index.md', `---\nsidebar_position: 0\n---\n# Welcome {#welcome}\n\n[Install **now**][install]\n\n[Stable install](02-Setup/APIReference.md#stable-install)\n\n[Reference title](02-Setup/APIReference.md#reference)\n\n[Setext](02-Setup/APIReference.md#setext)\n\n[Café](03-Caf%C3%A9/index.md)\n\n![Diagram][logo]\n\n[Download](<media/Q3 Report.pdf>)\n\n[install]: 02-Setup/APIReference.md?view=all#install "Install guide"\n[logo]: media/logo.svg\n\n${format === 'docusaurus' ? '![Static](/img/static.svg)\n' : ''}\n\n\`\`\`\`md\n\`\`\`\n# Literal {#literal}\n!!! note\n\`\`\`\`\n`);
  await writeFile(path.join(source, `docs/02-Setup/APIReference.${configured ? 'mdx' : 'md'}`), `\n[Website home](${['vitepress', 'jekyll'].includes(format) ? '../index.html' : '../../'}?from=relative#welcome)\n`, { flag: 'a' });
  const sourcePageUrl = configured ? '/manual/docs/Public/API' : `/manual/02-Setup/APIReference${['vitepress', 'jekyll'].includes(format) ? '.html' : '/'}`;
  const sourceAssetUrl = configured ? '/manual/img/static.svg' : '/manual/media/logo.svg';
  // VitePress has no site-origin setting. Absolute URLs stay external.
  // The other importers can identify this as their configured source host.
  const absoluteReference = format === 'vitepress' ? `https://legacy.example.test${sourcePageUrl}` : referenceUrl;
  await writeFile(path.join(source, 'docs/index.md'), `\n[Website URL](${sourcePageUrl}?source=website#stable-install)\n\n[Absolute URL](https://legacy.example.test${sourcePageUrl}#reference)\n\n![Website asset](https://legacy.example.test${sourceAssetUrl})\n`, { flag: 'a' });
  if (format === 'vitepress') await writeFile(path.join(source, 'docs/index.md'), '\n![Public asset](/static.svg)\n', { flag: 'a' });
  if (format === 'hugo') {
    const homePath = path.join(source, 'docs/index.md');
    const home = await readFile(homePath, 'utf8');
    await write('docs/_index.md', home + '\n![Static asset](/manual/static.svg)\n');
    await rm(homePath);
    const ref = path.join(source, 'docs/02-Setup/APIReference.md');
    await writeFile(ref, (await readFile(ref, 'utf8')).replace('slug: /Public/API', 'linkTitle: API navigation').replace('...\r\n', '---\r\n').replaceAll('../index.md', '../_index.md'));
    const cafe = path.join(source, 'docs/03-Café/index.md');
    await writeFile(cafe, '---\nweight: 2\n---\n' + await readFile(cafe, 'utf8') + '\n![Bundle](chart.svg)\n');
    await write('docs/03-Café/chart.svg', asset);
  }
  execFileSync(zel, [configured && mode === '--export' ? 'migrate' : 'convert', source, '--out', destination, '--from', format, '--layout', configured ? 'site' : 'content', '--assets', 'copy', ...(!configured && mode === '--export' ? ['--validate'] : [])], { stdio: 'inherit' });
  // A non-empty deployment prefix exercises the ordinary Markdown image path.
  await writeFile(path.join(content, 'site.yaml'), '\nbasePath: /products/docs\n', { flag: 'a' });
  execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'check', content], { stdio: 'inherit' });
  const site = await loadSite(new FolderContentSource({ contentDir: content }));
  assert.deepEqual(await deepCheck(site, content), { errors: [], warnings: [] });
  assert.deepEqual([...site.routes].sort(), ['/', '/guide', cafeRoute, ...(configured ? ['/guide/Overview', '/guide/Sections/Setup'] : format === 'jekyll' ? ['/guide/reference/extra'] : format === 'hugo' ? ['/guide/setup'] : []), referenceRoute].sort());
  const rendered = new Map();
  for (const page of site.guidePages) {
    const result = await compileMDX({ source: page.body, options: await mdxOptions(), components: mdxComponents(site) });
    rendered.set(page.route, renderToStaticMarkup(result.content));
  }
  const home = rendered.get('/guide');
  assert(home.includes(`href="${referenceUrl}?view=all#install"`));
  assert(home.includes(`href="${referenceUrl}?source=website#stable-install"`));
  assert(home.includes(`href="${absoluteReference}#reference"`));
  if (format !== 'vitepress') assert(!home.includes('https://legacy.example.test'));
  assert(home.includes(`href="${cafeUrl}"`));
  assert(rendered.get(referenceRoute).includes('id="install"'));
  assert(rendered.get(referenceRoute).includes('href="/guide?from=relative#welcome"'));
  assert(home.includes('id="welcome"'));
  assert(!home.includes('<h1'), 'The imported title is rendered only by the layout');
  const reference = site.guidePages.find(page => page.route === referenceRoute);
  assert.equal(reference.title, 'Authored reference title');
  assert.equal(reference.description, 'Authored reference description');
  const sidebar = site.guides[0].sidebar;
  if (format === 'docusaurus') {
    assert.deepEqual(sidebar, [{ kind: 'group', label: 'Documentation', children: [
      { kind: 'link', label: 'Documentation overview', route: '/guide/Overview' },
      { kind: 'link', label: 'Start here', route: '/guide' },
      { kind: 'link', label: 'Welcome', route: '/guide' },
      { kind: 'group', label: 'Setup navigation', children: [
        { kind: 'link', label: 'Setup overview', route: '/guide/Sections/Setup' },
        { kind: 'link', label: 'API navigation', route: reference.route },
      ] },
      { kind: 'link', label: 'Café', route: cafeRoute },
    ] }]);
    const overview = site.guidePages.find(page => page.route === '/guide/Sections/Setup');
    assert.equal(overview.title, 'Setup overview');
    assert.equal(overview.description, 'Build <docs> {safely} & [quickly]');
    assert(rendered.get(overview.route).includes('Build &lt;docs&gt; {safely} &amp; [quickly]'));
    assert(rendered.get(overview.route).includes(`href="${referenceUrl}"`));
    assert(rendered.get('/guide/Overview').includes('href="/guide/Sections/Setup"'));
    assert(indexGuidePage(overview).some(entry => entry.text.includes('Build <docs> {safely}')));
  } else if (format === 'hugo') {
    const setup = sidebar.find(node => node.kind === 'group' && node.label === 'Setup overview');
    assert.deepEqual(setup.children, [{kind:'link',label:'Setup overview',route:'/guide/setup'},{kind:'link',label:'API navigation',route:reference.route}]);
    assert(rendered.get('/guide/setup').includes('href="/guide#welcome"'));
    assert(rendered.get(cafeRoute).includes('src="/products/docs/_zellij/assets/imported/docs/03-Caf%C3%A9/chart.svg"'));
    assert(home.includes('src="/products/docs/_zellij/assets/imported/static/static.svg"'));
  } else {
    assert.deepEqual(sidebar.find(node => node.route === reference.route),
      { kind: 'link', label: 'API navigation', route: reference.route });
  }
  const headingOptions = { remarkPlugins: (await mdxOptions()).mdxOptions.remarkPlugins };
  assert.deepEqual(extractToc(reference.body, headingOptions).map(heading => heading.id), ['install', 'stable-install', 'setext', 'café']);
  assert(indexGuidePage(reference, headingOptions).some(entry => entry.route === `${referenceRoute}#stable-install` && entry.text.includes('searchable installation')));
  for (const id of ['reference', 'stable-install', 'setext', 'café']) {
    assert.equal(rendered.get(reference.route).split(`id="${id}"`).length - 1, 1, `Unique rendered ID ${id}`);
  }
  assert(home.includes('src="/products/docs/_zellij/assets/imported/docs/media/logo.svg"'));
  assert(home.includes('href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"'));
  assert.deepEqual(await readFile(path.join(content, 'assets/imported/docs/media/Q3 Report.pdf')), pdf);
  if (format === 'jekyll') {
    assert.equal(site.guidePages.find(page => page.route === '/guide/reference/extra').description, 'Inherited collection description');
    const extra = rendered.get('/guide/reference/extra');
    assert(extra.includes('href="/guide#welcome"'));
    assert(extra.includes('src="/products/docs/_zellij/assets/imported/docs/_reference/chart.svg"'));
    assert.equal(await readFile(path.join(content, 'assets/imported/docs/_reference/chart.svg'), 'utf8'), asset);
  }
  if (format === 'vitepress') {
    assert(home.includes('src="/products/docs/_zellij/assets/imported/public/static.svg"'));
    assert.equal(await readFile(path.join(content, 'assets/imported/public/static.svg'), 'utf8'), asset);
  }
  if (mode === '--export') {
    const output = path.join(root, `${variant}-site`);
    execFileSync(zel, ['build', destination, '--out', output], { stdio: 'inherit' });
    const html = await readFile(path.join(output, 'guide.html'), 'utf8');
    for (const reference of [
      `href="/products/docs${referenceUrl}?view=all#install"`,
      `href="/products/docs${referenceUrl}?source=website#stable-install"`,
      `href="${format === 'vitepress' ? absoluteReference : `/products/docs${referenceUrl}`}#reference"`,
      `href="/products/docs${cafeUrl}"`,
      `src="/products/docs/_zellij/assets/imported/docs/media/logo.svg?v=${assetHash}"`,
      'href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"',
    ]) assert(html.includes(reference), `Export missing ${reference}`);
    const installed = await readFile(path.join(output, `${referenceRoute.slice(1)}.html`), 'utf8');
    assert(installed.includes('id="install"'));
    assert(installed.includes('href="/products/docs/guide?from=relative#welcome"'), 'Relative website links use source URL directories, not source file parents');
    const cafe = await readFile(path.join(output, `${cafeRoute.slice(1)}.html`), 'utf8');
    assert(cafe.includes('<h1>Café</h1>'), 'The exported Unicode route renders its page instead of a 404 shell');
    assert(cafe.includes(`href="/products/docs${referenceUrl}"`), 'Encoded Unicode route exports and resolves its document link');
    assert([...cafe.matchAll(/<a\b[^>]*aria-current="page"[^>]*>/g)].some(([tag]) => tag.includes('zellij-guide-sidebar-link') && tag.includes('data-active="true"')), 'The exported page marks its sidebar link active');
    for (const id of ['reference', 'stable-install', 'setext', 'café']) assert(installed.includes(`id="${id}"`));
    assert.equal((installed.match(/<h1(?:\s|>)/g) ?? []).length, 1, 'One page title after full export');
    assert(installed.includes('Authored reference title'), 'Authored page title survives export');
    assert(installed.includes('name="description" content="Authored reference description"'), 'Authored description survives export');
    assert(installed.includes('API navigation'), 'Distinct navigation label survives export');
    if (format === 'docusaurus') {
      assert(installed.includes('Setup navigation'), 'Generated category label survives export');
      assert(installed.includes('Start here'), 'Manual category document link survives export');
      const overview = await readFile(path.join(output, 'guide/Sections/Setup.html'), 'utf8');
      assert(overview.includes(`href="/products/docs${referenceUrl}"`));
      assert(overview.includes('Build &lt;docs&gt; {safely} &amp; [quickly]'));
      const landing = await readFile(path.join(output, 'guide/Overview.html'), 'utf8');
      assert(landing.includes('href="/products/docs/guide/Sections/Setup"'));
      assert(landing.includes('export default {safe}; &lt;docs&gt; &amp; [links]'));
    }
    assert(html.includes(`href="/products/docs${referenceUrl}#stable-install"`));
    assert(installed.includes('href="#stable-install"'), 'TOC links to the preserved heading');
    assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/media/logo.svg'), 'utf8'), asset);
    assert.deepEqual(await readFile(path.join(output, '_zellij/assets/imported/docs/media/Q3 Report.pdf')), pdf);
    if (format === 'hugo') {
      const section = await readFile(path.join(output, 'guide/setup.html'), 'utf8');
      assert(section.includes('<h1>Setup overview</h1>'));
      assert(section.includes('href="/products/docs/guide#welcome"'));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/03-Café/chart.svg'), 'utf8'), asset);
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/static/static.svg'), 'utf8'), asset);
    }
    if (format === 'jekyll') {
      const extra = await readFile(path.join(output, 'guide/reference/extra.html'), 'utf8');
      assert(extra.includes('<h1>Extra reference</h1>'));
      assert(extra.includes('name="description" content="Inherited collection description"'));
      assert(extra.includes(`src="/products/docs/_zellij/assets/imported/docs/_reference/chart.svg?v=${assetHash}"`));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/_reference/chart.svg'), 'utf8'), asset);
    }
    if (format === 'vitepress') {
      assert(html.includes(`src="/products/docs/_zellij/assets/imported/public/static.svg?v=${assetHash}"`));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/public/static.svg'), 'utf8'), asset);
    }
    if (format === 'docusaurus') {
      assert(html.includes(`src="/products/docs/_zellij/assets/imported/static/img/static.svg?v=${assetHash}"`));
      assert.equal(await readFile(path.join(output, '_zellij/assets/imported/static/img/static.svg'), 'utf8'), asset);
    }
  }
  await checkLinkedAssets(format, source, content, site, variant);
  console.log(`${variant}: converted routes, heading fragment, rendered URLs and asset bytes verified.`);
}

// Derived Jekyll collection ordering must survive the engine's sidebar loader
// and real hosted export, not just serialization into _sidebar.yaml.
{
  const source = path.join(root, 'jekyll-sorting-source');
  const destination = path.join(root, 'jekyll-sorting-content');
  await mkdir(path.join(source, '_lessons'), {recursive:true});
  await writeFile(path.join(source, '_config.toml'), `title = 'Ordered lessons'
[collections.lessons]
output = true
sort_by = 'weight'
[[defaults]]
scope = {path = '_lessons/b-inherited.md', type = 'lessons'}
values = {weight = 0.5}
`);
  for (const [name, metadata] of [
    ['a-missing', ''], ['b-inherited', ''], ['c-tie', 'weight: 0.5'],
    ['d-first', 'weight: -2'], ['e-null', 'weight: null'],
  ]) {
    await writeFile(path.join(source, '_lessons', `${name}.md`), `---\ntitle: ${name}\n${metadata}\n---\n# Lesson\n\nLesson body for ${name}.\n`);
  }
  execFileSync(zel, ['cv', source, '--out', destination, '--layout', 'site'], {stdio:'inherit'});
  const content = path.join(destination, 'content');
  const site = await loadSite(new FolderContentSource({contentDir:content}));
  const expected = ['d-first', 'b-inherited', 'c-tie', 'a-missing', 'e-null'].map(name => `/guide/lessons/${name}`);
  const group = site.guides[0].sidebar.find(node => node.kind === 'group' && node.label === 'lessons');
  assert.deepEqual(group.children.map(node => node.route), expected);
  assert.deepEqual(await deepCheck(site, content), {errors:[],warnings:[]});
  for (const page of site.guidePages) {
    const rendered = await compileMDX({source:page.body, options:await mdxOptions(), components:mdxComponents(site)});
    assert(renderToStaticMarkup(rendered.content).includes('Lesson'));
  }
  if (mode === '--export') {
    const output = path.join(root, 'jekyll-sorting-site');
    execFileSync(zel, ['build', destination, '--out', output], {stdio:'inherit'});
    const html = await readFile(path.join(output, 'guide/lessons/d-first.html'), 'utf8');
    const links = [...html.matchAll(/<a\b[^>]*>/g)]
      .map(([tag]) => tag.includes('zellij-guide-sidebar-link') ? tag.match(/href="([^"]+)"/)?.[1] : null)
      .filter(href => expected.includes(href));
    assert.deepEqual(links.slice(0, expected.length), expected);
  }
  console.log('jekyll-sorting: inherited ranks, ties and missing values retain sidebar order.');
}

// Sphinx starts from reStructuredText, not from the Markdown fixtures above.
{
  const source = path.join(root, 'sphinx-source');
  const content = path.join(root, 'sphinx-content');
  async function write(file, body) {
    await mkdir(path.dirname(path.join(source, file)), { recursive: true });
    await writeFile(path.join(source, file), body);
  }
  await write('conf.py', "project='Sphinx manual'\nversion='2.4'\nrelease='2.4.1'\nhtml_baseurl='https://legacy.example.test/manual/'\nextensions=[]\nexclude_patterns=['glob/draft*', '**/private*']\n");
  await write('index.rst', `.. _welcome:

Welcome
========

.. toctree::
   :caption: Documentation

   API navigation <02-Setup/APIReference>
   03-Café/index
   tables
   lists
   substitutions
   literalinclude
   includes
   patterns

Read :doc:\`02-Setup/APIReference\` and :ref:\`stable-install\`.

\`Website <https://legacy.example.test/manual/02-Setup/APIReference.html?q=1#stable-install>\`_

.. image:: /media/logo.svg
   :alt: Logo

:download:\`Report <media/Q3 Report.pdf>\`

:download:\`Literal percent <media/Report%20.pdf>\`

.. image:: media/chart%20.svg

<script>literal_source()</script> and {literal_expression()}

:strong:\`<strong>literal role</strong>\` and \`\`a\`{literal_code}\`\`
`);
  await write('02-Setup/APIReference.rst', `.. _reference:

Authored reference title
========================

.. _stable-install:

Installation
------------

Searchable installation with **strong** and \`\`inline_{code}\`\`.

.. warning:: Keep your source.

   Return :ref:\`Home <welcome>\`.

\`Website home <../index.html?from=relative#welcome>\`_

.. code-block:: text

   .. include:: private-file
   :ref:\`literal\`
`);
  await write('03-Café/index.rst', 'Café\n====\n\n:doc:`/02-Setup/APIReference`\n');
  await write('tables.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-tables.rst'), 'utf8'));
  const externalCsv = await readFile(path.join(repo, 'scripts/fixtures/sphinx-external-table.csv'), 'utf8');
  await write('data/values%20.csv', '\uFEFF' + externalCsv.replace(/\r\n/g, '\n').replace(/\n/g, '\r\n'));
  await write('lists.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-lists.rst'), 'utf8'));
  await write('substitutions.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-substitutions.rst'), 'utf8'));
  await write('literalinclude.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-literalinclude.rst'), 'utf8'));
  const literalCode = await readFile(path.join(repo, 'scripts/fixtures/sphinx-literal-code.txt'), 'utf8');
  await write('snippets/Sample%20.py', literalCode);
  await write('includes.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-includes.rst'), 'utf8'));
  await write('fragments/section.rst', await readFile(path.join(repo, 'scripts/fixtures/sphinx-include-section.rst'), 'utf8'));
  await write('body.inc', await readFile(path.join(repo, 'scripts/fixtures/sphinx-include-body.inc'), 'utf8'));
  await write('fragments/values.csv', 'Included CSV origin,correct\n');
  await write('values.csv', 'WRONG origin,wrong\n');
  await write('patterns.rst', 'Pattern navigation\n==================\n\n.. toctree::\n   :glob:\n   :reversed:\n   :caption: Matched pages\n\n   glob/page?\n   glob/**\n');
  await write('glob/page1.rst', 'First matched page\n==================\n\nFirst wildcard searchable content.\n');
  await write('glob/page2.rst', 'Second matched page\n===================\n\nSecond wildcard searchable content.\n');
  await write('glob/draft.rst', '.. unsupported:: PRIVATE_EXCLUSION_SENTINEL\n');
  await write('glob/private/deep/page.rst', '.. unsupported:: PRIVATE_EXCLUSION_SENTINEL\n');

  await write('details.rst', '.. _install:\n\nInstallation\n============\n');
  await write('notes.txt', 'Table download bytes.\n');
  await write('media/logo.svg', asset);
  await write('media/Q3 Report.pdf', pdf);
  await write('media/Report%20.pdf', pdf);
  await write('media/Report .pdf', 'wrong source');
  await write('media/chart%20.svg', asset);
  await write('media/chart .svg', '<svg>wrong source</svg>');
  execFileSync(zel, ['convert', source, '--out', content, '--from', 'sphinx', ...(mode === '--export' ? ['--validate'] : [])], {stdio:'inherit'});
  await writeFile(path.join(content, 'site.yaml'), '\nbasePath: /products/docs\n', {flag:'a'});
  const site = await loadSite(new FolderContentSource({contentDir:content}));
  assert.deepEqual(await deepCheck(site, content), {errors:[],warnings:[]});
  assert.deepEqual([...site.routes].sort(), ['/', '/guide', '/guide/cafe', '/guide/details', '/guide/glob/page1', '/guide/glob/page2', '/guide/includes', '/guide/lists', '/guide/literalinclude', '/guide/patterns', '/guide/setup/api-reference', '/guide/substitutions', '/guide/tables']);
  const rendered = new Map();
  for (const page of site.guidePages) {
    const result = await compileMDX({source:page.body,options:await mdxOptions(),components:mdxComponents(site)});
    rendered.set(page.route, renderToStaticMarkup(result.content));
  }
  const home = rendered.get('/guide');
  function assertPatterns(html, prefix) {
    if (prefix) html = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/)[1];
    const links = [...html.matchAll(/<a\b[^>]*href="([^"]+)"/g)].map(m => m[1]);
    assert.deepEqual(links, [`${prefix}/guide/glob/page2`, `${prefix}/guide/glob/page1`]);
  }
  assertPatterns(rendered.get('/guide/patterns'), '');
  await assert.rejects(readFile(path.join(content, '01-guide/glob/draft.mdx')), {code:'ENOENT'});
  await assert.rejects(readFile(path.join(content, '01-guide/glob/private/deep/page.mdx')), {code:'ENOENT'});
  for (const text of ['href="/guide/setup/api-reference"', 'href="/guide/setup/api-reference#stable-install"', 'href="/guide/setup/api-reference?q=1#stable-install"', 'src="/products/docs/_zellij/assets/imported/docs/media/logo.svg"', 'href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"', 'id="welcome"', '&lt;script&gt;literal_source()&lt;/script&gt;', '{literal_expression()}']) assert(home.includes(text), `Sphinx home missing ${text}: ${home}`);
  assert(!home.includes('<script>') && !home.includes('<h1'));
  assert(home.includes('<strong>&lt;strong&gt;literal role&lt;/strong&gt;</strong>'));
  assert(home.includes('a`{literal_code}') && !home.includes('a`\\{literal_code\\}'));
  assert(home.includes('href="/products/docs/_zellij/assets/imported/docs/media/Report%2520.pdf"'));
  assert(home.includes('src="/products/docs/_zellij/assets/imported/docs/media/chart%2520.svg"'));
  const page = site.guidePages.find(p => p.route === '/guide/setup/api-reference');
  const html = rendered.get(page.route);
  for (const text of ['id="stable-install"', 'href="/guide#welcome"', 'href="/guide?from=relative#welcome"', '.. include:: private-file']) assert(html.includes(text), `Sphinx reference missing ${text}`);
  const headingOptions = {remarkPlugins:(await mdxOptions()).mdxOptions.remarkPlugins};
  assert.deepEqual(extractToc(page.body,headingOptions).map(h=>h.id), ['stable-install']);
  assert(indexGuidePage(page,headingOptions).some(e=>e.route===`${page.route}#stable-install` && e.text.includes('Searchable installation')));
  function assertTables(html, prefix) {
    assert.equal((html.match(/<table(?:\s|>)/g) ?? []).length, 9);
    assert.equal((html.match(/<thead(?:\s|>)/g) ?? []).length, 5, 'Headerless tables must not invent headers');
    assert.equal((html.match(/<th(?:\s|>)/g) ?? []).length, 28, 'Column and row headings survive');
    assert(html.includes('<caption>') && html.includes('<strong>Feature</strong> matrix'));
    for (const text of ['Café 漢字 é', 'a|{b}', '&lt;script&gt;literal&lt;/script&gt;',
      'First paragraph.', 'Second paragraph.', 'First item', 'Second item',
      'First line extends beyond the border.', 'Continuation with <strong>strong</strong> text.',
      'Empty first cell.', 'No invented header.',
      `href="${prefix}/guide/details"`, `href="${prefix}/guide/details#install"`,
      'href="/products/docs/_zellij/assets/imported/docs/notes.txt"',
      'alt="Table logo"', 'src="/products/docs/_zellij/assets/imported/docs/media/logo.svg',
    ]) assert(html.includes(text), `Sphinx table missing ${text}`);
    assert.equal((html.match(/<th scope="row"(?:\s|>)/g) ?? []).length, 9);
    for (const text of ['<strong>List</strong> matrix', 'List first paragraph.', 'List second paragraph',
      'Empty list row heading.', 'Quoted &quot;word&quot;, comma and <strong>bold</strong>.',
      'Second CSV paragraph.', 'Padded CSV row', '漢字│quoted &#x27;word&#x27;',
      `href="${prefix}/guide/details"`, 'href="/products/docs/_zellij/assets/imported/docs/notes.txt"',
      'alt="List logo"', '.. csv-table:: &lt;script&gt;{code}|literal&lt;/script&gt;',
      ':header: literal field in code', 'Local CSV matrix',
      'External CSV searchable <strong>value</strong>', 'alt="CSV file logo"',
    ]) assert(html.includes(text), `Sphinx directive table missing ${text}`);
    assert(!html.includes('<script>literal</script>'));
  }
  assertTables(rendered.get('/guide/tables'), '');
  assert(indexGuidePage(site.guidePages.find(p => p.route === '/guide/tables'), headingOptions).some(e => e.text.includes('External CSV searchable')));
  await assert.rejects(readFile(path.join(content, 'assets/imported/docs/data/values%20.csv')), {code:'ENOENT'});
  function assertLists(html, prefix) {
    if (prefix) {
      // Hosted exports also contain navigation/sidebar lists. Count only the
      // authored article, matching the MDX-only render used by the local check.
      const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/);
      assert(article, 'Hosted list fixture must contain its guide article');
      html = article[1];
    }
    assert.equal((html.match(/<ul(?:\s|>)/g) ?? []).length, 6);
    assert.equal((html.match(/<ol(?:\s|>)/g) ?? []).length, 9);
    assert.equal((html.match(/<table(?:\s|>)/g) ?? []).length, 2);
    for (const start of [9, 7, 3, 4, 2]) assert(html.includes(`start="${start}"`));
    for (const [type, start] of [['a', 8], ['A', 8], ['i', 4], ['I', 4]]) {
      assert(html.includes(`<ol start="${start}" type="${type}">`), `Missing ${type} sequence`);
    }
    for (const text of ['Prepare <strong>workspace</strong>.', 'Nested searchable paragraph.',
      'Unicode parent Café 漢字.', 'Unicode leaf.', 'Table child <strong>strong</strong>.',
      ':header: literal option', '- literal bullet', '#. literal number', '.. include:: /not/executed',
      '- literal block bullet', '9. literal block number', 'Nested table',
      'Roman <strong>four</strong>', 'Upper alphabetic automatic eleven.', 'Lower alphabetic nine.',
      `href="${prefix}/guide/details"`, `href="${prefix}/guide/details#install"`,
      'href="/products/docs/_zellij/assets/imported/docs/notes.txt"', 'alt="Nested logo"',
    ]) assert(html.includes(text), `Nested Sphinx list missing ${text}`);
  }
  assertLists(rendered.get('/guide/lists'), '');
  function assertSubstitutions(html, prefix) {
    for (const text of [
      '<strong>Zellij</strong> is the <strong>Zellij</strong> documentation engine.',
      '<em>Read</em> the guide</a>', 'The release is 2.4.1 (2.4).',
      `href="${prefix}/guide/details#install"`,
      'href="/products/docs/_zellij/assets/imported/docs/notes.txt"',
      'alt="Substitution logo"', 'class="zellij-inline-image"',
      '|undefined-in-code|',
      '<strong>Zellij</strong> features',
    ]) assert(html.includes(text), `Sphinx substitution missing ${text}`);
    assert([...html.matchAll(/<pre\b[^>]*>([\s\S]*?)<\/pre>/g)].some(match =>
      match[1].replace(/<[^>]*>/g, '').includes('.. |ignored| replace:: not a definition')),
    'Highlighted substitution definition must remain literal code');
  }
  assertSubstitutions(rendered.get('/guide/substitutions'), '');
  function assertLiteralIncludes(html) {
    const codes = [...html.matchAll(/<pre\b[^>]*>([\s\S]*?)<\/pre>/g)].map(match => match[1]
      .replace(/<[^>]*>/g, '').replace(/&quot;/g, '"').replace(/&#x27;|&#39;/g, "'")
      .replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&'));
    assert.equal(codes.length, 6);
    assert.equal(codes[0].trimEnd(), literalCode.replace(/\r\n?/g, '\n').trimEnd());
    const alpha = 'literal_alpha = "Café 漢字"';
    const beta = 'literal_beta = "{not_executed()} <script>literal()</script>"';
    assert.equal(codes[1].trimEnd(), `${beta}\n${alpha}\n${beta}`);
    assert.equal(codes[2].trimEnd(), alpha);
    assert.equal(codes[3].trimEnd(), beta);
    assert.equal(codes[4].trimEnd(), `${alpha}\n${beta}`);
    assert.equal(codes[5].trimEnd(), '```\nfenceonlytoken');
    assert(!html.includes('<script>literal()</script>'));
    assert(!html.includes('href="../missing.md"'));
  }
  assertLiteralIncludes(rendered.get('/guide/literalinclude'));
  function assertParsedIncludes(html, prefix = '') {
    for (const text of ['id="shared-section"', 'id="after-inclusion"', 'Included fragment searchable phrase',
      '<strong>Shared product</strong>', 'Included CSV origin',
      `href="${prefix}/guide/details"`, `href="${prefix}/guide/includes#shared-section"`,
      '/products/docs/_zellij/assets/imported/docs/media/logo.svg']) assert(html.includes(text), `Parsed include missing ${text}`);
    assert(!html.includes('WRONG origin'));
    assert(html.replace(/<[^>]*>/g, '').includes('.. include:: missing-example.inc'));
  }
  assertParsedIncludes(rendered.get('/guide/includes'));
  const includePage = site.guidePages.find(p => p.route === '/guide/includes');
  assert.deepEqual(extractToc(includePage.body, headingOptions).map(h => h.id), ['shared-section', 'after-inclusion']);
  assert(indexGuidePage(includePage, headingOptions).some(entry => entry.route.endsWith('#shared-section') && entry.text.includes('Included fragment searchable phrase')));
  await assert.rejects(readFile(path.join(content, 'assets/imported/docs/body.inc')), {code:'ENOENT'});
  await assert.rejects(readFile(path.join(content, '01-guide/fragments/section.mdx')), {code:'ENOENT'});

  const literalPage = site.guidePages.find(p => p.route === '/guide/literalinclude');
  const literalIndex = indexGuidePage(literalPage, headingOptions);
  function assertLiteralSearch(entries) {
    const passages = entries.filter(entry => entry.route.startsWith('/guide/literalinclude'));
    assert(passages.some(entry => entry.text.includes('explanatory prose remains searchable')));
    assert(passages.some(entry => entry.route.endsWith('#selected-excerpt') && entry.text.includes('Search keeps the explanation')));
    assert(!passages.some(entry => /fenceonlytoken|literal.?alpha|literal.?beta|undefined-substitution/.test(entry.text)));
  }
  assertLiteralSearch(literalIndex);
  assert.deepEqual(extractToc(literalPage.body, headingOptions).map(h => h.id), ['selected-excerpt']);
  await assert.rejects(readFile(path.join(content, 'assets/imported/docs/snippets/Sample%20.py')), {code:'ENOENT'});
  const substitutionPage = site.guidePages.find(p => p.route === '/guide/substitutions');
  assert(indexGuidePage(substitutionPage, headingOptions).some(entry => entry.text.includes('documentation engine')));

  const listPage = site.guidePages.find(p => p.route === '/guide/lists');
  for (const phrase of ['Nested searchable paragraph.', 'Upper alphabetic automatic eleven.', 'Roman automatic seven.']) {
    assert(indexGuidePage(listPage, headingOptions).some(entry => entry.text.includes(phrase)), `List text searchable: ${phrase}`);
  }
  const tablePage = site.guidePages.find(p => p.route === '/guide/tables');
  const tableIndex = indexGuidePage(tablePage, headingOptions);
  for (const phrase of ['Empty first cell.', 'List second paragraph', 'Second CSV paragraph.']) {
    assert(tableIndex.some(entry => entry.text.includes(phrase)), `Table cell text is searchable: ${phrase}`);
  }
  assert.deepEqual(site.guides[0].sidebar, [
    {kind:'link',label:'Welcome',route:'/guide'},
    {kind:'group',label:'Documentation',children:[
      {kind:'link',label:'API navigation',route:'/guide/setup/api-reference'},
      {kind:'link',label:'Café',route:'/guide/cafe'},
      {kind:'group',label:'Table reference',children:[
        {kind:'link',label:'Table reference',route:'/guide/tables'},
        {kind:'link',label:'Installation',route:'/guide/details'},
      ]},
      {kind:'link',label:'Nested instructions',route:'/guide/lists'},
      {kind:'link',label:'Reusable content',route:'/guide/substitutions'},
      {kind:'link',label:'Referenced examples',route:'/guide/literalinclude'},
      {kind:'link',label:'Included manual',route:'/guide/includes'},
      {kind:'group',label:'Pattern navigation',children:[
        {kind:'link',label:'Pattern navigation',route:'/guide/patterns'},
        {kind:'group',label:'Matched pages',children:[
          {kind:'link',label:'Second matched page',route:'/guide/glob/page2'},
          {kind:'link',label:'First matched page',route:'/guide/glob/page1'},
        ]},
      ]},
    ]},
  ]);
  assert.equal(await readFile(path.join(content,'assets/imported/docs/media/logo.svg'),'utf8'),asset);
  assert.deepEqual(await readFile(path.join(content,'assets/imported/docs/media/Q3 Report.pdf')),pdf);
  assert.deepEqual(await readFile(path.join(content,'assets/imported/docs/media/Report%20.pdf')),pdf);
  assert.equal(await readFile(path.join(content,'assets/imported/docs/media/chart%20.svg'),'utf8'),asset);
  if (mode === '--export') {
    const output = path.join(root,'sphinx-site');
    execFileSync(zel,['build',content,'--out',output],{stdio:'inherit'});
    const home = await readFile(path.join(output,'guide.html'),'utf8');
    const reference = await readFile(path.join(output,'guide/setup/api-reference.html'),'utf8');
    const cafe = await readFile(path.join(output,'guide/cafe.html'),'utf8');
    assertPatterns(await readFile(path.join(output, 'guide/patterns.html'), 'utf8'), '/products/docs');
    assertTables(await readFile(path.join(output, 'guide/tables.html'), 'utf8'), '/products/docs');
    assertLists(await readFile(path.join(output, 'guide/lists.html'), 'utf8'), '/products/docs');
    assertSubstitutions(await readFile(path.join(output, 'guide/substitutions.html'), 'utf8'), '/products/docs');
    assertLiteralIncludes(await readFile(path.join(output, 'guide/literalinclude.html'), 'utf8'));
    assertParsedIncludes(await readFile(path.join(output, 'guide/includes.html'), 'utf8'), '/products/docs');
    assertLiteralSearch(JSON.parse(await readFile(path.join(output, '_zellij/search-index.json'), 'utf8')));
    await assert.rejects(readFile(path.join(output, '_zellij/assets/imported/docs/snippets/Sample%20.py')), {code:'ENOENT'});
    assert.equal(await readFile(path.join(output, '_zellij/assets/imported/docs/notes.txt'), 'utf8'), 'Table download bytes.\n');
    for (const text of ['href="/products/docs/guide/setup/api-reference#stable-install"','href="/products/docs/guide/setup/api-reference?q=1#stable-install"',`src="/products/docs/_zellij/assets/imported/docs/media/logo.svg?v=${assetHash}"`,'href="/products/docs/_zellij/assets/imported/docs/media/Q3%20Report.pdf"','&lt;script&gt;literal_source()&lt;/script&gt;']) assert(home.includes(text),`Sphinx export missing ${text}`);
    for (const text of ['id="stable-install"','href="#stable-install"','href="/products/docs/guide#welcome"','href="/products/docs/guide?from=relative#welcome"','API navigation']) assert(reference.includes(text), `Sphinx exported reference missing ${text}`);
    assert.equal((reference.match(/<h1(?:\s|>)/g)??[]).length,1);
    assert(cafe.includes('<h1>Café</h1>') && cafe.includes('href="/products/docs/guide/setup/api-reference"'));
    assert.equal(await readFile(path.join(output,'_zellij/assets/imported/docs/media/logo.svg'),'utf8'),asset);
    assert.deepEqual(await readFile(path.join(output,'_zellij/assets/imported/docs/media/Q3 Report.pdf')),pdf);
    assert(home.includes('href="/products/docs/_zellij/assets/imported/docs/media/Report%2520.pdf"'));
    assert(home.includes(`src="/products/docs/_zellij/assets/imported/docs/media/chart%2520.svg?v=${assetHash}"`));
    assert.deepEqual(await readFile(path.join(output,'_zellij/assets/imported/docs/media/Report%20.pdf')),pdf);
    assert.equal(await readFile(path.join(output,'_zellij/assets/imported/docs/media/chart%20.svg'),'utf8'),asset);
    const { checkImportedTables } = await import('./check-imported-tables.mjs');
    await checkImportedTables(output);
  }
  await checkLinkedAssets('sphinx', source, content, site);
  console.log('sphinx: reStructuredText navigation, inline/local CSV tables, nested lists, substitutions, literalinclude excerpts, parsed includes, references, literal code/HTML, search, headings and assets verified.');
}
