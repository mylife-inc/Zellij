import { Fragment, type ReactNode } from 'react';
import { jsx, jsxs } from 'react/jsx-runtime';
import Link from 'next/link';
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
  const tree = parser.runSync(parser.parse(body));
  const content = toJsxRuntime(tree, { Fragment, jsx, jsxs, components: {
    h1: 'h2',
    a: ({ href, children }: { href?: string; children?: ReactNode }) => !href || !safe(href)
      ? <span>{children}</span> : <Link href={href.startsWith('assets/') || href.startsWith('/_zellij/assets/') ? mediaHref(href, basePath) : href}>{children}</Link>,
    img: ({ src, alt }: { src?: string; alt?: string }) => src && safe(src)
      ? <img src={mediaHref(src, basePath)} alt={alt ?? ''} loading="lazy" /> : null,
  } });
  return <div className="zellij-compose-prose">{content}</div>;
}
