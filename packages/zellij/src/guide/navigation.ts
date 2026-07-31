import type { SidebarSection } from '../content/types';

export interface GuideNeighbour {
  label: string;
  route: string;
}

export interface Breadcrumb {
  label: string;
  route?: string;
}

/** Sidebar order defines reading order, so prev/next follow it exactly. */
export function flattenSidebar(sidebar: readonly SidebarSection[]): GuideNeighbour[] {
  const flat: GuideNeighbour[] = [];
  for (const section of sidebar) {
    for (const item of section.items) {
      flat.push({ label: item.label, route: item.route });
      for (const child of item.children ?? []) {
        flat.push({ label: child.label, route: child.route });
      }
    }
  }
  return flat;
}

export function findNeighbours(
  sidebar: readonly SidebarSection[],
  route: string,
): { previous?: GuideNeighbour; next?: GuideNeighbour } {
  const flat = flattenSidebar(sidebar);
  const index = flat.findIndex((entry) => entry.route === route);
  if (index === -1) return {};

  const previous = index > 0 ? flat[index - 1] : undefined;
  const next = index < flat.length - 1 ? flat[index + 1] : undefined;

  return {
    ...(previous ? { previous } : {}),
    ...(next ? { next } : {}),
  };
}

/**
 * Breadcrumbs run Guide › <sidebar section> › <page>. The section comes from
 * the sidebar rather than the file path, so it matches what the reader sees in
 * the nav rather than how files happen to be arranged on disk.
 */
export function buildBreadcrumbs(
  sidebar: readonly SidebarSection[],
  route: string,
  pageTitle: string,
): Breadcrumb[] {
  const crumbs: Breadcrumb[] = [{ label: 'Guide', route: '/guide' }];
  if (route === '/guide') return crumbs;

  const owning = sidebar.find((section) =>
    section.items.some(
      (item) => item.route === route || item.children?.some((c) => c.route === route),
    ),
  );
  if (owning) crumbs.push({ label: owning.label });

  crumbs.push({ label: pageTitle });
  return crumbs;
}
