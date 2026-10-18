import type { Root } from 'hast';
import { toJsxRuntime } from 'hast-util-to-jsx-runtime';
import { Fragment, jsx, jsxs } from 'react/jsx-runtime';
import { bundledLanguages } from 'shiki';
import { unified } from 'unified';
import type { Site } from '../content/site';
import type { PortableDocument } from '../content/portable-document';
import { walkPortable } from '../content/portable-document';
import { mdxComponents } from '../mdx/components';
import { mdxOptions, supportsMaths } from '../mdx/pipeline';

/** Same highlighter, diagrams, theme and link components as hosted guides;
 * only the parsing step differs. No eval or raw HTML crosses the bridge. */
export async function PortableBody({ document, site }: { document: PortableDocument; site: Site }) {
  let maths = false;
  let diagrams = false;
  walkPortable(document.body, node => {
    if (node.type !== 'element') return;
    const classes = node.properties.className;
    if (Array.isArray(classes)) {
      maths ||= classes.includes('language-math');
      diagrams ||= classes.includes('language-mermaid');
    }
  });
  if (maths && !await supportsMaths()) throw new Error('This portable document requires maths; use a Zellij engine image with maths enabled.');
  if (diagrams) {
    try { await import('mermaid'); }
    catch { throw new Error('This portable document requires Mermaid; use a Zellij engine image with diagrams enabled.'); }
  }
  const options = await mdxOptions();
  const input = structuredClone(document.body);
  walkPortable(input, node => {
    if (node.type !== 'element' || node.tagName !== 'code') return;
    const classes = node.properties.className;
    if (!Array.isArray(classes)) return;
    node.properties.className = classes.map(value => {
      const lang = String(value).replace(/^language-/, '');
      return String(value).startsWith('language-') && !['text', 'plaintext', 'math', 'mermaid', 'mindmap'].includes(lang) && !(lang in bundledLanguages)
        ? 'language-text' : value;
    });
  });
  const tree = await unified().use(options.mdxOptions.rehypePlugins).run(input as Root);
  return toJsxRuntime(tree as Root, { Fragment, jsx, jsxs, components: mdxComponents(site) });
}
