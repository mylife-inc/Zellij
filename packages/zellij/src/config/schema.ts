import { z } from 'zod';

/**
 * The built-in themes ("glazes") — spec §6. Ten from the original inventory,
 * plus four drawn from design specs: `cupertino`, `studio`, `iris`, and
 * `meridian`. The latter three carry neutral names for the same reason
 * `cupertino` does — they capture a design language, not a brand.
 */
export const THEME_NAMES = [
  'midnight',
  'paper',
  'terracotta',
  'mono',
  'aurora',
  'slate',
  'forest',
  'sunburst',
  'carbon',
  'porcelain',
  'cupertino',
  'studio',
  'iris',
  'meridian',
] as const;

export const themeNameSchema = z.enum(THEME_NAMES);
export type ThemeName = (typeof THEME_NAMES)[number];

/** Motion presets — spec §6.5. */
export const MOTION_PRESETS = ['none', 'subtle', 'expressive'] as const;
export const motionPresetSchema = z.enum(MOTION_PRESETS);
export type MotionPreset = (typeof MOTION_PRESETS)[number];

/** Page layouts — spec §5.4. A closed set, deliberately. */
export const PAGE_LAYOUTS = ['stack', 'stack-narrow', 'stack-sidebar', 'landing'] as const;
export const pageLayoutSchema = z.enum(PAGE_LAYOUTS);
export type PageLayout = (typeof PAGE_LAYOUTS)[number];

export const ctaStyleSchema = z.enum(['primary', 'secondary', 'ghost']);

export const ctaSchema = z.object({
  label: z.string().min(1),
  href: z.string().min(1),
  style: ctaStyleSchema.default('primary'),
});
export type Cta = z.infer<typeof ctaSchema>;

const navLeafSchema = z.object({
  label: z.string().min(1),
  href: z.string().min(1),
  description: z.string().optional(),
  icon: z.string().optional(),
});

const megaGroupSchema = z.object({
  heading: z.string().min(1),
  items: z.array(navLeafSchema).min(1),
});

/**
 * A nav item is one of: a plain link, a dropdown (children), or a mega panel.
 * Enforced with a refine rather than a union so error messages stay readable.
 */
export const navItemSchema = z
  .object({
    label: z.string().min(1),
    href: z.string().min(1).optional(),
    children: z.array(navLeafSchema).min(1).optional(),
    mega: z.object({ groups: z.array(megaGroupSchema).min(1) }).optional(),
  })
  .refine(
    (item) => [item.href, item.children, item.mega].filter(Boolean).length === 1,
    { message: 'a nav item must declare exactly one of: href, children, or mega' },
  );
export type NavItem = z.infer<typeof navItemSchema>;

export const announcementSchema = z.object({
  text: z.string().min(1),
  link: z.object({ label: z.string().min(1), href: z.string().min(1) }).optional(),
  dismissible: z.boolean().default(true),
});

export const socialSchema = z.object({
  github: z.url().optional(),
  x: z.url().optional(),
  linkedin: z.url().optional(),
  youtube: z.url().optional(),
  discord: z.url().optional(),
});
export type SocialConfig = z.infer<typeof socialSchema>;

export const navSchema = z.object({
  sticky: z.boolean().default(true),
  align: z.enum(['left', 'center']).default('left'),
  hideOnScrollDown: z.boolean().default(false),
  transparentOverHero: z.boolean().default(true),
  scrollThreshold: z.number().int().nonnegative().default(80),
  themeToggle: z.boolean().default(false),
  /**
   * Adds a theme picker to the bar. Opt-in because it changes what ships: the
   * engine normally emits only the active theme's tokens, and a runtime
   * switcher needs every theme's tokens present in the page.
   */
  themeSwitcher: z.boolean().default(false),
  search: z.boolean().default(false),
  links: z.array(navItemSchema).default([]),
  cta: z.array(ctaSchema).default([]),
  social: socialSchema.optional(),
  announcement: announcementSchema.optional(),
});
export type NavConfig = z.infer<typeof navSchema>;

export const footerSchema = z.object({
  columns: z
    .array(
      z.object({
        heading: z.string().min(1),
        links: z.array(z.object({ label: z.string().min(1), href: z.string().min(1) })).min(1),
      }),
    )
    .default([]),
  legal: z.string().optional(),
  social: socialSchema.optional(),
});
export type FooterConfig = z.infer<typeof footerSchema>;

export const analyticsSchema = z.object({
  provider: z.enum(['plausible', 'none']).default('none'),
  domain: z.string().optional(),
});

/** Token overrides let a product tweak a theme without forking it (spec §6). */
export const overridesSchema = z
  .object({
    accent: z.string().optional(),
    accentForeground: z.string().optional(),
    radius: z.enum(['none', 'sm', 'md', 'lg', 'xl']).optional(),
    fontHeading: z.string().optional(),
    fontBody: z.string().optional(),
  })
  .strip();

export const zellijConfigSchema = z.object({
  product: z.object({
    name: z.string().min(1),
    tagline: z.string().optional(),
    logo: z.string().optional(),
    logoDark: z.string().optional(),
  }),
  /** Path to the content directory, relative to the wrapper app root. */
  content: z.string().min(1).default('./content'),
  /** Mirrors next.config basePath so asset()/links stay prefix-correct (spec §13.4). */
  basePath: z.string().default(''),
  /** Absolute site origin; required for absolute OG and canonical URLs. */
  siteUrl: z.url().optional(),
  /** Content-relative social share image, e.g. assets/og.png. */
  ogImage: z.string().optional(),
  theme: themeNameSchema.default('paper'),
  mode: z.enum(['light', 'dark', 'system']).default('system'),
  motion: motionPresetSchema.default('subtle'),
  overrides: overridesSchema.default({}),
  // `prefault` (not `default`) so an omitted block is parsed through its schema
  // and picks up all inner field defaults, rather than needing a full output value.
  nav: navSchema.prefault({}),
  footer: footerSchema.prefault({}),
  analytics: analyticsSchema.prefault({}),
});

/**
 * Unknown top-level keys are a warning, not an error, so a newer config can be
 * read by an older engine (spec §10 forward-compatibility).
 */
export type ZellijConfig = z.infer<typeof zellijConfigSchema>;

export const KNOWN_CONFIG_KEYS: readonly string[] = Object.keys(zellijConfigSchema.shape);
