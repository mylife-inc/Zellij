import Link from 'next/link';
import { PortableBody } from './portable-body';
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
export async function GuidePageView({ site, page }: { site: Site; page: GuidePage }) {
  const options = await mdxOptions();
  const toc = page.portable
    ? page.portable.headings.flatMap(h => h.depth === 2 || h.depth === 3 ? [{ ...h, depth: h.depth as 2 | 3 }] : [])
    : extractToc(page.body, { remarkPlugins: options.mdxOptions.remarkPlugins });
  // A site can have any number of guide trees, so a page navigates within its
  // own — `page.guide` is the route of the folder it came from.
  const tree = site.guides.find((guide) => guide.route === page.guide);
  const sidebar = tree?.sidebar ?? [];
  const { previous, next } = findNeighbours(sidebar, page.route);
  const breadcrumbs = buildBreadcrumbs(sidebar, page.route, page.title, {
    label: tree?.title ?? 'Guide',
    route: tree?.route ?? page.guide,
  });

  return (
    <div className="zellij-guide">
      <aside className="zellij-guide-sidebar">
        <GuideSidebar sidebar={sidebar} label={tree?.title ?? 'Guide'} />
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
          {/* The title is rendered here, not in the body, so it stays a single
              source of truth across <title>, sidebar, breadcrumbs and search. */}
          {page.portable ? <PortableBody document={page.portable} site={site} /> : <>
            <h1>{page.title}</h1>
            {page.description ? <p className="zellij-lead">{page.description}</p> : null}
            <MDXRemote source={page.body} components={mdxComponents(site)} options={options} />
          </>}
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
