import { ZellijContentError } from '../errors';
import type { AuthoredMenuItem } from './model-schema';
import { isExternal } from './paths';
import type { Destination, GuideTree, MenuGroup, MenuItem, MenuLink, SectionsPage } from './types';

/** Label for the drawer that absorbs everything past the overflow threshold. */
export const OVERFLOW_LABEL = 'More';

export interface MenuContext {
  /** Reference name → destination, for resolving `page:`. */
  byName: ReadonlyMap<string, Destination>;
  overflowAfter: number;
}

/**
 * Resolves an authored menu into links. A `page:` that names nothing is a build
 * error listing what is available: broken navigation is the most visible defect
 * a site can ship (spec §5.5).
 */
export function resolveMenu(
  authored: readonly AuthoredMenuItem[],
  context: MenuContext,
): MenuItem[] {
  const problems: string[] = [];

  const resolveHref = (item: AuthoredMenuItem, where: string): string => {
    if (item.url) return item.url;

    const destination = context.byName.get(item.page!);
    if (!destination) {
      const available = [...context.byName.keys()].sort();
      problems.push(
        `  · ${where}: "${item.name}" → page: ${item.page}\n` +
          `    No page or guide folder is named "${item.page}".` +
          (available.length > 0 ? `\n    Available: ${available.join(', ')}` : ''),
      );
      return '#';
    }
    return destination.route;
  };

  const toLink = (item: AuthoredMenuItem, where: string): MenuLink => {
    const href = resolveHref(item, where);
    const external = isExternal(href);
    return {
      label: item.name,
      href,
      external,
      // Leaving the site opens a new tab by default so the visitor keeps their
      // place; an internal link never does.
      newTab: item.newTab ?? external,
      ...(item.description ? { description: item.description } : {}),
      ...(item.icon ? { icon: item.icon } : {}),
    };
  };

  // Overflow applies to every menu, not only a derived one.
  //
  // `menuOverflowAfter` was read when the menu was derived from the content and
  // ignored when a `menu.yaml` authored it — which is the case where it matters,
  // because that is where someone adds a ninth item. The bar did not wrap or
  // scroll; the last entries simply drew on top of the search field. A setting
  // whose whole purpose is "how many fit" should not depend on how the list was
  // produced.
  const items = applyOverflow(authored, context.overflowAfter).map((item): MenuItem => {
    const where = `menu › ${item.name}`;

    if (!item.items) {
      const link = toLink(item, 'menu');
      return {
        label: link.label,
        href: link.href,
        external: link.external,
        newTab: link.newTab,
        ...(link.icon ? { icon: link.icon } : {}),
      };
    }

    // A drawer whose children are themselves lists renders as headed columns;
    // otherwise as a flat dropdown.
    const hasGroups = item.items.some((child) => 'items' in child && Array.isArray(child.items));

    if (hasGroups) {
      const groups: MenuGroup[] = item.items.map((child) => {
        if ('items' in child && Array.isArray(child.items)) {
          return {
            heading: child.name,
            items: child.items.map((leaf) => toLink(leaf as AuthoredMenuItem, where)),
          };
        }
        // A bare leaf among groups becomes a single-item column rather than
        // being dropped.
        return { heading: child.name, items: [toLink(child as AuthoredMenuItem, where)] };
      });
      return { label: item.name, wide: item.wide ?? true, groups };
    }

    return {
      label: item.name,
      ...(item.wide ? { wide: true } : {}),
      children: (item.items as AuthoredMenuItem[]).map((child) => toLink(child, where)),
    };
  });

  if (problems.length > 0) {
    throw new ZellijContentError(
      `Broken menu references (${problems.length}):\n${problems.join('\n')}`,
    );
  }

  return items;
}

/**
 * Builds a menu when none is declared — design §10.
 *
 * The union of pages and guide folders, in that order, because deriving from
 * `pages.yaml` alone would leave guide folders unreachable from the bar. Every
 * destination on the site keeps a way in.
 */
export function deriveMenu(
  pages: readonly SectionsPage[],
  guides: readonly GuideTree[],
): AuthoredMenuItem[] {
  const named = new Set(pages.map((page) => page.name));
  const entries: AuthoredMenuItem[] = [
    ...pages.map((page) => ({ name: page.title, page: page.name })),
    ...guides
      .filter((guide) => !named.has(guide.name))
      .map((guide) => ({ name: guide.title, page: guide.name })),
  ];

  // Overflow is `resolveMenu`'s job, for every menu rather than only this one.
  return entries;
}

/**
 * Collapses everything past `overflowAfter` into one trailing drawer, so a book
 * with thirty chapters still gets a usable bar with no configuration.
 */
export function applyOverflow(
  entries: readonly AuthoredMenuItem[],
  overflowAfter: number,
): AuthoredMenuItem[] {
  if (entries.length <= overflowAfter) return [...entries];

  const visible = entries.slice(0, overflowAfter);
  const rest = entries.slice(overflowAfter);
  return [...visible, { name: OVERFLOW_LABEL, wide: true, items: rest }];
}
