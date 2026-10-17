import type { NavItemModel, NavLeafModel } from './nav-model';

/** Keep an ordered prefix, reserving the More trigger only when necessary. */
export function fittingItems(widths: readonly number[], available: number, more: number, gap: number): number {
  const total = widths.reduce((sum, width) => sum + width, 0) + Math.max(0, widths.length - 1) * gap;
  if (total <= available) return widths.length;
  let used = more;
  let count = 0;
  for (const width of widths) {
    if (used + gap + width > available) break;
    used += gap + width;
    count += 1;
  }
  return count;
}

/** Preserve every destination and its metadata when drawers move into More. */
export function overflowItem(items: readonly NavItemModel[]): NavItemModel {
  if (items.every((item) => item.href)) {
    return { label: 'More', children: items.map((item) => ({ ...item, href: item.href! })) };
  }
  return {
    label: 'More',
    mega: {
      groups: items.flatMap((item) => {
        if (item.mega) return item.mega.groups.map((group) => ({
          heading: `${item.label} · ${group.heading}`,
          items: group.items,
        }));
        if (item.children) return [{ heading: item.label, items: item.children }];
        return [{ heading: '', items: [{ ...item, href: item.href! } satisfies NavLeafModel] }];
      }),
    },
  };
}
