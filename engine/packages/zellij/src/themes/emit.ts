import type { SiteConfig } from '../config/schema';
import { THEMES, type Palette, type ThemeTokens } from './tokens';

const RADIUS_SCALE: Record<string, string> = {
  none: '0',
  sm: '0.25rem',
  md: '0.5rem',
  lg: '1rem',
  xl: '1.5rem',
};

/**
 * Rejects anything that is not a plain CSS colour or length. Override values
 * come from a config file rather than a request, but they are interpolated into
 * a stylesheet, so they are validated rather than trusted.
 */
function safeValue(value: string): string | undefined {
  return /^[#a-zA-Z0-9(),.%\s/-]+$/.test(value) && !value.includes('</') ? value.trim() : undefined;
}

function paletteVars(palette: Palette): string[] {
  return [
    `--zellij-bg: ${palette.bg};`,
    `--zellij-fg: ${palette.fg};`,
    `--zellij-muted: ${palette.muted};`,
    `--zellij-surface: ${palette.surface};`,
    `--zellij-border: ${palette.border};`,
    `--zellij-accent: ${palette.accent};`,
    `--zellij-accent-fg: ${palette.accentFg};`,
    `--zellij-inverted-bg: ${palette.invertedBg};`,
    `--zellij-inverted-fg: ${palette.invertedFg};`,
  ];
}

function structuralVars(theme: ThemeTokens): string[] {
  return [
    `--zellij-font-heading: ${theme.fonts.heading};`,
    `--zellij-font-body: ${theme.fonts.body};`,
    `--zellij-font-mono: ${theme.fonts.mono};`,
    `--zellij-radius: ${theme.geometry.radius};`,
    `--zellij-radius-button: ${theme.geometry.radiusButton};`,
    `--zellij-radius-media: ${theme.geometry.radiusMedia};`,
    `--zellij-container: ${theme.geometry.container};`,
    `--zellij-section-gap: ${theme.geometry.sectionGap};`,
    `--zellij-nav-height: ${theme.geometry.navHeight};`,
    `--zellij-tracking: ${theme.type.tracking};`,
    `--zellij-heading-weight: ${theme.type.headingWeight};`,
    `--zellij-display-scale: ${theme.type.displayScale};`,
    `--zellij-shadow-card: ${theme.shadow.card};`,
    `--zellij-shadow-float: ${theme.shadow.float};`,
    `--zellij-motion-duration: ${theme.motion.duration};`,
    `--zellij-motion-ease: ${theme.motion.ease};`,
    `--zellij-motion-distance: ${theme.motion.distance};`,
    `--zellij-motion-stagger: ${theme.motion.stagger};`,
  ];
}

/** Product-level token overrides, applied on top of the theme (spec §6). */
function overrideVars(overrides: SiteConfig['overrides']): string[] {
  const vars: string[] = [];

  const accent = overrides.accent ? safeValue(overrides.accent) : undefined;
  if (accent) vars.push(`--zellij-accent: ${accent};`);

  const accentFg = overrides.accentForeground
    ? safeValue(overrides.accentForeground)
    : undefined;
  if (accentFg) vars.push(`--zellij-accent-fg: ${accentFg};`);

  if (overrides.radius) {
    const radius = RADIUS_SCALE[overrides.radius];
    if (radius) {
      vars.push(`--zellij-radius: ${radius};`);
      vars.push(`--zellij-radius-media: ${radius};`);
    }
  }

  const heading = overrides.fontHeading ? safeValue(overrides.fontHeading) : undefined;
  if (heading) vars.push(`--zellij-font-heading: ${heading};`);

  const body = overrides.fontBody ? safeValue(overrides.fontBody) : undefined;
  if (body) vars.push(`--zellij-font-body: ${body};`);

  return vars;
}

/**
 * Builds the stylesheet for the active theme only, so a site ships one theme's
 * tokens rather than all eleven. Dark values are emitted for both the explicit
 * `mode: dark` selector and the `system` + prefers-dark case.
 */
export function emitThemeCss(config: SiteConfig): string {
  const theme = THEMES[config.theme];
  const overrides = overrideVars(config.overrides);

  const root = [...structuralVars(theme), ...paletteVars(theme.light), ...overrides];
  const dark = [...paletteVars(theme.dark), ...overrides];

  if (config.nav.themeSwitcher) return emitAllThemes(config);

  // Selectors are deliberately doubled (`:root:root`) to raise specificity above
  // the fallback tokens in zellij.css. That keeps this correct no matter which
  // order Next emits the stylesheet link and this inline <style>.
  return [
    `:root:root{${root.join('')}}`,
    `:root[data-zellij-mode="dark"]{${dark.join('')}}`,
    `@media (prefers-color-scheme: dark){:root[data-zellij-mode="system"]{${dark.join('')}}}`,
  ].join('');
}

/**
 * Every theme's tokens, each scoped to its own name — the stylesheet a runtime
 * switcher needs (`nav.themeSwitcher`).
 *
 * Two selectors per rule, and both earn their place:
 *
 *   `:root[data-zellij-theme="x"]`  styles the page when <html> carries the
 *                                   name, which is what switching changes.
 *   `[data-zellij-theme="x"]`       styles *any* element carrying it, so a
 *                                   swatch in the picker paints itself in the
 *                                   theme it represents with no JavaScript.
 *
 * Nothing here is emitted at `:root:root`. The active theme must not outrank
 * the others, or setting the attribute would have no effect.
 */
function emitAllThemes(config: SiteConfig): string {
  const overrides = overrideVars(config.overrides);
  const blocks: string[] = [];

  for (const [name, theme] of Object.entries(THEMES)) {
    const self = `[data-zellij-theme="${name}"]`;

    // Structural tokens — fonts, geometry, motion — do not vary by colour mode,
    // so they are emitted once. Repeating them in each mode block roughly
    // trebled the stylesheet, and it is inlined in <head> on every page.
    const base = [...structuralVars(theme), ...paletteVars(theme.light), ...overrides].join('');
    const dark = [...paletteVars(theme.dark), ...overrides].join('');

    blocks.push(`:root${self},${self}{${base}}`);
    blocks.push(
      `:root${self}[data-zellij-mode="dark"],[data-zellij-mode="dark"] ${self}{${dark}}`,
    );
    blocks.push(
      `@media (prefers-color-scheme: dark){` +
        `:root${self}[data-zellij-mode="system"],[data-zellij-mode="system"] ${self}{${dark}}}`,
    );
  }

  return blocks.join('');
}

export { THEMES };
export type { ThemeTokens, Palette };
