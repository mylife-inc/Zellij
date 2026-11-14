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
    el('span', {id:'_zel-eq-eq:energy',className:['zellij-equation']}, [
      el('span', {className:['zellij-equation-math']}, [el('code', {className:['language-math','math-display']}, [text('E=mc^2')])]),
      el('a', {href:'#_zel-eq-eq%3Aenergy',className:['zellij-equation-number'],ariaLabel:'Equation 1.1'}, [text('(1.1)')]),
    ]),
    el('p', {}, [el('a', {href:'/other#_zel-eq-eq%3Asum',title:'Equation 2.1'}, [text('(2.1)')])]),
    el('div', {id:'_zel-stmt-thm:energy',className:['zellij-statement','zellij-statement-theorem']}, [
      el('p', {className:['zellij-statement-head']}, [el('strong', {}, [text('Theorem 1.3 (Energy & equality)')])]),
      el('div', {className:['zellij-statement-body']}, [el('p', {}, [text('An argument.'),el('a',{href:'#_zel-stmt-thm%3Aenergy',title:'Theorem 1.3'},[text('Theorem 1.3')])])]),
    ]),
    el('div', {className:['zellij-statement','zellij-statement-proof']}, [
      el('p', {className:['zellij-statement-head']}, [text('Proof')]),
      el('div', {className:['zellij-statement-body']}, [el('p',{},[text('The conclusion follows.')])]),
      el('p', {className:['zellij-statement-qed'],ariaHidden:'true'}, [text('∎')]),
    ]),
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
    await writeFile(path.join(root,'manual/other.mdx'), '# Other\n\nOther page.\n\n<span id="_zel-eq-eq:sum" />\n');
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
    expect(html).toContain('id="_zel-eq-eq:energy"');
    expect(html).toContain('aria-label="Equation 1.1"');
    expect(html).toContain('href="/other#_zel-eq-eq%3Asum"');
    expect(html).toContain('(2.1)');
    expect(html).toContain('zellij-mindmap');
    expect(html).toContain('id="_zel-stmt-thm:energy"');
    expect(html).toContain('Theorem 1.3 (Energy &amp; equality)');
    expect(html).toContain('href="#_zel-stmt-thm%3Aenergy"');
    expect(html.match(/<p\b[^>]*class="zellij-statement-qed"[^>]*>/)?.[0]).toContain('aria-hidden="true"');
    expect(html).toContain('literal');
    expect(html).not.toContain('<script>');
  } finally { await rm(root,{recursive:true,force:true}); }
}, 30000);
