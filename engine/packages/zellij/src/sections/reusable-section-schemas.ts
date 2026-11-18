// Standalone schemas shared by the bounded composition adapter. No registry imports
// from schemas.ts: keeping these definitions independent avoids a recursive schema.
import { z } from 'zod';
import { baseSectionSchema, backgroundSchema } from './registry';
import { ctaSchema } from '../config/schema';
import { assetReferenceSchema, hrefReferenceSchema, inlineReferenceSchema } from '../content/reference-schema';
const asset = assetReferenceSchema.min(1);

/**
 * A short text field that authors will reasonably write as a bare number —
 * `date: 2019`, `value: 99.9`. YAML hands those over as numbers, and rejecting
 * them would be pedantry about a quoting rule nobody should have to know.
 */
const text = z
  .union([z.string(), z.number()])
  .transform((value) => String(value))
  .pipe(z.string().min(1));


export const logoStripSchema = baseSectionSchema.extend({
  type: z.literal('logo-strip'),
  heading: z.string().optional(),
  logos: z.array(asset).min(1),
});

export const featureGridSchema = baseSectionSchema.extend({
  type: z.literal('feature-grid'),
  eyebrow: z.string().optional(),
  heading: z.string().optional(),
  subhead: inlineReferenceSchema.optional(),
  columns: z.union([z.literal(2), z.literal(3), z.literal(4)]).default(3),
  items: z
    .array(
      z.object({
        icon: z.string().min(1).optional(),
        title: z.string().min(1),
        body: inlineReferenceSchema.optional(),
        href: hrefReferenceSchema.optional(),
      }),
    )
    .min(1),
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

export const pricingTableSchema = baseSectionSchema.extend({
  type: z.literal('pricing-table'),
  heading: z.string().optional(),
  subhead: inlineReferenceSchema.optional(),
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

export const ctaBannerSchema = baseSectionSchema.extend({
  type: z.literal('cta-banner'),
  headline: z.string().min(1),
  body: inlineReferenceSchema.optional(),

  /**
   * One line somebody is about to paste into a terminal, with a copy button.
   *
   * On the banner rather than in a section of its own: an install line is a
   * call to action that happens to be executable, and it belongs beside the
   * buttons offering the same thing by other routes. A `markdown-prose` code
   * fence would put it in a different band with different padding and no
   * relationship to the CTAs.
   *
   * Deliberately a single line and not a transcript — `terminal` on
   * `hero-split` and `steps-panel` is for showing what a session looks like.
   * This is for something to be run.
   */
  command: z.string().min(1).optional(),

  cta: z.union([ctaSchema, z.array(ctaSchema).max(2)]).optional(),
  background: backgroundSchema.default('tinted'),
});

/**
 * An asymmetric tile grid. `feature-grid` gives every item the same weight;
 * this one lets a few carry more, which is what makes a page of features read
 * as having a shape rather than a list.
 */
export const bentoGridSchema = baseSectionSchema.extend({
  type: z.literal('bento-grid'),
  heading: z.string().optional(),
  subhead: inlineReferenceSchema.optional(),
  items: z
    .array(
      z.object({
        title: z.string().min(1),
        body: inlineReferenceSchema.optional(),
        icon: z.string().min(1).optional(),
        image: asset.optional(),
        href: hrefReferenceSchema.optional(),
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
  subhead: inlineReferenceSchema.optional(),
  items: z
    .array(
      z.object({
        /** Free text, not a date type: "1984", "Q3", "Chapter one" all belong. */
        date: text,
        title: z.string().min(1),
        body: inlineReferenceSchema.optional(),
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
  subhead: inlineReferenceSchema.optional(),
  columns: z.union([z.literal(2), z.literal(3), z.literal(4)]).default(3),
  people: z
    .array(
      z.object({
        name: z.string().min(1),
        role: z.string().optional(),
        bio: inlineReferenceSchema.optional(),
        photo: asset.optional(),
        links: z
          .array(z.object({ label: z.string().min(1), href: hrefReferenceSchema.min(1) }))
          .default([]),
      }),
    )
    .min(1),
});
