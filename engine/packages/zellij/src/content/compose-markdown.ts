import type { Element, Root } from 'hast';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';

const parser = unified().use(remarkParse).use(remarkGfm).use(remarkRehype);
export const safeComposeUrl = (url: string) => !/^[\s\u0000-\u0020]*(?:javascript|vbscript|data):/i.test(url.replace(/[\u0000-\u0020]/g, ''));

/** The renderer and deep checker share plain Markdown semantics, never MDX. */
export function composeMarkdown(body: string, prefix: string): Root {
  const tree = parser.runSync(parser.parse(body));
  // GFM footnotes emit IDs and aria-describedby references. Namespace every
  // generated identifier per instance, including the shared footnote label.
  const elements: Element[] = [];
  const walk = (node: Root | Element | Root['children'][number]): void => {
    if (node.type === 'element') elements.push(node);
    if ('children' in node) node.children.forEach(walk);
  };
  walk(tree);
  const ids = new Map(elements.filter(node => typeof node.properties.id === 'string')
    .map(node => [String(node.properties.id), prefix + node.properties.id]));
  for (const node of elements) {
    if (typeof node.properties.id === 'string') node.properties.id = ids.get(node.properties.id)!;
    const href = node.properties.href;
    if (typeof href === 'string' && href.startsWith('#') && ids.has(href.slice(1))) node.properties.href = '#' + ids.get(href.slice(1));
    for (const key of ['ariaDescribedBy', 'ariaLabelledBy']) {
      const value = node.properties[key];
      if (Array.isArray(value)) node.properties[key] = value.map(id => ids.get(String(id)) ?? id);
    }
  }
  return tree;
}
