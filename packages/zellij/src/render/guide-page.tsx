import Link from 'next/link';
import { MDXRemote } from 'next-mdx-remote/rsc';

import type { Site } from '../content/site';
import type { GuidePage } from '../content/types';
import { buildBreadcrumbs, findNeighbours } from '../guide/navigation';
import { GuideSidebar } from '../guide/sidebar';
import { TableOfContents } from '../guide/table-of-contents';
import { extractToc } from '../guide/toc';
import { mdxComponents } from '../mdx/components';
import { mdxOptions } from '../mdx/pipeline';

/**
 * Docs layout: left sidebar, prose column, right-hand "On this page" rail,
 * breadcrumbs, and previous/next links (spec §7).
 */
export function GuidePageView({ site, page }: { site: Site; page: GuidePage }) {
  const toc = extractToc(page.body);
  const { previous, next } = findNeighbours(site.sidebar, page.route);
  const breadcrumbs = buildBreadcrumbs(site.sidebar, page.route, page.title);

  return (
    <div className="zellij-guide">
      <aside className="zellij-guide-sidebar">
        <GuideSidebar sidebar={site.sidebar} />
      </aside>

      <main id="content" className="zellij-guide-main" data-zellij-layout="guide">
        <nav className="zellij-breadcrumbs" aria-label="Breadcrumb">
          <ol>
            {breadcrumbs.map((crumb, index) => (
              <li key={`${crumb.label}-${index}`}>
                {crumb.route ? (
                  <Link href={crumb.route}>{crumb.label}</Link>
                ) : (
                  <span
                    {...(index === breadcrumbs.length - 1
                      ? { 'aria-current': 'page' as const }
                      : {})}
                  >
                    {crumb.label}
                  </span>
                )}
              </li>
            ))}
          </ol>
        </nav>

        <article className="zellij-prose">
          <MDXRemote source={page.body} components={mdxComponents} options={mdxOptions()} />
        </article>

        {previous || next ? (
          <nav className="zellij-guide-pager" aria-label="Guide pages">
            {previous ? (
              <Link className="zellij-guide-pager-link" data-direction="prev" href={previous.route}>
                <span className="zellij-guide-pager-label">Previous</span>
                <span className="zellij-guide-pager-title">{previous.label}</span>
              </Link>
            ) : (
              <span />
            )}
            {next ? (
              <Link className="zellij-guide-pager-link" data-direction="next" href={next.route}>
                <span className="zellij-guide-pager-label">Next</span>
                <span className="zellij-guide-pager-title">{next.label}</span>
              </Link>
            ) : null}
          </nav>
        ) : null}
      </main>

      <TableOfContents entries={toc} />
    </div>
  );
}
