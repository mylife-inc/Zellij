import { z } from 'zod';
import { baseSectionSchema, registerSection } from './registry';
import { ctaSchema } from '../config/schema';

/**
 * The v1 section vocabulary — spec §5.2. Every entry here gets a schema now and
 * a component in M3. Schemas live apart from components so the content loader
 * (a build-time, server-only module) never pulls React into its graph.
 */

const asset = z.string().min(1);
const cta1or2 = z.array(ctaSchema).max(2);

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
  background: z.enum(['default', 'tinted', 'inverted']).default('default'),
  cta: ctaSchema.optional(),
});

export const statsBandSchema = baseSectionSchema.extend({
  type: z.literal('stats-band'),
  heading: z.string().optional(),
  stats: z
    .array(
      z.object({
        value: z.string().min(1),
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
  background: z.enum(['default', 'tinted', 'inverted']).default('tinted'),
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
  | VideoEmbedSection;
