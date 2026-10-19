import { describe, expect, it } from 'vitest';
import type { Element, Root } from 'hast';

import { rehypeDiagrams } from './rehype-mermaid';

/** A ```<lang> fence as rehype sees it: <pre><code class="language-<lang>">. */
function fence(lang: string, code: string): Element {
  return {
    type: 'element',
    tagName: 'pre',
    properties: {},
    children: [
      {
        type: 'element',
        tagName: 'code',
        properties: { className: [`language-${lang}`] },
        children: [{ type: 'text', value: code }],
      },
    ],
  };
}

function run(tree: Root): Root {
  (rehypeDiagrams as unknown as () => (tree: Root) => void)()(tree);
  return tree;
}

const root = (...children: Root['children']): Root => ({ type: 'root', children });

describe('rehypeMermaid', () => {
  it('replaces a mermaid fence with a diagram element carrying its source', () => {
    const tree = run(root(fence('mermaid', 'flowchart LR\n  A --> B\n')));

    expect(tree.children).toHaveLength(1);
    const node = tree.children[0] as Element;
    expect(node.tagName).toBe('mermaid-diagram');
    expect(node.properties?.['code']).toBe('flowchart LR\n  A --> B');
    // The <pre> goes with it: a diagram nested in a code box is not a diagram.
    expect(node.children).toEqual([]);
  });

  it('leaves every other language alone', () => {
    const tree = run(root(fence('ts', 'const a = 1;\n'), fence('yaml', 'type: hero\n')));

    expect(tree.children.map((child) => (child as Element).tagName)).toEqual(['pre', 'pre']);
  });

  it('finds fences nested inside other elements', () => {
    const tree = run(
      root({
        type: 'element',
        tagName: 'div',
        properties: {},
        children: [fence('mermaid', 'mindmap\n  root((A))')],
      }),
    );

    const wrapper = tree.children[0] as Element;
    expect((wrapper.children[0] as Element).tagName).toBe('mermaid-diagram');
  });

  it('converts every fence in a document, not just the first', () => {
    const tree = run(
      root(fence('mermaid', 'graph TD'), fence('ts', 'x'), fence('mermaid', 'pie title P')),
    );

    expect(tree.children.map((child) => (child as Element).tagName)).toEqual([
      'mermaid-diagram',
      'pre',
      'mermaid-diagram',
    ]);
  });

  it('keeps a code element that is not inside a pre untouched', () => {
    // Inline `code` spans carry no language class, and must never be swallowed.
    const tree = run(
      root({
        type: 'element',
        tagName: 'p',
        properties: {},
        children: [
          { type: 'element', tagName: 'code', properties: {}, children: [{ type: 'text', value: 'mermaid' }] },
        ],
      }),
    );

    const paragraph = tree.children[0] as Element;
    expect((paragraph.children[0] as Element).tagName).toBe('code');
  });
});

describe('mindmap fences', () => {
  it('draws the diagram at build time rather than deferring it', () => {
    const tree = run(root(fence('mindmap', 'Zellij:\n  Content: [site.yaml]\n')));

    const figure = tree.children[0] as Element;
    expect(figure.tagName).toBe('figure');

    const svg = figure.children[0] as Element;
    expect(svg.tagName).toBe('svg');
    // Finished markup, not a component to hydrate: nothing about this diagram
    // needs the browser.
    expect(JSON.stringify(svg)).toContain('site.yaml');
  });

  it('fails the build on a mindmap it cannot read', () => {
    // An empty box on the page would look like a styling problem and be
    // debugged as one.
    expect(() => run(root(fence('mindmap', '- a\n- b\n')))).toThrow(/mindmap/i);
  });

  it('names the problem when there is more than one root', () => {
    expect(() => run(root(fence('mindmap', 'A: 1\nB: 2\n')))).toThrow(/one root/);
  });
});

it('leaves Mermaid fences as source when disabled while retaining native mindmaps', () => {
  const tree = root(fence('mermaid', 'graph TD; A-->B'), fence('mindmap', 'Root:\n  Child: []\n'));
  (rehypeDiagrams as unknown as (options: { mermaid: boolean }) => (tree: Root) => void)({ mermaid: false })(tree);
  expect((tree.children[0] as Element).tagName).toBe('pre');
  expect((tree.children[1] as Element).tagName).toBe('figure');
});
