import './fonts/fonts.css';

/**
 * Pinned local WOFF2 files keep builds independent of Google Fonts. Unicode
 * ranges, weights and metric-adjusted fallbacks are preserved in the CSS.
 * No font is preloaded: the browser downloads only families/glyphs in use.
 * See fonts/README.md for provenance and updating the checked-in snapshot.
 */
export const fontVariables = 'zellij-fonts';

export { FONT_STACKS, SYSTEM_SANS } from './font-stacks';
