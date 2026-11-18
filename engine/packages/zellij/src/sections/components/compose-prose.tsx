import { Fragment, useId, type ComponentProps } from 'react';
import { jsx, jsxs } from 'react/jsx-runtime';
import Link from 'next/link';
import type { Element, Root } from 'hast';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import { toJsxRuntime } from 'hast-util-to-jsx-runtime';
import { assetHref, publicAssetHref } from '../../content/asset-href';

const mediaHref = (src: string, basePath: string) => src.startsWith('/_zellij/assets/') ? publicAssetHref(src, basePath) : assetHref(src, basePath);
const parser = unified().use(remarkParse).use(remarkGfm).use(remarkRehype);
const safe = (url: string) => !/^[\s\u0000-\u0020]*(?:javascript|vbscript|data):/i.test(url.replace(/[\u0000-\u0020]/g, ''));

/** Plain Markdown only. Raw HTML is dropped; no MDX evaluation or heading IDs. */
export function ComposeProse({ body, basePath }: { body: string; basePath: string }) {
  const prefix = `compose-${useId()}-`;
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
  const content = toJsxRuntime(tree, { Fragment, jsx, jsxs, components: {
    h1: 'h2',
    a: ({ href, children, ...props }: ComponentProps<'a'>) => !href || !safe(href)
      ? <span>{children}</span> : <Link {...props} href={href.startsWith('assets/') || href.startsWith('/_zellij/assets/') ? mediaHref(href, basePath) : href}>{children}</Link>,
    img: ({ src, alt }: { src?: string; alt?: string }) => src && safe(src)
      ? <img src={mediaHref(src, basePath)} alt={alt ?? ''} loading="lazy" /> : null,
  } });
  return <div className="zellij-compose-prose">{content}</div>;
}
