import Link from 'next/link';
import type { AnchorHTMLAttributes, ReactNode } from 'react';

import { Callout } from './callout';
import { Card, CardGrid } from './card';
import { CodeBlock } from './code-block';
import { Screenshot } from './screenshot';
import { Step, Steps } from './steps';
import { CodeGroup, Tab, Tabs } from './tabs';
import { isExternal } from '../content/paths';

/** Internal links route through next/link so they get basePath and prefetching. */
function MdxLink({ href, children, ...rest }: AnchorHTMLAttributes<HTMLAnchorElement>) {
  if (!href) return <a {...rest}>{children}</a>;

  if (isExternal(href) || href.startsWith('#')) {
    const externalProps = isExternal(href)
      ? { target: '_blank', rel: 'noreferrer noopener' as const }
      : {};
    return (
      <a href={href} {...externalProps} {...rest}>
        {children}
      </a>
    );
  }

  return (
    <Link href={href} {...rest}>
      {children}
    </Link>
  );
}

/**
 * Headings get a hover anchor so readers can link to any section. Ids come from
 * rehype-slug, matching the TOC.
 */
function heading(Tag: 'h2' | 'h3' | 'h4') {
  return function Heading({ id, children }: { id?: string; children?: ReactNode }) {
    if (!id) return <Tag>{children}</Tag>;
    return (
      <Tag id={id} className="zellij-heading">
        <a className="zellij-heading-anchor" href={`#${id}`} aria-label="Link to this section">
          {children}
        </a>
      </Tag>
    );
  };
}

/**
 * The component set available in every guide file without imports (spec §7.2).
 */
export const mdxComponents = {
  a: MdxLink,
  pre: CodeBlock,
  h2: heading('h2'),
  h3: heading('h3'),
  h4: heading('h4'),
  Callout,
  Steps,
  Step,
  Tabs,
  Tab,
  CodeGroup,
  Card,
  CardGrid,
  Screenshot,
};

export type MdxComponents = typeof mdxComponents;
