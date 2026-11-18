import { Fragment, useId, type ComponentProps } from 'react';
import { jsx, jsxs } from 'react/jsx-runtime';
import Link from 'next/link';
import { toJsxRuntime } from 'hast-util-to-jsx-runtime';
import { composeMarkdown, safeComposeUrl } from '../../content/compose-markdown';
import { assetHref, publicAssetHref } from '../../content/asset-href';

const mediaHref = (src: string, basePath: string) => src.startsWith('/_zellij/assets/') ? publicAssetHref(src, basePath) : assetHref(src, basePath);

/** Plain Markdown only. Raw HTML is dropped; no MDX evaluation or heading IDs. */
export function ComposeProse({ body, basePath }: { body: string; basePath: string }) {
  const prefix = `compose-${useId()}-`;
  const tree = composeMarkdown(body, prefix);
  const content = toJsxRuntime(tree, { Fragment, jsx, jsxs, components: {
    h1: 'h2',
    a: ({ href, children, ...props }: ComponentProps<'a'>) => !href || !safeComposeUrl(href)
      ? <span>{children}</span> : <Link {...props} href={href.startsWith('assets/') || href.startsWith('/_zellij/assets/') ? mediaHref(href, basePath) : href}>{children}</Link>,
    img: ({ src, alt }: { src?: string; alt?: string }) => src && safeComposeUrl(src)
      ? <img src={mediaHref(src, basePath)} alt={alt ?? ''} loading="lazy" /> : null,
  } });
  return <div className="zellij-compose-prose">{content}</div>;
}
