import type { Site } from '../content/site';
import type { GuidePage } from '../content/types';

/**
 * M1 placeholder frame. M2 replaces the body with the compiled MDX pipeline and
 * adds the sidebar, on-this-page TOC, breadcrumbs, and prev/next links (spec §7).
 */
export function GuidePageView({ site, page }: { site: Site; page: GuidePage }) {
  return (
    <main id="content" className="zellij-layout-guide" data-zellij-layout="guide">
      <article className="zellij-prose">
        <h1>{page.title}</h1>
        {page.description ? <p className="zellij-lead">{page.description}</p> : null}
        <p className="zellij-placeholder">
          Guide rendering arrives in M2 (MDX pipeline). Source: <code>{page.sourceFile}</code> ·
          Sidebar sections loaded: {site.sidebar.length}
        </p>
      </article>
    </main>
  );
}
