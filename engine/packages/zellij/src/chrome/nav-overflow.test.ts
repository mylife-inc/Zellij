import { describe, expect, it } from 'vitest';
import { fittingItems, overflowItem } from './nav-overflow';

describe('width-based navigation overflow', () => {
  it('uses all space when every item fits, without reserving a redundant More button', () => {
    expect(fittingItems([90, 110, 100], 308, 70, 4)).toBe(3);
  });

  it('reserves More and the actual gaps when only a prefix fits', () => {
    expect(fittingItems([90, 110, 100], 278, 70, 4)).toBe(2);
    expect(fittingItems([90, 110, 100], 277.5, 70, 4)).toBe(1);
    expect(fittingItems([300, 110], 280, 70, 4)).toBe(0);
  });

  it('handles empty menus and grows back to the full menu', () => {
    expect(fittingItems([], 100, 70, 4)).toBe(0);
    expect([100, 180, 240, 500].map((space) => fittingItems([90, 110, 100], space, 70, 4)))
      .toEqual([0, 1, 1, 3]);
  });

  it('keeps outbound behavior and descriptive metadata on flat links', () => {
    const leaf = { label: 'Company', href: 'https://example.com', external: true, newTab: true, description: 'Owner' };
    expect(overflowItem([leaf]).children).toEqual([leaf]);
  });

  it('preserves links, dropdowns and every mega-menu group in order', () => {
    const leaf = { label: 'Home', href: '/' };
    const nested = { label: 'Docs', href: '/guide', description: 'Start here' };
    const external = { label: 'Source', href: 'https://example.com', external: true, newTab: false };
    expect(overflowItem([
      leaf,
      { label: 'Learn', children: [nested] },
      { label: 'Explore', mega: { groups: [{ heading: 'Community', items: [external] }] } },
    ]).mega?.groups).toEqual([
      { heading: '', items: [leaf] },
      { heading: 'Learn', items: [nested] },
      { heading: 'Explore · Community', items: [external] },
    ]);
  });
});
