import type { Element, Root, Text } from 'hast';
import type { Plugin } from 'unified';

/** Reads the text of a `<code>` node, which for a fence is a single text child. */
function codeText(node: Element): string {
  return node.children
    .filter((child): child is Text => child.type === 'text')
    .map((child) => child.value)
    .join('');
}

function isMermaidFence(node: Element): boolean {
  if (node.tagName !== 'code') return false;
  const className = node.properties?.['className'];
  const names: unknown[] = Array.isArray(className) ? className : [className];
  return names.includes('language-mermaid');
}

/**
 * Rewrites ```mermaid fences into `<mermaid-diagram>` elements carrying their
 * source.
 *
 * This has to run *before* the syntax highlighter. Shiki would otherwise turn
 * the diagram into a tree of coloured spans, and the renderer needs the source
 * intact — a diagram is a drawing to be laid out, not code to be read.
 *
 * The element is a custom tag rather than a `<div>` so the MDX component map
 * can claim it: only that gives the diagram a client component to hydrate with.
 */
export const rehypeMermaid: Plugin<[], Root> = () => (tree) => {
  const visit = (node: Root | Element): void => {
    const children = node.children;
    if (!children) return;

    for (let i = 0; i < children.length; i += 1) {
      const child = children[i];
      if (!child || child.type !== 'element') continue;

      // Fences arrive as <pre><code class="language-mermaid">, so the pre is
      // replaced wholesale — leaving it would nest a diagram inside a code box.
      if (child.tagName === 'pre') {
        const code = child.children.find(
          (grandchild): grandchild is Element =>
            grandchild.type === 'element' && isMermaidFence(grandchild),
        );

        if (code) {
          children[i] = {
            type: 'element',
            tagName: 'mermaid-diagram',
            properties: { code: codeText(code).trimEnd() },
            children: [],
          };
          continue;
        }
      }

      visit(child);
    }
  };

  visit(tree);
};
