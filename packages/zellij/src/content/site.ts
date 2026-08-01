import { ZellijContentError } from '../errors';
import type { ZellijConfig, NavItem } from '../config/schema';
import type { ContentSource, GuidePage, MarketingPage, SidebarNode } from './types';
import type { SidebarEntry } from './page-schema';
import { guideRouteFromFile, isExternal, normalizeRoute, splitAnchor } from './paths';

export interface Site {
  config: ZellijConfig;
  marketingPages: MarketingPage[];
  guidePages: GuidePage[];
  sidebar: SidebarNode[];
  /** Every valid internal route on the site. */
  routes: Set<string>;
  /** route → set of section anchor ids declared on that page. */
  anchors: Map<string, Set<string>>;
  warnings: string[];
}

/**
 * Loads and cross-validates the whole site. Runs after all pages are parsed
 * because link validation needs the complete route + anchor inventory (spec §13.3).
 */
export async function loadSite(source: ContentSource): Promise<Site> {
  const warnings: string[] = [];

  const config = await source.loadConfig();
  const [marketingPages, guidePages, rawSidebar] = await Promise.all([
    source.listMarketingPages(),
    source.listGuidePages(),
    source.loadGuideSidebar(),
  ]);

  const routes = new Set<string>();
  const anchors = new Map<string, Set<string>>();

  for (const page of marketingPages) {
    routes.add(page.route);
    const ids = new Set<string>();
    for (const section of page.sections) {
      const id = (section as { id?: string }).id;
      if (id) ids.add(id);
    }
    anchors.set(page.route, ids);
  }

  const guideRoutes = new Set<string>();
  for (const page of guidePages) {
    if (routes.has(page.route)) {
      throw new ZellijContentError(
        `Route collision: "${page.route}" is claimed by both a marketing page and ${page.sourceFile}.`,
      );
    }
    routes.add(page.route);
    guideRoutes.add(page.route);
  }

  // The guide index is a valid link target even with no index.mdx on disk.
  if (guidePages.length > 0) routes.add('/guide');

  const sidebar = resolveSidebar(rawSidebar, guidePages, warnings);
  validateLinks(config, routes, anchors, warnings);

  return { config, marketingPages, guidePages, sidebar, routes, anchors, warnings };
}

/**
 * Turns `_sidebar.yaml` into resolved links. Entries with no matching file are a
 * build error; files missing from the sidebar are a warning (spec §7.1).
 */
function resolveSidebar(
  entries: readonly SidebarEntry[],
  guidePages: readonly GuidePage[],
  warnings: string[],
): SidebarNode[] {
  const byRoute = new Map(guidePages.map((p) => [p.route, p]));
  const referenced = new Set<string>();
  const missing: string[] = [];

  function resolveLeaf(slug: string): SidebarNode | undefined {
    const route = guideRouteFromFile(slug);
    const page = byRoute.get(route);
    if (!page) {
      missing.push(`  · "${slug}" → expected guide/${slug}.mdx (no such file)`);
      return undefined;
    }
    referenced.add(route);
    return { kind: 'link', label: page.title, route };
  }

  /**
   * Walks the authored tree to any depth. The previous version resolved only
   * string entries one level down, so a group nested inside a group was
   * silently discarded — valid per the schema, gone from the output, with no
   * error to say so.
   */
  function resolveNodes(nodes: readonly SidebarEntry[]): SidebarNode[] {
    const out: SidebarNode[] = [];

    for (const node of nodes) {
      if (typeof node === 'string') {
        const leaf = resolveLeaf(node);
        if (leaf) out.push(leaf);
        continue;
      }

      const children = resolveNodes(node.items);
      // An empty group would render as an unopenable heading.
      if (children.length > 0) out.push({ kind: 'group', label: node.label, children });
    }

    return out;
  }

  const tree = resolveNodes(entries);

  if (missing.length > 0) {
    throw new ZellijContentError(
      `guide/_sidebar.yaml references pages that do not exist:\n${missing.join('\n')}`,
      'guide/_sidebar.yaml',
    );
  }

  for (const page of guidePages) {
    if (!referenced.has(page.route) && page.route !== '/guide') {
      warnings.push(
        `${page.sourceFile} is not listed in guide/_sidebar.yaml — the page is still generated, but nothing links to it.`,
      );
    }
  }

  return tree;
}

/** Collects every internal href declared in nav and footer config. */
function collectConfigLinks(config: ZellijConfig): Array<{ label: string; href: string; where: string }> {
  const links: Array<{ label: string; href: string; where: string }> = [];

  const pushNavItem = (item: NavItem): void => {
    if (item.href) links.push({ label: item.label, href: item.href, where: 'nav.links' });
    for (const child of item.children ?? []) {
      links.push({ label: child.label, href: child.href, where: `nav.links › ${item.label}` });
    }
    for (const group of item.mega?.groups ?? []) {
      for (const child of group.items) {
        links.push({
          label: child.label,
          href: child.href,
          where: `nav.links › ${item.label} › ${group.heading}`,
        });
      }
    }
  };

  for (const item of config.nav.links) pushNavItem(item);
  for (const cta of config.nav.cta) links.push({ label: cta.label, href: cta.href, where: 'nav.cta' });

  const announcementLink = config.nav.announcement?.link;
  if (announcementLink) {
    links.push({
      label: announcementLink.label,
      href: announcementLink.href,
      where: 'nav.announcement.link',
    });
  }

  for (const column of config.footer.columns) {
    for (const link of column.links) {
      links.push({ label: link.label, href: link.href, where: `footer › ${column.heading}` });
    }
  }

  return links;
}

/**
 * Fails the build for any nav/footer link pointing at a nonexistent route or
 * anchor. Broken nav is the most visible possible defect on a marketing site
 * (spec §5.5), so this reports every problem at once.
 */
function validateLinks(
  config: ZellijConfig,
  routes: ReadonlySet<string>,
  anchors: ReadonlyMap<string, Set<string>>,
  warnings: string[],
): void {
  const problems: string[] = [];

  for (const { label, href, where } of collectConfigLinks(config)) {
    if (isExternal(href)) continue;

    const [routePart, anchor] = splitAnchor(href);

    // A bare "#anchor" can't be checked without knowing the current page.
    if (routePart === '') {
      warnings.push(`${where}: "${label}" uses a page-relative anchor "${href}" — not validated.`);
      continue;
    }

    const route = normalizeRoute(routePart);
    if (!routes.has(route)) {
      problems.push(
        `  · ${where}: "${label}" → ${href}\n` +
          `    No page resolves to "${route}".`,
      );
      continue;
    }

    if (anchor) {
      const pageAnchors = anchors.get(route);
      // Guide pages have heading anchors we don't index at build time; skip those.
      const isMarketing = anchors.has(route);
      if (isMarketing && !pageAnchors?.has(anchor)) {
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
        `Every nav/footer href must resolve to a page route, a section anchor, or an absolute URL (spec §5.5).`,
    );
  }
}
