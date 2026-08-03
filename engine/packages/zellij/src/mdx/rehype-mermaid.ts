import type { Element, Root, Text } from 'hast';
import type { Plugin } from 'unified';

import { renderMindmap } from './mindmap';

/** Reads the text of a `<code>` node, which for a fence is a single text child. */
function codeText(node: Element): string {
  return node.children
    .filter((child): child is Text => child.type === 'text')
    .map((child) => child.value)
    .join('');
}

function fenceLanguage(node: Element): string | undefined {
  if (node.tagName !== 'code') return undefined;
  const className = node.properties?.['className'];
  const names: unknown[] = Array.isArray(className) ? className : [className];

  for (const name of names) {
    if (typeof name === 'string' && name.startsWith('language-')) {
      return name.slice('language-'.length);
    }
  }

  return undefined;
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
export const rehypeDiagrams: Plugin<[], Root> = () => (tree) => {
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
          (grandchild): grandchild is Element => grandchild.type === 'element',
        );
        const language = code ? fenceLanguage(code) : undefined;

        if (code && language === 'mermaid') {
          children[i] = {
            type: 'element',
            tagName: 'mermaid-diagram',
            properties: { code: codeText(code).trimEnd() },
            children: [],
          };
          continue;
        }

        if (code && language === 'mindmap') {
          // Drawn here rather than handed to a component: the layout is pure
          // arithmetic, so the finished SVG can go straight into the page and
          // nothing has to run in the browser to see it.
          try {
            children[i] = {
              type: 'element',
              tagName: 'figure',
              properties: { className: ['zellij-mindmap'] },
              children: [renderMindmap(codeText(code)) as Element],
            };
          } catch (err) {
            // A malformed mindmap fails the build naming the problem, rather
            // than rendering an empty box that looks like a styling bug.
            throw new Error(
              `Could not draw a mindmap: ${err instanceof Error ? err.message : String(err)}`,
            );
          }
          continue;
        }
      }

      visit(child);
    }
  };

  visit(tree);
};
