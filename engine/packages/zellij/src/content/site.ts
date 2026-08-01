import { ZellijContentError } from '../errors';
import type { SiteConfig } from '../config/schema';
import type { AuthoredMenuItem } from './model-schema';
import { deriveMenu, resolveMenu } from './menu';
import { isExternal, normalizeRoute, splitAnchor } from './paths';
import type {
  ContentSource,
  Destination,
  GuidePage,
  GuideTree,
  MenuItem,
  MenuLink,
  SectionsPage,
} from './types';

export interface Site {
  config: SiteConfig;
  pages: SectionsPage[];
  guides: GuideTree[];
  /** Every guide page across every tree, for route lookup. */
  guidePages: GuidePage[];
  menu: MenuItem[];
  /** Footer columns with `page:` references already resolved to routes. */
  footer: Array<{ heading: string; links: MenuLink[] }>;
  /** What "/" shows. Undefined only for a site with no destinations at all. */
  home?: Destination;
  /**
   * Set when "/" cannot render the home destination in place — a guide folder,
   * or a page whose author gave it an explicit route. "/" then redirects here
   * rather than serving the same content at two URLs.
   */
  homeRedirect?: string;
  /** Every valid internal route on the site. */
  routes: Set<string>;
  /** route → set of section anchor ids declared on that page. */
  anchors: Map<string, Set<string>>;
  warnings: string[];
}

/**
 * Loads and cross-validates the whole site. Link validation runs last because it
 * needs the complete route and anchor inventory (spec §13.3).
 */
export async function loadSite(source: ContentSource): Promise<Site> {
  const warnings: string[] = [];

  const config = await source.loadSiteConfig();
  const [authoredMenu, pages, guides] = await Promise.all([
    source.loadMenu(),
    source.listPages(),
    source.listGuides(),
  ]);

  sortGuides(guides);

  const byName = indexByName(pages, guides);
  const home = resolveHome(config, authoredMenu, pages, guides, byName, warnings);

  // The home destination owns "/" outright rather than being reachable at two
  // URLs, which would split its ranking and duplicate its content. A guide
  // folder can't move — its pages hang below its route — and an explicit
  // `route:` is the author's decision, so both redirect instead.
  let homeRedirect: string | undefined;
  if (home?.kind === 'sections' && !home.routeExplicit) home.route = '/';
  else if (home && home.route !== '/') homeRedirect = home.route;

  const menu = resolveMenu(authoredMenu ?? deriveMenu(pages, guides, config.menuOverflowAfter), {
    byName,
    overflowAfter: config.menuOverflowAfter,
  });

  const footer = resolveFooter(config, byName);
  const guidePages = guides.flatMap((guide) => guide.pages);
  const { routes, anchors } = collectRoutes(pages, guides, guidePages, homeRedirect);

  validateLinks(config, footer, routes, anchors, warnings);

  return {
    config,
    pages,
    guides,
    guidePages,
    menu,
    footer,
    ...(home ? { home } : {}),
    ...(homeRedirect ? { homeRedirect } : {}),
    routes,
    anchors,
    warnings,
  };
}

/**
 * Ordering ladder rungs 2–4 (design §9). Rung 1 — an explicit listing in
 * `menu.yaml` or `pages.yaml` — is honoured by those files' own order and never
 * reaches here.
 */
function sortGuides(guides: GuideTree[]): void {
  guides.sort((a, b) => {
    if (a.order !== undefined && b.order !== undefined) return a.order - b.order;
    if (a.order !== undefined) return -1;
    if (b.order !== undefined) return 1;
    return a.name.localeCompare(b.name);
  });
}

function indexByName(
  pages: readonly SectionsPage[],
  guides: readonly GuideTree[],
): Map<string, Destination> {
  const byName = new Map<string, Destination>();
  const clashes: string[] = [];

  for (const page of pages) {
    if (byName.has(page.name)) clashes.push(`  · two pages are both named "${page.name}"`);
    byName.set(page.name, page);
  }

  for (const guide of guides) {
    const prior = byName.get(guide.name);
    if (prior) {
      const where = prior.kind === 'sections' ? prior.sourceFile : `${prior.sourceDir}/`;
      clashes.push(
        `  · "${guide.name}" names both a page in ${where} and the folder ${guide.sourceDir}/`,
      );
    }
    byName.set(guide.name, guide);
  }

  if (clashes.length > 0) {
    throw new ZellijContentError(`Duplicate page names:\n${clashes.join('\n')}`);
  }

  return byName;
}

/**
 * What "/" shows: `home:` in site.yaml, else the first menu item that resolves
 * to something renderable, else the first page or guide folder (design §4, §10).
 */
function resolveHome(
  config: SiteConfig,
  authoredMenu: readonly AuthoredMenuItem[] | undefined,
  pages: readonly SectionsPage[],
  guides: readonly GuideTree[],
  byName: ReadonlyMap<string, Destination>,
  warnings: string[],
): Destination | undefined {
  if (config.home) {
    const named = byName.get(config.home);
    if (!named) {
      throw new ZellijContentError(
        `site.yaml: home "${config.home}" names no page or guide folder.\n` +
          `  Available: ${[...byName.keys()].sort().join(', ') || '(none)'}`,
        'site.yaml',
      );
    }
    return named;
  }

  // A page the author put at "/" has said what it is more plainly than menu
  // order can. This also keeps the legacy `marketing/landing.yaml` at "/" when
  // alphabetical file order would otherwise pick a different page.
  const declared = pages.find((page) => page.route === '/');
  if (declared) return declared;

  // A drawer or an external link has nothing to render at "/", so the search
  // continues past it rather than leaving the site without a front page.
  for (const item of authoredMenu ?? []) {
    if (item.page) {
      const named = byName.get(item.page);
      if (named) return named;
    }
  }

  const fallback = pages[0] ?? guides[0];
  if (!fallback) {
    warnings.push(
      'This content root has no pages and no guide folders, so the site has no front page. ' +
        'Add a pages.yaml, or a folder containing _sidebar.yaml.',
    );
  }
  return fallback;
}

function collectRoutes(
  pages: readonly SectionsPage[],
  guides: readonly GuideTree[],
  guidePages: readonly GuidePage[],
  homeRedirect: string | undefined,
): { routes: Set<string>; anchors: Map<string, Set<string>> } {
  const routes = new Set<string>();
  const anchors = new Map<string, Set<string>>();
  const owners = new Map<string, string>();
  const clashes: string[] = [];

  const claim = (route: string, owner: string): void => {
    const prior = owners.get(route);
    if (prior) {
      clashes.push(`  · "${route}" is claimed by both ${prior} and ${owner}`);
      return;
    }
    owners.set(route, owner);
    routes.add(route);
  };

  const duplicateAnchors: string[] = [];

  for (const page of pages) {
    claim(page.route, page.sourceFile);
    const ids = new Set<string>();
    for (const section of page.sections) {
      const id = (section as { id?: string }).id;
      if (!id) continue;
      // Two sections sharing an anchor makes "#id" ambiguous and silently
      // sends every link to whichever happens to come first. This is easy to
      // hit when a page uses one section twice, which the model encourages.
      if (ids.has(id)) {
        duplicateAnchors.push(`  · "${page.name}" has two sections with id: ${id}`);
      }
      ids.add(id);
    }
    anchors.set(page.route, ids);
  }

  if (duplicateAnchors.length > 0) {
    throw new ZellijContentError(
      `Duplicate section anchors:\n${duplicateAnchors.join('\n')}\n\n` +
        `Give the second one its own id where the page references it.`,
    );
  }

  for (const guidePage of guidePages) claim(guidePage.route, guidePage.sourceFile);

  // A tree root with no index page is still a valid link target: it redirects
  // to the first page in its navigation.
  for (const guide of guides) if (!routes.has(guide.route)) claim(guide.route, `${guide.sourceDir}/`);

  // "/" is a real route even when it only redirects.
  if (homeRedirect) routes.add('/');

  if (clashes.length > 0) {
    throw new ZellijContentError(`Route collisions:\n${clashes.join('\n')}`);
  }

  return { routes, anchors };
}

/** Resolves footer `page:` references the same way menu items are resolved. */
function resolveFooter(
  config: SiteConfig,
  byName: ReadonlyMap<string, Destination>,
): Array<{ heading: string; links: MenuLink[] }> {
  const problems: string[] = [];

  const columns = config.footer.columns.map((column) => ({
    heading: column.heading,
    links: column.links.map((link): MenuLink => {
      let href = link.href ?? '#';
      if (link.page) {
        const destination = byName.get(link.page);
        if (destination) {
          href = destination.route;
        } else {
          problems.push(
            `  · footer › ${column.heading}: "${link.label}" → page: ${link.page}\n` +
              `    No page or guide folder is named "${link.page}".`,
          );
        }
      }
      const external = isExternal(href);
      return { label: link.label, href, external, newTab: external };
    }),
  }));

  if (problems.length > 0) {
    throw new ZellijContentError(
      `Broken footer references (${problems.length}):\n${problems.join('\n')}`,
    );
  }

  return columns;
}

/** Collects every internal href declared outside `menu.yaml`. */
function collectConfigLinks(
  config: SiteConfig,
  footer: ReadonlyArray<{ heading: string; links: MenuLink[] }>,
): Array<{ label: string; href: string; where: string }> {
  const links: Array<{ label: string; href: string; where: string }> = [];

  for (const cta of config.nav.cta) {
    links.push({ label: cta.label, href: cta.href, where: 'nav.cta' });
  }

  const announcementLink = config.nav.announcement?.link;
  if (announcementLink) {
    links.push({
      label: announcementLink.label,
      href: announcementLink.href,
      where: 'nav.announcement.link',
    });
  }

  for (const column of footer) {
    for (const link of column.links) {
      links.push({ label: link.label, href: link.href, where: `footer › ${column.heading}` });
    }
  }

  return links;
}

/**
 * Fails the build for any link pointing at a nonexistent route or anchor.
 * Reports every problem at once rather than one per build.
 */
function validateLinks(
  config: SiteConfig,
  footer: ReadonlyArray<{ heading: string; links: MenuLink[] }>,
  routes: ReadonlySet<string>,
  anchors: ReadonlyMap<string, Set<string>>,
  warnings: string[],
): void {
  const problems: string[] = [];

  for (const { label, href, where } of collectConfigLinks(config, footer)) {
    if (isExternal(href)) continue;

    const [routePart, anchor] = splitAnchor(href);

    // A bare "#anchor" can't be checked without knowing the current page.
    if (routePart === '') {
      warnings.push(`${where}: "${label}" uses a page-relative anchor "${href}" — not validated.`);
      continue;
    }

    const route = normalizeRoute(routePart);
    if (!routes.has(route)) {
      problems.push(`  · ${where}: "${label}" → ${href}\n    No page resolves to "${route}".`);
      continue;
    }

    if (anchor) {
      const pageAnchors = anchors.get(route);
      // Guide pages have heading anchors we don't index at build time; skip those.
      const isSectionsPage = anchors.has(route);
      if (isSectionsPage && !pageAnchors?.has(anchor)) {
        problems.push(
          `  · ${where}: "${label}" → ${href}\n` +
            `    Page "${route}" declares no section with id: ${anchor}.` +
            (pageAnchors && pageAnchors.size > 0
              ? `\n    Available anchors: ${[...pageAnchors].join(', ')}`
              : `\n    That page declares no section ids at all.`),
        );
      }
    }
  }

  if (problems.length > 0) {
    throw new ZellijContentError(
      `Broken navigation links (${problems.length}):\n${problems.join('\n')}\n\n` +
        `Every href must resolve to a page route, a section anchor, or an absolute URL (spec §5.5).`,
    );
  }
}
