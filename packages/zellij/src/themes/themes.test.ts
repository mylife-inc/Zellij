import { describe, expect, it } from 'vitest';
import { THEMES } from './tokens';
import { emitThemeCss } from './emit';
import { THEME_NAMES, zellijConfigSchema, type ThemeName } from '../config/schema';

function configFor(theme: ThemeName, overrides: Record<string, string> = {}) {
  return zellijConfigSchema.parse({
    product: { name: 'Test' },
    theme,
    ...(Object.keys(overrides).length > 0 ? { overrides } : {}),
  });
}

function switcherConfig(theme: ThemeName, overrides: Record<string, string> = {}) {
  return zellijConfigSchema.parse({
    product: { name: 'Test' },
    theme,
    nav: { themeSwitcher: true },
    ...(Object.keys(overrides).length > 0 ? { overrides } : {}),
  });
}

const PALETTE_KEYS = [
  'bg',
  'fg',
  'muted',
  'surface',
  'border',
  'accent',
  'accentFg',
  'invertedBg',
  'invertedFg',
] as const;

describe('theme inventory', () => {
  it('defines tokens for every declared theme name', () => {
    for (const name of THEME_NAMES) {
      expect(THEMES[name], `missing tokens for "${name}"`).toBeDefined();
    }
    expect(Object.keys(THEMES).sort()).toEqual([...THEME_NAMES].sort());
  });

  it('includes the cupertino theme', () => {
    expect(THEME_NAMES).toContain('cupertino');
    expect(THEMES.cupertino.label).toBe('Cupertino');
  });

  /**
   * Guards against typos in hand-written palettes — a stray alpha channel or a
   * malformed value silently produces an unstyled site rather than an error.
   */
  it.each(THEME_NAMES)('%s uses well-formed colours in both modes', (name) => {
    const theme = THEMES[name];
    for (const mode of ['light', 'dark'] as const) {
      for (const key of PALETTE_KEYS) {
        const value = theme[mode][key];
        expect(value, `${name}.${mode}.${key}`).toMatch(
          /^(#[0-9a-fA-F]{3}|#[0-9a-fA-F]{6}|rgb|hsl|oklch|color-mix|transparent)/,
        );
        // 8-digit hex (with alpha) is almost always an accidental extra pair.
        expect(value, `${name}.${mode}.${key} looks like a mistyped hex`).not.toMatch(
          /^#[0-9a-fA-F]{8}$/,
        );
      }
    }
  });

  it.each(THEME_NAMES)('%s declares complete geometry, type and motion tokens', (name) => {
    const theme = THEMES[name];
    expect(theme.fonts.heading).toBeTruthy();
    expect(theme.fonts.body).toBeTruthy();
    expect(theme.geometry.radius).toBeTruthy();
    expect(theme.geometry.sectionGap).toBeTruthy();
    expect(theme.type.tracking).toBeTruthy();
    expect(theme.motion.duration).toMatch(/ms|s$/);
    expect(theme.motion.ease).toMatch(/cubic-bezier|ease|linear/);
  });

  /**
   * Spec §6: themes must be genuinely distinct, "not palette swaps". Font
   * pairing and geometry are the axes that make that true.
   */
  it('gives themes distinct personalities beyond colour', () => {
    const headings = new Set(Object.values(THEMES).map((t) => t.fonts.heading));
    expect(headings.size).toBeGreaterThanOrEqual(9);

    const radii = new Set(Object.values(THEMES).map((t) => t.geometry.radius));
    expect(radii.size).toBeGreaterThanOrEqual(6);

    const durations = new Set(Object.values(THEMES).map((t) => t.motion.duration));
    expect(durations.size).toBeGreaterThanOrEqual(7);
  });
});

describe('emitThemeCss', () => {
  it('emits only the active theme, with light and dark blocks', () => {
    const css = emitThemeCss(configFor('forest'));

    expect(css).toContain(THEMES.forest.light.accent);
    expect(css).toContain(THEMES.forest.dark.accent);
    expect(css).toContain('prefers-color-scheme: dark');

    // Other themes must not leak into the page.
    expect(css).not.toContain(THEMES.sunburst.light.accent);
  });

  it('raises specificity so emitted tokens beat the stylesheet fallbacks', () => {
    const css = emitThemeCss(configFor('paper'));
    expect(css).toContain(':root:root{');
    expect(css).toContain(':root[data-zellij-mode="dark"]');
  });

  it('emits geometry, type and motion tokens, not just colours', () => {
    const css = emitThemeCss(configFor('cupertino'));
    expect(css).toContain(`--zellij-radius-button: ${THEMES.cupertino.geometry.radiusButton}`);
    expect(css).toContain(`--zellij-section-gap: ${THEMES.cupertino.geometry.sectionGap}`);
    expect(css).toContain(`--zellij-motion-ease: ${THEMES.cupertino.motion.ease}`);
    expect(css).toContain(`--zellij-display-scale: ${THEMES.cupertino.type.displayScale}`);
  });

  it.each(THEME_NAMES)('%s produces a complete token block', (name) => {
    const css = emitThemeCss(configFor(name));
    for (const token of [
      '--zellij-bg',
      '--zellij-fg',
      '--zellij-accent',
      '--zellij-font-heading',
      '--zellij-radius',
      '--zellij-motion-duration',
    ]) {
      expect(css, `${name} is missing ${token}`).toContain(`${token}:`);
    }
  });
});

describe('token overrides', () => {
  it('applies an accent override over the theme value', () => {
    const css = emitThemeCss(configFor('slate', { accent: '#E86A33' }));
    // The override is emitted after the theme value, so it wins.
    expect(css.lastIndexOf('--zellij-accent: #E86A33')).toBeGreaterThan(
      css.indexOf(`--zellij-accent: ${THEMES.slate.light.accent}`),
    );
  });

  it('applies the override in dark mode too', () => {
    const css = emitThemeCss(configFor('slate', { accent: '#E86A33' }));
    const darkBlock = css.slice(css.indexOf(':root[data-zellij-mode="dark"]'));
    expect(darkBlock).toContain('--zellij-accent: #E86A33');
  });

  it('maps the named radius scale', () => {
    const css = emitThemeCss(configFor('mono', { radius: 'xl' }));
    expect(css).toContain('--zellij-radius: 1.5rem');
  });

  it('rejects an unknown radius name at config validation, before rendering', () => {
    // Caught by the schema rather than silently ignored downstream, so the
    // author sees a build error naming the valid options (spec §3).
    expect(() => configFor('mono', { radius: 'enormous' })).toThrow(/radius/);
  });

  /**
   * Override values are interpolated into a stylesheet, so they are validated
   * rather than trusted even though they come from a local config file.
   */
  it('rejects an override that could break out of the style block', () => {
    const css = emitThemeCss(
      configFor('paper', { accent: 'red;} body{display:none} :root{--x:' }),
    );
    expect(css).not.toContain('display:none');
    expect(css).toContain(`--zellij-accent: ${THEMES.paper.light.accent}`);
  });

  it('accepts legitimate non-hex colour syntax', () => {
    const css = emitThemeCss(configFor('paper', { accent: 'rgb(232 106 51)' }));
    expect(css).toContain('--zellij-accent: rgb(232 106 51)');
  });
});

describe('runtime theme switching', () => {
  it('ships only the active theme by default', () => {
    const css = emitThemeCss(configFor('forest'));
    expect(css).toContain(THEMES.forest.light.accent);
    expect(css).not.toContain(THEMES.carbon.light.accent);
  });

  it('ships every theme when the switcher is enabled', () => {
    const css = emitThemeCss(switcherConfig('forest'));
    for (const name of THEME_NAMES) {
      expect(css, `${name} missing`).toContain(`[data-zellij-theme="${name}"]`);
      expect(css, `${name} light palette missing`).toContain(THEMES[name].light.accent);
      expect(css, `${name} dark palette missing`).toContain(THEMES[name].dark.accent);
    }
  });

  /**
   * The active theme must not be emitted at a higher specificity than the
   * others, or switching the attribute on <html> would have no visible effect.
   */
  it('gives no theme precedence over the others', () => {
    const css = emitThemeCss(switcherConfig('forest'));
    expect(css).not.toContain(':root:root');
  });

  /**
   * Each rule also matches the bare attribute, so a swatch carrying
   * `data-zellij-theme` paints itself in the theme it previews.
   */
  it('scopes tokens to any element carrying the theme name, not just :root', () => {
    const css = emitThemeCss(switcherConfig('paper'));
    expect(css).toContain('[data-zellij-theme="carbon"]{');
    expect(css).toContain('[data-zellij-mode="dark"] [data-zellij-theme="carbon"]');
  });

  it('switches structural tokens, not only colours', () => {
    const css = emitThemeCss(switcherConfig('paper'));
    // mono is square; sunburst is round. Both must be present and distinct.
    expect(css).toContain(`--zellij-radius: ${THEMES.mono.geometry.radius}`);
    expect(css).toContain(`--zellij-radius: ${THEMES.sunburst.geometry.radius}`);
    expect(css).toContain(`--zellij-font-heading: ${THEMES.carbon.fonts.heading}`);
  });

  it('applies token overrides to every theme', () => {
    const css = emitThemeCss(switcherConfig('paper', { accent: '#E86A33' }));
    const occurrences = css.match(/--zellij-accent: #E86A33/g) ?? [];
    // One per theme per mode block (light, dark, system).
    expect(occurrences.length).toBeGreaterThanOrEqual(THEME_NAMES.length * 2);
  });
});
