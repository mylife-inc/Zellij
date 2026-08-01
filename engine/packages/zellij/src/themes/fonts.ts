import {
  Cormorant_Garamond,
  Epilogue,
  Fraunces,
  IBM_Plex_Sans,
  IBM_Plex_Mono,
  Inter,
  Inter_Tight,
  JetBrains_Mono,
  Outfit,
  Poppins,
  Source_Serif_4,
  Space_Grotesk,
} from 'next/font/google';

/**
 * Theme typography (spec §6).
 *
 * Every family is declared once here and exposed as a CSS custom property.
 * `preload: false` is deliberate: all themes' @font-face rules are present in
 * the stylesheet, but a browser only downloads the families the active theme
 * actually references — so shipping many themes costs bytes of CSS rather than
 * font downloads, and the Lighthouse budget (§11.6) is unaffected.
 *
 * Each call is written out in full because next/font is a compile-time macro:
 * its options must be object literals, so a shared spread will not build.
 */

const inter = Inter({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-inter',
});

const interTight = Inter_Tight({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-inter-tight',
});

const sourceSerif = Source_Serif_4({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-source-serif',
});

const fraunces = Fraunces({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-fraunces',
});

const spaceGrotesk = Space_Grotesk({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-space-grotesk',
});

const outfit = Outfit({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-outfit',
});

const plexSans = IBM_Plex_Sans({
  subsets: ['latin'],
  weight: ['400', '500', '600', '700'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-plex-sans',
});

const plexMono = IBM_Plex_Mono({
  subsets: ['latin'],
  weight: ['400', '500', '600'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-plex-mono',
});

const epilogue = Epilogue({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-epilogue',
});

const poppins = Poppins({
  subsets: ['latin'],
  weight: ['400', '500', '600', '700'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-poppins',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-jetbrains',
});

const cormorant = Cormorant_Garamond({
  subsets: ['latin'],
  weight: ['300', '400', '500', '600', '700'],
  display: 'swap',
  preload: false,
  variable: '--zellij-font-cormorant',
});

/**
 * Applied to <html> so every family's CSS variable is defined; themes pick
 * which ones they use.
 */
export const fontVariables = [
  inter,
  interTight,
  sourceSerif,
  fraunces,
  spaceGrotesk,
  outfit,
  plexSans,
  plexMono,
  epilogue,
  poppins,
  jetbrainsMono,
  cormorant,
]
  .map((font) => font.variable)
  .join(' ');

export { FONT_STACKS, SYSTEM_SANS } from './font-stacks';
