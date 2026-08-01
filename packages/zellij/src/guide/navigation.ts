import type { SidebarGroup, SidebarNode } from '../content/types';

export interface GuideNeighbour {
  label: string;
  route: string;
}

export interface Breadcrumb {
  label: string;
  route?: string;
}

/**
 * Sidebar order defines reading order, so prev/next follow it exactly —
 * depth-first, so a nested group reads as part of the flow rather than being
 * skipped over.
 */
export function flattenSidebar(sidebar: readonly SidebarNode[]): GuideNeighbour[] {
  const flat: GuideNeighbour[] = [];

  const walk = (nodes: readonly SidebarNode[]): void => {
    for (const node of nodes) {
      if (node.kind === 'link') flat.push({ label: node.label, route: node.route });
      else walk(node.children);
    }
  };

  walk(sidebar);
  return flat;
}

export function findNeighbours(
  sidebar: readonly SidebarNode[],
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

/** Every group between the tree root and `route`, outermost first. */
export function findAncestors(
  sidebar: readonly SidebarNode[],
  route: string,
): SidebarGroup[] {
  const trail: SidebarGroup[] = [];

  const walk = (nodes: readonly SidebarNode[]): boolean => {
    for (const node of nodes) {
      if (node.kind === 'link') {
        if (node.route === route) return true;
        continue;
      }

      trail.push(node);
      if (walk(node.children)) return true;
      trail.pop();
    }
    return false;
  };

  return walk(sidebar) ? trail : [];
}

/**
 * Breadcrumbs run Guide › <group…> › <page>. Groups come from the sidebar
 * rather than the file path, so they match what the reader sees in the nav
 * rather than how files happen to be arranged on disk — and every level of
 * nesting is included, not just the first.
 */
export function buildBreadcrumbs(
  sidebar: readonly SidebarNode[],
  route: string,
  pageTitle: string,
): Breadcrumb[] {
  const crumbs: Breadcrumb[] = [{ label: 'Guide', route: '/guide' }];
  if (route === '/guide') return crumbs;

  for (const group of findAncestors(sidebar, route)) {
    crumbs.push({ label: group.label });
  }

  crumbs.push({ label: pageTitle });
  return crumbs;
}
