import { z } from 'zod';
import { backgroundSchema, baseSectionSchema, registerSection } from './registry';
import { ctaSchema } from '../config/schema';

/**
 * The v1 section vocabulary — spec §5.2. Every entry here gets a schema now and
 * a component in M3. Schemas live apart from components so the content loader
 * (a build-time, server-only module) never pulls React into its graph.
 */

const asset = z.string().min(1);
const cta1or2 = z.array(ctaSchema).max(2);

/**
 * A short text field that authors will reasonably write as a bare number —
 * `date: 2019`, `value: 99.9`. YAML hands those over as numbers, and rejecting
 * them would be pedantry about a quoting rule nobody should have to know.
 */
const text = z
  .union([z.string(), z.number()])
  .transform((value) => String(value))
  .pipe(z.string().min(1));

export const heroSchema = baseSectionSchema.extend({
  type: z.literal('hero'),
  headline: z.string().min(1),
  subhead: z.string().optional(),
  badge: z.string().optional(),
  cta: cta1or2.default([]),
  visual: asset.optional(),
  visualStyle: z.enum(['floating', 'framed', 'full-bleed']).default('floating'),
});

export const logoStripSchema = baseSectionSchema.extend({
  type: z.literal('logo-strip'),
  heading: z.string().optional(),
  logos: z.array(asset).min(1),
});

export const featureGridSchema = baseSectionSchema.extend({
  type: z.literal('feature-grid'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  columns: z.union([z.literal(2), z.literal(3), z.literal(4)]).default(3),
  items: z
    .array(
      z.object({
        icon: z.string().min(1).optional(),
        title: z.string().min(1),
        body: z.string().optional(),
        href: z.string().optional(),
      }),
    )
    .min(1),
});

export const zigzagSchema = baseSectionSchema.extend({
  type: z.literal('zigzag'),
  heading: z.string().optional(),
  rows: z
    .array(
      z.object({
        title: z.string().min(1),
        body: z.string().optional(),
        image: asset.optional(),
        align: z.enum(['image-right', 'image-left']).default('image-right'),
        cta: ctaSchema.optional(),
      }),
    )
    .min(1),
});

export const showcaseSchema = baseSectionSchema.extend({
  type: z.literal('showcase'),
  eyebrow: z.string().optional(),
  headline: z.string().min(1),
  body: z.string().optional(),
  visual: asset.optional(),
  background: backgroundSchema.default('default'),
  cta: ctaSchema.optional(),
});

export const statsBandSchema = baseSectionSchema.extend({
  type: z.literal('stats-band'),
  heading: z.string().optional(),
  stats: z
    .array(
      z.object({
        value: text,
        label: z.string().min(1),
        prefix: z.string().optional(),
        suffix: z.string().optional(),
      }),
    )
    .min(3)
    .max(5),
});

export const testimonialSchema = baseSectionSchema.extend({
  type: z.literal('testimonial'),
  heading: z.string().optional(),
  quotes: z
    .array(
      z.object({
        quote: z.string().min(1),
        author: z.string().min(1),
        role: z.string().optional(),
        avatar: asset.optional(),
      }),
    )
    .min(1)
    .max(5),
});

export const pricingTableSchema = baseSectionSchema.extend({
  type: z.literal('pricing-table'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  tiers: z
    .array(
      z.object({
        name: z.string().min(1),
        price: z.string().min(1),
        period: z.string().optional(),
        description: z.string().optional(),
        features: z.array(z.string()).default([]),
        cta: ctaSchema.optional(),
        highlighted: z.boolean().default(false),
      }),
    )
    .min(1)
    .max(4),
});

export const faqSchema = baseSectionSchema.extend({
  type: z.literal('faq'),
  heading: z.string().optional(),
  items: z
    .array(z.object({ question: z.string().min(1), answer: z.string().min(1) }))
    .min(1),
});

export const ctaBannerSchema = baseSectionSchema.extend({
  type: z.literal('cta-banner'),
  headline: z.string().min(1),
  body: z.string().optional(),
  cta: z.union([ctaSchema, z.array(ctaSchema).max(2)]).optional(),
  background: backgroundSchema.default('tinted'),
});

export const markdownProseSchema = baseSectionSchema.extend({
  type: z.literal('markdown-prose'),
  heading: z.string().optional(),
  /** Inline markdown body. The escape hatch for free-form pages (spec §5.2 #11). */
  body: z.string().min(1),
});

export const videoEmbedSchema = baseSectionSchema.extend({
  type: z.literal('video-embed'),
  heading: z.string().optional(),
  /** Self-hosted mp4 (content-relative) or a YouTube/Vimeo URL. */
  src: z.string().min(1),
  poster: asset.optional(),
  caption: z.string().optional(),
});

/**
 * A rounded full-bleed panel carrying an image or a video, with the copy laid
 * over it. The "take a closer look" building block.
 *
 * Exactly one of `image` or `video` — a panel is one piece of media, and
 * accepting both would leave the precedence to guesswork.
 */
export const mediaPanelSchema = baseSectionSchema
  .extend({
    type: z.literal('media-panel'),
    /** Optional heading *above* the panel, outside it. */
    heading: z.string().optional(),
    /** Copy laid over the media. */
    eyebrow: z.string().optional(),
    title: z.string().optional(),
    body: z.string().optional(),
    image: asset.optional(),
    /** Self-hosted video, played muted and looping; still under reduced motion. */
    video: asset.optional(),
    poster: asset.optional(),
    alt: z.string().default(''),
    /** Where the copy sits over the media. */
    align: z.enum(['left', 'center', 'right']).default('left'),
    /** Which way the copy is keyed, since the engine cannot see the artwork. */
    tone: z.enum(['light', 'dark', 'auto']).default('auto'),
    ratio: z.enum(['ultrawide', 'wide', 'square', 'portrait']).default('wide'),
    /** Darkens the media behind the copy. 0 leaves the artwork untouched. */
    scrim: z.number().min(0).max(1).default(0),
    cta: cta1or2.default([]),
    href: z.string().optional(),
  })
  .refine((section) => [section.image, section.video].filter(Boolean).length === 1, {
    message: 'a media-panel must declare exactly one of: image, video',
  });

/**
 * A horizontally scrolling row of cards. Scroll-snapped, with the next card
 * peeking past the edge so it reads as scrollable before anything moves.
 */
export const cardCarouselSchema = baseSectionSchema.extend({
  type: z.literal('card-carousel'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  /** Trailing link on the heading row, e.g. "Browse everything ›". */
  link: z.object({ label: z.string().min(1), href: z.string().min(1) }).optional(),
  cardWidth: z.enum(['narrow', 'medium', 'wide']).default('medium'),
  items: z
    .array(
      z.object({
        eyebrow: z.string().optional(),
        title: z.string().min(1),
        body: z.string().optional(),
        image: asset.optional(),
        href: z.string().optional(),
        /** Per-card surface, for a row that alternates tone. */
        background: z.enum(['default', 'tinted', 'inverted', 'accent']).optional(),
      }),
    )
    .min(2),
});

/**
 * An asymmetric tile grid. `feature-grid` gives every item the same weight;
 * this one lets a few carry more, which is what makes a page of features read
 * as having a shape rather than a list.
 */
export const bentoGridSchema = baseSectionSchema.extend({
  type: z.literal('bento-grid'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  items: z
    .array(
      z.object({
        title: z.string().min(1),
        body: z.string().optional(),
        icon: z.string().min(1).optional(),
        image: asset.optional(),
        href: z.string().optional(),
        /** How many cells the tile claims on a wide viewport. */
        span: z.enum(['normal', 'wide', 'tall', 'large']).default('normal'),
        background: z.enum(['default', 'tinted', 'inverted', 'accent']).optional(),
      }),
    )
    .min(2),
});

/**
 * A dated sequence: history, milestones, a roadmap, the chapters of a life.
 * Rendered as an ordered list, because that is what it is.
 */
export const timelineSchema = baseSectionSchema.extend({
  type: z.literal('timeline'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  items: z
    .array(
      z.object({
        /** Free text, not a date type: "1984", "Q3", "Chapter one" all belong. */
        date: text,
        title: z.string().min(1),
        body: z.string().optional(),
        image: asset.optional(),
        icon: z.string().min(1).optional(),
      }),
    )
    .min(2),
});

/** People: a board, a faculty, a team, a list of contributors. */
export const teamGridSchema = baseSectionSchema.extend({
  type: z.literal('team-grid'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  columns: z.union([z.literal(2), z.literal(3), z.literal(4)]).default(3),
  people: z
    .array(
      z.object({
        name: z.string().min(1),
        role: z.string().optional(),
        bio: z.string().optional(),
        photo: asset.optional(),
        links: z
          .array(z.object({ label: z.string().min(1), href: z.string().min(1) }))
          .default([]),
      }),
    )
    .min(1),
});

/** A grid of images, each openable full-size. */
export const gallerySchema = baseSectionSchema.extend({
  type: z.literal('gallery'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  columns: z.union([z.literal(2), z.literal(3), z.literal(4)]).default(3),
  /** Opening an image full-size is the point of a gallery; opt out for a plain grid. */
  lightbox: z.boolean().default(true),
  items: z
    .array(
      z.object({
        image: asset,
        alt: z.string().default(''),
        caption: z.string().optional(),
      }),
    )
    .min(2),
});

/** Registration order is the order shown in "valid types" build errors. */
const ALL = [
  { type: 'hero', schema: heroSchema },
  { type: 'logo-strip', schema: logoStripSchema },
  { type: 'feature-grid', schema: featureGridSchema },
  { type: 'zigzag', schema: zigzagSchema },
  { type: 'showcase', schema: showcaseSchema },
  { type: 'stats-band', schema: statsBandSchema },
  { type: 'testimonial', schema: testimonialSchema },
  { type: 'pricing-table', schema: pricingTableSchema },
  { type: 'faq', schema: faqSchema },
  { type: 'cta-banner', schema: ctaBannerSchema },
  { type: 'markdown-prose', schema: markdownProseSchema },
  { type: 'video-embed', schema: videoEmbedSchema },
  { type: 'media-panel', schema: mediaPanelSchema },
  { type: 'card-carousel', schema: cardCarouselSchema },
  { type: 'bento-grid', schema: bentoGridSchema },
  { type: 'timeline', schema: timelineSchema },
  { type: 'team-grid', schema: teamGridSchema },
  { type: 'gallery', schema: gallerySchema },
] as const;

export function registerBuiltinSections(): void {
  for (const def of ALL) registerSection(def);
}

// Built-ins register on import so any consumer of the registry sees them.
registerBuiltinSections();

export type HeroSection = z.infer<typeof heroSchema>;
export type LogoStripSection = z.infer<typeof logoStripSchema>;
export type FeatureGridSection = z.infer<typeof featureGridSchema>;
export type ZigzagSection = z.infer<typeof zigzagSchema>;
export type ShowcaseSection = z.infer<typeof showcaseSchema>;
export type StatsBandSection = z.infer<typeof statsBandSchema>;
export type TestimonialSection = z.infer<typeof testimonialSchema>;
export type PricingTableSection = z.infer<typeof pricingTableSchema>;
export type FaqSection = z.infer<typeof faqSchema>;
export type CtaBannerSection = z.infer<typeof ctaBannerSchema>;
export type MarkdownProseSection = z.infer<typeof markdownProseSchema>;
export type VideoEmbedSection = z.infer<typeof videoEmbedSchema>;
export type MediaPanelSection = z.infer<typeof mediaPanelSchema>;
export type CardCarouselSection = z.infer<typeof cardCarouselSchema>;
export type BentoGridSection = z.infer<typeof bentoGridSchema>;
export type TimelineSection = z.infer<typeof timelineSchema>;
export type TeamGridSection = z.infer<typeof teamGridSchema>;
export type GallerySection = z.infer<typeof gallerySchema>;

export type AnySection =
  | HeroSection
  | LogoStripSection
  | FeatureGridSection
  | ZigzagSection
  | ShowcaseSection
  | StatsBandSection
  | TestimonialSection
  | PricingTableSection
  | FaqSection
  | CtaBannerSection
  | MarkdownProseSection
  | VideoEmbedSection
  | MediaPanelSection
  | CardCarouselSection
  | BentoGridSection
  | TimelineSection
  | TeamGridSection
  | GallerySection;
