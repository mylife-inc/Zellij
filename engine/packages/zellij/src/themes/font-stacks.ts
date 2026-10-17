/**
 * Font stacks are plain strings referencing the CSS variables that
 * fonts/fonts.css defines. They live apart from the stylesheet import so the
 * token layer stays pure — importable, and testable, without a CSS loader.
 */

/**
 * The `cupertino` theme resolves to SF Pro on Apple platforms through the
 * system stack — the real thing, and no download — falling back to Inter
 * elsewhere.
 */
export const SYSTEM_SANS =
  '-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", var(--zellij-font-inter), system-ui, sans-serif';

export const FONT_STACKS = {
  inter: 'var(--zellij-font-inter), system-ui, sans-serif',
  interTight: 'var(--zellij-font-inter-tight), system-ui, sans-serif',
  sourceSerif: 'var(--zellij-font-source-serif), Georgia, serif',
  fraunces: 'var(--zellij-font-fraunces), Georgia, serif',
  spaceGrotesk: 'var(--zellij-font-space-grotesk), system-ui, sans-serif',
  outfit: 'var(--zellij-font-outfit), system-ui, sans-serif',
  plexSans: 'var(--zellij-font-plex-sans), system-ui, sans-serif',
  plexMono: 'var(--zellij-font-plex-mono), ui-monospace, monospace',
  epilogue: 'var(--zellij-font-epilogue), system-ui, sans-serif',
  poppins: 'var(--zellij-font-poppins), system-ui, sans-serif',
  jetbrains: 'var(--zellij-font-jetbrains), ui-monospace, monospace',
  cormorant: 'var(--zellij-font-cormorant), Georgia, serif',
  system: SYSTEM_SANS,
  mono: 'ui-monospace, SFMono-Regular, Menlo, monospace',
} as const;
