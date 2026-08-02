import Link from 'next/link';
import type { AnchorHTMLAttributes, ReactNode } from 'react';

import { Callout } from './callout';
import { Card, CardGrid } from './card';
import { CodeBlock } from './code-block';
import { Screenshot } from './screenshot';
import { SectionPreview, type SectionPreviewProps } from './section-preview';
import { Step, Steps } from './steps';
import { CodeGroup, Tab, Tabs } from './tabs';
import { isExternal } from '../content/paths';
import type { Site } from '../content/site';
import { ZellijContentError } from '../errors';

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
 *
 * A factory rather than a constant, because MDX components receive nothing from
 * the page they render in and two of them need it: `<Screenshot>` has to bake
 * the basePath into its `next/image` source, and `<SectionPreview>` needs the
 * site to look an example up and to render it.
 */
export function mdxComponents(site?: Site) {
  const basePath = site?.config.basePath ?? '';
  return {
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
    Screenshot: (props: Parameters<typeof Screenshot>[0]) => (
      <Screenshot basePath={basePath} {...props} />
    ),
    SectionPreview: (props: SectionPreviewProps) => {
      if (!site) {
        throw new ZellijContentError(
          '<SectionPreview> needs the site in scope, and this MDX was compiled without it.',
        );
      }
      return <SectionPreview site={site} {...props} />;
    },
  };
}

export type MdxComponents = ReturnType<typeof mdxComponents>;
