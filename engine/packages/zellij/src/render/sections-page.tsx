import type { Site } from '../content/site';
import type { SectionsPage } from '../content/types';
import { pageHeadingSectionTypes } from '../sections/registry';
import { SectionRenderer } from './section-renderer';
import { emitPageThemeCss } from '../themes/emit';

/**
 * Page layouts are a closed set for the same reason themes are: arbitrary
 * per-product layouts would reintroduce visual drift (spec §5.4).
 */
export function SectionsPageView({ site, page }: { site: Site; page: SectionsPage }) {
  const motion = page.motion ?? site.config.motion;

  const sections = page.sections.map((section, index) => (
    <SectionRenderer
      key={`${section.type}-${index}`}
      section={section}
      site={site}
      defaultMotion={motion}
    />
  ));

  // A page built entirely from sections that render no `<h1>` still needs one.
  // It is hidden rather than shown: on a page like a promo grid the tiles carry
  // the visual weight, and a second visible title would only compete with them.
  const withHeading = pageHeadingSectionTypes();
  const needsHeading = !page.sections.some((section) => withHeading.has(section.type));

  const main = (
    <main
      id="content"
      className={`zellij-layout zellij-layout-${page.layout}`}
      data-zellij-layout={page.layout}
      data-zellij-motion={motion}
      data-zellij-theme={page.theme}
    >
      {page.theme ? <style dangerouslySetInnerHTML={{ __html: emitPageThemeCss(page.theme, site.config) }} /> : null}
      {needsHeading ? <h1 className="zellij-visually-hidden">{page.title}</h1> : null}
      {page.layout === 'stack-sidebar' ? (
        <div className="zellij-sidebar-grid">
          <div className="zellij-sidebar-main">{sections}</div>
          <PageRail page={page} />
        </div>
      ) : (
        sections
      )}
    </main>
  );

  return main;
}

/**
 * The `stack-sidebar` rail. Derived from the page's own section anchors so long
 * feature pages get in-page navigation without extra content authoring.
 */
function PageRail({ page }: { page: SectionsPage }) {
  const anchors = page.sections
    .map((section) => ({
      id: (section as { id?: string }).id,
      label: sectionLabel(section),
    }))
    .filter((entry): entry is { id: string; label: string } => Boolean(entry.id && entry.label));

  if (anchors.length === 0) return null;

  return (
    <aside className="zellij-page-rail" aria-label="On this page">
      <nav className="zellij-page-rail-inner">
        <p className="zellij-page-rail-heading">On this page</p>
        <ul>
          {anchors.map((anchor) => (
            <li key={anchor.id}>
              <a className="zellij-page-rail-link" href={`#${anchor.id}`}>
                {anchor.label}
              </a>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  );
}

/** Best available human label for a section, for rail and TOC use. */
function sectionLabel(section: { heading?: string; headline?: string; type: string }): string {
  return section.heading ?? section.headline ?? titleCase(section.type);
}

function titleCase(value: string): string {
  return value
    .split('-')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}
