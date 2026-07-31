import { FONT_STACKS } from './font-stacks';
import type { ThemeName } from '../config/schema';

export interface Palette {
  bg: string;
  fg: string;
  muted: string;
  surface: string;
  border: string;
  accent: string;
  accentFg: string;
  invertedBg: string;
  invertedFg: string;
}

export interface ThemeTokens {
  label: string;
  /** One-line personality summary, used in the README and theme gallery. */
  description: string;
  fonts: { heading: string; body: string; mono: string };
  geometry: {
    /** Base corner radius for cards and panels. */
    radius: string;
    /** Buttons and pills — Apple-style themes go fully round. */
    radiusButton: string;
    /** Imagery and large surfaces. */
    radiusMedia: string;
    container: string;
    /** Vertical rhythm between sections. */
    sectionGap: string;
    navHeight: string;
  };
  type: {
    /** Headline letter-spacing: tight geometry vs. airy editorial. */
    tracking: string;
    headingWeight: string;
    /** Multiplier on the hero/section headline scale. */
    displayScale: string;
  };
  shadow: {
    card: string;
    float: string;
  };
  motion: {
    duration: string;
    ease: string;
    distance: string;
    stagger: string;
  };
  light: Palette;
  dark: Palette;
}

const NEUTRAL_INVERT = { invertedBg: '#14161a', invertedFg: '#f7f8fa' };

export const THEMES: Record<ThemeName, ThemeTokens> = {
  /* ------------------------------------------------------------- midnight */
  midnight: {
    label: 'Midnight',
    description: 'Dark, electric accent, tight geometry — a product-tool aesthetic.',
    fonts: { heading: FONT_STACKS.interTight, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '0.5rem',
      radiusButton: '0.5rem',
      radiusMedia: '0.75rem',
      container: '72rem',
      sectionGap: '6rem',
      navHeight: '3.75rem',
    },
    type: { tracking: '-0.03em', headingWeight: '650', displayScale: '1' },
    shadow: {
      card: '0 1px 2px rgb(0 0 0 / 0.18)',
      float: '0 24px 60px -24px rgb(0 0 0 / 0.7)',
    },
    motion: {
      duration: '420ms',
      ease: 'cubic-bezier(0.22, 1, 0.36, 1)',
      distance: '14px',
      stagger: '55ms',
    },
    light: {
      bg: '#ffffff',
      fg: '#14161f',
      muted: '#5b6072',
      surface: '#f6f7fb',
      border: '#e4e6ef',
      accent: '#5b5bd6',
      accentFg: '#ffffff',
      ...NEUTRAL_INVERT,
    },
    dark: {
      bg: '#0c0d12',
      fg: '#e9eaf2',
      muted: '#8d93a8',
      surface: '#14161f',
      border: '#23262f',
      accent: '#7c7cf0',
      accentFg: '#0c0d12',
      invertedBg: '#1a1d29',
      invertedFg: '#e9eaf2',
    },
  },

  /* ---------------------------------------------------------------- paper */
  paper: {
    label: 'Paper',
    description: 'Light, warm, editorial serif headings over a cream page.',
    fonts: { heading: FONT_STACKS.sourceSerif, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '0.5rem',
      radiusButton: '0.375rem',
      radiusMedia: '0.5rem',
      container: '70rem',
      sectionGap: '6.5rem',
      navHeight: '4rem',
    },
    type: { tracking: '-0.015em', headingWeight: '600', displayScale: '0.97' },
    shadow: {
      card: '0 1px 2px rgb(28 25 23 / 0.06)',
      float: '0 24px 50px -26px rgb(28 25 23 / 0.32)',
    },
    motion: {
      duration: '520ms',
      ease: 'cubic-bezier(0.16, 1, 0.3, 1)',
      distance: '16px',
      stagger: '70ms',
    },
    light: {
      bg: '#fdfcfa',
      fg: '#1c1917',
      muted: '#6b6560',
      surface: '#f5f3ef',
      border: '#e7e2da',
      accent: '#9a3412',
      accentFg: '#ffffff',
      invertedBg: '#1c1917',
      invertedFg: '#faf8f5',
    },
    dark: {
      bg: '#171513',
      fg: '#f2efe9',
      muted: '#a8a099',
      surface: '#211e1b',
      border: '#302b26',
      accent: '#e07a4d',
      accentFg: '#1c1917',
      invertedBg: '#2a2622',
      invertedFg: '#f2efe9',
    },
  },

  /* ----------------------------------------------------------- terracotta */
  terracotta: {
    label: 'Terracotta',
    description: 'Warm Moroccan palette with generous curves — the Shebka house style.',
    fonts: { heading: FONT_STACKS.fraunces, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '1rem',
      radiusButton: '0.875rem',
      radiusMedia: '1.25rem',
      container: '72rem',
      sectionGap: '7rem',
      navHeight: '4.25rem',
    },
    type: { tracking: '-0.02em', headingWeight: '600', displayScale: '1' },
    shadow: {
      card: '0 2px 4px rgb(120 53 15 / 0.06)',
      float: '0 28px 60px -28px rgb(120 53 15 / 0.4)',
    },
    motion: {
      duration: '600ms',
      ease: 'cubic-bezier(0.34, 1.2, 0.44, 1)',
      distance: '20px',
      stagger: '85ms',
    },
    light: {
      bg: '#fff8f3',
      fg: '#2a1d16',
      muted: '#7a6154',
      surface: '#fdefe4',
      border: '#f0dccb',
      accent: '#e86a33',
      accentFg: '#ffffff',
      invertedBg: '#2a1d16',
      invertedFg: '#fdefe4',
    },
    dark: {
      bg: '#1a120d',
      fg: '#f7e9df',
      muted: '#b39684',
      surface: '#241811',
      border: '#3a281d',
      accent: '#f2854f',
      accentFg: '#1a120d',
      invertedBg: '#2f2117',
      invertedFg: '#f7e9df',
    },
  },

  /* ----------------------------------------------------------------- mono */
  mono: {
    label: 'Mono',
    description: 'Austere black and white with a single hot accent and brutalist edges.',
    fonts: { heading: FONT_STACKS.spaceGrotesk, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '0',
      radiusButton: '0',
      radiusMedia: '0',
      container: '74rem',
      sectionGap: '5.5rem',
      navHeight: '3.75rem',
    },
    type: { tracking: '-0.04em', headingWeight: '700', displayScale: '1.04' },
    shadow: {
      // Hard offset, no blur — the brutalist signature.
      card: '4px 4px 0 0 #000000',
      float: '10px 10px 0 0 #000000',
    },
    motion: {
      duration: '220ms',
      ease: 'cubic-bezier(0.2, 0, 0, 1)',
      distance: '10px',
      stagger: '35ms',
    },
    light: {
      bg: '#ffffff',
      fg: '#000000',
      muted: '#525252',
      surface: '#f4f4f4',
      border: '#000000',
      accent: '#ff3b00',
      accentFg: '#ffffff',
      invertedBg: '#000000',
      invertedFg: '#ffffff',
    },
    dark: {
      bg: '#000000',
      fg: '#ffffff',
      muted: '#a3a3a3',
      surface: '#111111',
      border: '#ffffff',
      accent: '#ff5c2b',
      accentFg: '#000000',
      invertedBg: '#ffffff',
      invertedFg: '#000000',
    },
  },

  /* --------------------------------------------------------------- aurora */
  aurora: {
    label: 'Aurora',
    description: 'Dark with soft gradient glows and glassy surfaces.',
    fonts: { heading: FONT_STACKS.outfit, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '1.25rem',
      radiusButton: '999px',
      radiusMedia: '1.5rem',
      container: '74rem',
      sectionGap: '7.5rem',
      navHeight: '4.25rem',
    },
    type: { tracking: '-0.025em', headingWeight: '600', displayScale: '1.02' },
    shadow: {
      card: '0 1px 0 rgb(255 255 255 / 0.06) inset, 0 2px 10px rgb(0 0 0 / 0.35)',
      float: '0 40px 90px -30px rgb(56 189 248 / 0.35)',
    },
    motion: {
      duration: '760ms',
      ease: 'cubic-bezier(0.16, 1, 0.3, 1)',
      distance: '26px',
      stagger: '110ms',
    },
    light: {
      bg: '#f7f9ff',
      fg: '#111a2e',
      muted: '#5a6683',
      surface: '#ecf1ff',
      border: '#dce4f7',
      accent: '#3b82f6',
      accentFg: '#ffffff',
      invertedBg: '#0b1020',
      invertedFg: '#e8ecf8',
    },
    dark: {
      bg: '#0b1020',
      fg: '#e8ecf8',
      muted: '#95a1c4',
      surface: '#141b31',
      border: '#232c4a',
      accent: '#7dd3fc',
      accentFg: '#0b1020',
      invertedBg: '#141b31',
      invertedFg: '#e8ecf8',
    },
  },

  /* ---------------------------------------------------------------- slate */
  slate: {
    label: 'Slate',
    description: 'Cool corporate blue-grey — enterprise-safe and unfussy.',
    fonts: { heading: FONT_STACKS.plexSans, body: FONT_STACKS.plexSans, mono: FONT_STACKS.plexMono },
    geometry: {
      radius: '0.375rem',
      radiusButton: '0.375rem',
      radiusMedia: '0.5rem',
      container: '72rem',
      sectionGap: '5.5rem',
      navHeight: '4rem',
    },
    type: { tracking: '-0.015em', headingWeight: '600', displayScale: '0.95' },
    shadow: {
      card: '0 1px 2px rgb(15 23 42 / 0.08)',
      float: '0 20px 44px -24px rgb(15 23 42 / 0.35)',
    },
    motion: {
      duration: '400ms',
      ease: 'cubic-bezier(0.4, 0, 0.2, 1)',
      distance: '12px',
      stagger: '50ms',
    },
    light: {
      bg: '#ffffff',
      fg: '#0f172a',
      muted: '#64748b',
      surface: '#f8fafc',
      border: '#e2e8f0',
      accent: '#2563eb',
      accentFg: '#ffffff',
      invertedBg: '#0f172a',
      invertedFg: '#f1f5f9',
    },
    dark: {
      bg: '#0b1220',
      fg: '#e6edf7',
      muted: '#94a3b8',
      surface: '#131c2e',
      border: '#22304a',
      accent: '#60a5fa',
      accentFg: '#0b1220',
      invertedBg: '#131c2e',
      invertedFg: '#e6edf7',
    },
  },

  /* --------------------------------------------------------------- forest */
  forest: {
    label: 'Forest',
    description: 'Deep greens, natural and calm.',
    fonts: { heading: FONT_STACKS.epilogue, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '0.875rem',
      radiusButton: '0.75rem',
      radiusMedia: '1rem',
      container: '70rem',
      sectionGap: '6.5rem',
      navHeight: '4rem',
    },
    type: { tracking: '-0.025em', headingWeight: '650', displayScale: '0.98' },
    shadow: {
      card: '0 1px 3px rgb(20 38 28 / 0.07)',
      float: '0 26px 54px -28px rgb(20 38 28 / 0.4)',
    },
    motion: {
      duration: '580ms',
      ease: 'cubic-bezier(0.22, 1, 0.36, 1)',
      distance: '18px',
      stagger: '75ms',
    },
    light: {
      bg: '#fbfdfb',
      fg: '#14261c',
      muted: '#55705f',
      surface: '#eff5f0',
      border: '#dce8df',
      accent: '#157f52',
      accentFg: '#ffffff',
      invertedBg: '#14261c',
      invertedFg: '#eff5f0',
    },
    dark: {
      bg: '#0b1610',
      fg: '#e6f2ea',
      muted: '#8fae9c',
      surface: '#111f18',
      border: '#1e3226',
      accent: '#34d399',
      accentFg: '#0b1610',
      invertedBg: '#111f18',
      invertedFg: '#e6f2ea',
    },
  },

  /* ------------------------------------------------------------- sunburst */
  sunburst: {
    label: 'Sunburst',
    description: 'Light and high-energy, playful radius and a hot warm accent.',
    fonts: { heading: FONT_STACKS.poppins, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '1.5rem',
      radiusButton: '999px',
      radiusMedia: '1.75rem',
      container: '72rem',
      sectionGap: '6.5rem',
      navHeight: '4.25rem',
    },
    type: { tracking: '-0.03em', headingWeight: '700', displayScale: '1.02' },
    shadow: {
      card: '0 2px 6px rgb(180 83 9 / 0.08)',
      float: '0 30px 60px -28px rgb(249 115 22 / 0.5)',
    },
    motion: {
      duration: '480ms',
      ease: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      distance: '22px',
      stagger: '65ms',
    },
    light: {
      bg: '#fffdf7',
      fg: '#2b1b00',
      muted: '#7d6440',
      surface: '#fff4e0',
      border: '#ffe2b8',
      accent: '#f97316',
      accentFg: '#ffffff',
      invertedBg: '#2b1b00',
      invertedFg: '#fff4e0',
    },
    dark: {
      bg: '#1a1206',
      fg: '#fdf3e3',
      muted: '#b99b6f',
      surface: '#241a0c',
      border: '#3b2b14',
      accent: '#fb923c',
      accentFg: '#1a1206',
      invertedBg: '#241a0c',
      invertedFg: '#fdf3e3',
    },
  },

  /* --------------------------------------------------------------- carbon */
  carbon: {
    label: 'Carbon',
    description: 'Near-black developer-tool aesthetic with monospace-flavoured headings.',
    fonts: { heading: FONT_STACKS.jetbrains, body: FONT_STACKS.inter, mono: FONT_STACKS.jetbrains },
    geometry: {
      radius: '0.25rem',
      radiusButton: '0.25rem',
      radiusMedia: '0.375rem',
      container: '70rem',
      sectionGap: '5rem',
      navHeight: '3.5rem',
    },
    type: { tracking: '-0.02em', headingWeight: '600', displayScale: '0.9' },
    shadow: {
      card: '0 0 0 1px rgb(255 255 255 / 0.04)',
      float: '0 20px 50px -24px rgb(0 0 0 / 0.8)',
    },
    motion: {
      duration: '260ms',
      ease: 'cubic-bezier(0.3, 0, 0.2, 1)',
      distance: '8px',
      stagger: '40ms',
    },
    light: {
      bg: '#fafafa',
      fg: '#0a0a0a',
      muted: '#525252',
      surface: '#f0f0f0',
      border: '#dcdcdc',
      accent: '#0f766e',
      accentFg: '#ffffff',
      invertedBg: '#0a0a0a',
      invertedFg: '#ededed',
    },
    dark: {
      bg: '#0a0a0a',
      fg: '#ededed',
      muted: '#8a8a8a',
      surface: '#141414',
      border: '#262626',
      accent: '#4ade80',
      accentFg: '#0a0a0a',
      invertedBg: '#141414',
      invertedFg: '#ededed',
    },
  },

  /* ------------------------------------------------------------ porcelain */
  porcelain: {
    label: 'Porcelain',
    description: 'Ultra-minimal light, hairline borders, quiet luxury.',
    fonts: { heading: FONT_STACKS.cormorant, body: FONT_STACKS.inter, mono: FONT_STACKS.mono },
    geometry: {
      radius: '0.125rem',
      radiusButton: '0.125rem',
      radiusMedia: '0.25rem',
      container: '68rem',
      sectionGap: '8rem',
      navHeight: '4.5rem',
    },
    type: { tracking: '-0.01em', headingWeight: '500', displayScale: '1.15' },
    shadow: {
      card: 'none',
      float: '0 30px 60px -40px rgb(0 0 0 / 0.28)',
    },
    motion: {
      duration: '700ms',
      ease: 'cubic-bezier(0.16, 1, 0.3, 1)',
      distance: '12px',
      stagger: '95ms',
    },
    light: {
      bg: '#ffffff',
      fg: '#111111',
      muted: '#8a8a8a',
      surface: '#fafafa',
      border: '#ececec',
      accent: '#8a7b5c',
      accentFg: '#ffffff',
      invertedBg: '#111111',
      invertedFg: '#fafafa',
    },
    dark: {
      bg: '#0d0d0d',
      fg: '#f2f2f2',
      muted: '#9a9a9a',
      surface: '#141414',
      border: '#232323',
      accent: '#c4b58f',
      accentFg: '#0d0d0d',
      invertedBg: '#1a1a1a',
      invertedFg: '#f2f2f2',
    },
  },

  /* ---------------------------------------------------------------- apple */
  apple: {
    label: 'Apple',
    description:
      'Very large tight headlines, centred compositions, and huge breathing room — the Apple product-page cadence.',
    fonts: { heading: FONT_STACKS.system, body: FONT_STACKS.system, mono: FONT_STACKS.mono },
    geometry: {
      radius: '1.125rem',
      // Apple's buttons are fully rounded pills.
      radiusButton: '980px',
      radiusMedia: '1.375rem',
      container: '69rem',
      // The signature: far more vertical room than a typical SaaS page.
      sectionGap: '8.5rem',
      navHeight: '3rem',
    },
    type: { tracking: '-0.035em', headingWeight: '600', displayScale: '1.18' },
    shadow: {
      card: '0 4px 14px rgb(0 0 0 / 0.06)',
      float: '0 30px 70px -30px rgb(0 0 0 / 0.35)',
    },
    motion: {
      duration: '650ms',
      // Apple's easing is slow-out, very settled.
      ease: 'cubic-bezier(0.28, 0.11, 0.32, 1)',
      distance: '24px',
      stagger: '90ms',
    },
    light: {
      bg: '#ffffff',
      fg: '#1d1d1f',
      muted: '#6e6e73',
      surface: '#f5f5f7',
      border: '#d2d2d7',
      accent: '#0071e3',
      accentFg: '#ffffff',
      invertedBg: '#000000',
      invertedFg: '#f5f5f7',
    },
    dark: {
      bg: '#000000',
      fg: '#f5f5f7',
      muted: '#86868b',
      surface: '#1d1d1f',
      border: '#424245',
      accent: '#2997ff',
      accentFg: '#ffffff',
      invertedBg: '#1d1d1f',
      invertedFg: '#f5f5f7',
    },
  },
};
