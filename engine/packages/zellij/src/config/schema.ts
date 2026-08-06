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

/**
 * The logo and site name at the left of the bar — design §4.1.
 *
 * `logo.href` is what lets a family of sites share one owner: point it at the
 * hub and the mark leaves for the hub, while `title` becomes the way home
 * within this site. Omitted, the logo links to "/" and `title` is unnecessary.
 */
export const brandSchema = z.object({
  logo: z
    .object({
      src: z.string().min(1),
      dark: z.string().optional(),
      alt: z.string().optional(),
      href: z.string().optional(),
      /**
       * How tall the mark sits in the bar, as a CSS length.
       *
       * The default suits a wordmark, whose cap height *is* its box: matched to
       * the nav label size it reads far larger than the labels around it. A
       * square or round mark is the opposite — the same box height leaves a
       * thumbnail beside text it should be anchoring, since its ink is spread
       * over two dimensions instead of one.
       *
       * There is no way to infer this from the file. A tall narrow crest and a
       * wide wordmark can share a bounding box and want different heights, so
       * the site says.
       */
      height: z
        .string()
        .regex(
          /^\d+(\.\d+)?(rem|em|px)$/,
          'must be a CSS length in rem, em or px — for example "1.75rem"',
        )
        .optional(),
    })
    .optional(),
  title: z.string().optional(),
});
export type BrandConfig = z.infer<typeof brandSchema>;

/**
 * How the bar behaves and which built-in controls it carries. What is *in* the
 * bar is `menu.yaml`'s business (design §5); this is everything else.
 */
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
  cta: z.array(ctaSchema).default([]),
  social: socialSchema.optional(),
  announcement: announcementSchema.optional(),
});
export type NavConfig = z.infer<typeof navSchema>;

/** A link that names either an internal page or a literal href. */
export const linkTargetSchema = z
  .object({
    label: z.string().min(1),
    href: z.string().min(1).optional(),
    page: z.string().min(1).optional(),
  })
  .refine((link) => [link.href, link.page].filter(Boolean).length === 1, {
    message: 'a link must declare exactly one of: href, page',
  });

export const footerSchema = z.object({
  columns: z
    .array(
      z.object({
        heading: z.string().min(1),
        links: z.array(linkTargetSchema).min(1),
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

/** Token overrides let a site tweak a theme without forking it (spec §6). */
export const overridesSchema = z
  .object({
    accent: z.string().optional(),
    accentForeground: z.string().optional(),
    radius: z.enum(['none', 'sm', 'md', 'lg', 'xl']).optional(),
    fontHeading: z.string().optional(),
    fontBody: z.string().optional(),
  })
  .strip();

/**
 * `site.yaml` — the content root's own identity (design §4).
 *
 * Every key but `name` is optional, and the file itself is optional: a folder
 * containing nothing but `_sidebar.yaml` still builds. It lives inside the
 * content root rather than beside the wrapper app so that pointing a build at a
 * folder — including one just cloned from a repo — is enough on its own.
 */
export const siteConfigSchema = z.object({
  name: z.string().min(1),
  tagline: z.string().optional(),
  brand: brandSchema.prefault({}),

  theme: themeNameSchema.default('paper'),
  mode: z.enum(['light', 'dark', 'system']).default('system'),
  motion: motionPresetSchema.default('subtle'),
  overrides: overridesSchema.default({}),

  /** Page or guide folder shown at "/". Defaults to the first menu item (design §10). */
  home: z.string().min(1).optional(),
  /** Menu items past this count collapse into one trailing drawer (design §10). */
  menuOverflowAfter: z.number().int().min(1).default(7),

  /** Mirrors next.config basePath so assets and links stay prefix-correct (spec §13.4). */
  basePath: z.string().default(''),
  /** Absolute site origin; required for absolute OG and canonical URLs. */
  siteUrl: z.url().optional(),
  /** Content-relative social share image, e.g. assets/og.png. */
  ogImage: z.string().optional(),

  // `prefault` (not `default`) so an omitted block is parsed through its schema
  // and picks up all inner field defaults, rather than needing a full output value.
  nav: navSchema.prefault({}),
  footer: footerSchema.prefault({}),
  analytics: analyticsSchema.prefault({}),
});

export type SiteConfig = z.infer<typeof siteConfigSchema>;

/**
 * Unknown top-level keys are a warning, not an error, so a newer content folder
 * can be read by an older engine (spec §10 forward-compatibility).
 */
export const KNOWN_SITE_KEYS: readonly string[] = Object.keys(siteConfigSchema.shape);
