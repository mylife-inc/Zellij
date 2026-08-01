import { describe, expect, it } from 'vitest';
import { resolveIcon } from './icons';

describe('resolveIcon', () => {
  /**
   * Regression: lucide icons are forwardRef *objects*, not functions. An
   * over-strict `typeof === 'function'` guard rejected every valid icon.
   */
  it.each(['book', 'rocket', 'users', 'shield-check', 'bar-chart-3', 'refresh-cw'])(
    'resolves the built-in icon %s',
    (name) => {
      expect(() => resolveIcon(name, 'test')).not.toThrow();
      expect(resolveIcon(name, 'test')).toBeDefined();
    },
  );

  it('maps kebab-case names with digits to lucide PascalCase exports', () => {
    expect(resolveIcon('bar-chart-3', 'test')).toBe(resolveIcon('bar-chart-3', 'test'));
  });

  it('fails the build on an unknown icon, naming the icon and its context', () => {
    expect(() => resolveIcon('not-a-real-icon', 'nav.links › Resources')).toThrow(
      /Unknown icon "not-a-real-icon" in nav\.links › Resources/,
    );
  });

  it('reports the export name it looked for, to make typos obvious', () => {
    expect(() => resolveIcon('shieldcheck', 'test')).toThrow(/"Shieldcheck" export/);
  });

  it('does not resolve non-icon lucide exports as icons', () => {
    // `createLucideIcon` is a factory function, not a renderable icon; guard
    // against a lookup that would silently render something nonsensical.
    expect(() => resolveIcon('default', 'test')).toThrow(/Unknown icon/);
  });
});
