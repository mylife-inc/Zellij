import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { portableDocumentSchema } from './portable-document';
import { FolderContentSource, loadSite } from './index';
import { PortableBody } from '../render/portable-body';
import { deepCheck } from '../cli/deep-check';
import { indexGuidePage } from '../search/build-index';
import { flattenSidebar, findNeighbours } from '../guide/navigation';

const text = (value: string) => ({ type: 'text', value });
const el = (tagName: string, properties: object = {}, children: object[] = []) => ({ type: 'element', tagName, properties, children });
const document = {
  version: 1, title: 'Docs', description: 'Description', route: '/', text: 'A searchable document',
  headings: [{ id: 'custom', text: 'Custom heading', depth: 2 }],
  body: { type: 'root', children: [
    el('h1', {id:'title'}, [text('Docs')]),
    el('h2', {id:'custom'}, [text('Custom heading')]),
    el('p', {}, [text('<script>{process.exit(1)}</script>'), el('a', {href:'#custom'}, [text('Jump')])]),
    el('img', {src:'/_zellij/assets/logo.svg', alt:'Logo'}),
    el('pre', {}, [el('code', {className:['language-an-unknown-language']}, [text('literal <code> {brace}')])]),
    el('code', {className:['language-math','math-inline']}, [text('x^2')]),
    el('pre', {}, [el('code', {className:['language-mindmap']}, [text('Root: [A, B]')])]),
  ]},
};

it('rejects executable tags, properties, URL schemes and expression nodes in generated data', () => {
  for (const bad of [el('script'), el('div', {dangerouslySetInnerHTML:{__html:'bad'}}), el('img', {onError:'bad'}), el('a', {href:'javascript:alert(1)'}), {type:'mdxFlowExpression',value:'process.exit(1)'}]) {
    expect(portableDocumentSchema.safeParse({...document, body:{type:'root', children:[bad]}}).success).toBe(false);
  }
  expect(portableDocumentSchema.safeParse({...document,version:2}).success).toBe(false);
});

it('loads and renders native trees with metadata, assets, safe literals, navigation labels and external links', async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'zellij-portable-tree-'));
  try {
    await mkdir(path.join(root,'manual'));
    await mkdir(path.join(root,'assets'));
    await writeFile(path.join(root,'assets/logo.svg'), '<svg xmlns="http://www.w3.org/2000/svg"/>');
    await writeFile(path.join(root,'site.yaml'), 'name: Docs\nhome: manual\nbasePath: /products/docs\n');
    await writeFile(path.join(root,'manual/_sidebar.yaml'), 'title: Docs\nroute: /\nnav:\n - {page: index, label: Welcome}\n - {href: "https://example.com", label: Company}\n - label: Group\n   items: [{page: other, label: Second}]\n');
    await writeFile(path.join(root,'manual/index.zeldoc.json'), JSON.stringify(document));
    await writeFile(path.join(root,'manual/other.mdx'), '# Other\n\nOther page.\n');
    await writeFile(path.join(root,'manual/unlisted.mdx'), '# Unlisted\n\nKept.\n');
    const site = await loadSite(new FolderContentSource({contentDir:root}));
    expect(site.guidePages).toHaveLength(3);
    const sidebar = site.guides[0]!.sidebar;
    expect(flattenSidebar(sidebar).map(p => p.label)).toEqual(['Welcome','Second']);
    expect(findNeighbours(sidebar,'/').next?.route).toBe('/other');
    expect(sidebar[1]).toMatchObject({external:true,route:'https://example.com'});
    const page = site.guidePages.find(p => p.route === '/')!;
    expect(indexGuidePage(page)).toEqual(expect.arrayContaining([
      expect.objectContaining({route:'/',text:expect.stringContaining('searchable')}),
      expect.objectContaining({route:'/#custom',title:'Custom heading'}),
    ]));
    expect((await deepCheck(site,root)).errors).toEqual([]);
    const html = renderToStaticMarkup(await PortableBody({document:page.portable!,site}));
    expect(html).toContain('id="custom"');
    expect(html).toContain('&lt;script&gt;{process.exit(1)}&lt;/script&gt;');
    expect(html).toContain('src="/products/docs/_zellij/assets/logo.svg"');
    expect(html).toContain('mjx-container');
    expect(html).toContain('zellij-mindmap');
    expect(html).toContain('literal');
    expect(html).not.toContain('<script>');
  } finally { await rm(root,{recursive:true,force:true}); }
}, 30000);
