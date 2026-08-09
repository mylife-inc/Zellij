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

/*
 * Both heroes default to `glow` — the only sections that do.
 *
 * A glow works by being the one lit thing on the page. Default it everywhere
 * and pools stack down the scroll until none of them reads as a source, which
 * is the same reason a page with everything emphasised has nothing emphasised.
 * The opening section is where the effect is worth spending, so that is where
 * it is spent.
 *
 * Still a default and not a fixture: `background: default` in the YAML turns it
 * off, and any other named ground replaces it.
 */
export const heroSchema = baseSectionSchema.extend({
  type: z.literal('hero'),
  background: backgroundSchema.default('glow'),
  headline: z.string().min(1),
  subhead: z.string().optional(),
  badge: z.string().optional(),
  cta: cta1or2.default([]),
  visual: asset.optional(),
  visualStyle: z.enum(['floating', 'framed', 'full-bleed']).default('floating'),
});

/**
 * A hero with the copy on one side and something to look at on the other.
 *
 * Its own type rather than a `visualStyle` on `hero`, because it is not the
 * same section arranged differently. It carries things a centred hero has no
 * place for — a row of qualifiers under the buttons, a terminal transcript
 * instead of a picture — and a variant would have meant one schema where half
 * the fields are inert depending on another field's value, which is the shape
 * that stops a section being able to change later.
 *
 * `hero` stays the right choice when the headline is the whole message.
 */
export const heroSplitSchema = baseSectionSchema.extend({
  type: z.literal('hero-split'),
  background: backgroundSchema.default('glow'),
  headline: z.string().min(1),
  subhead: z.string().optional(),
  badge: z.string().optional(),
  badgeIcon: z.string().min(1).optional(),
  cta: cta1or2.default([]),

  /**
   * Short qualifiers under the buttons — "Local-only plaintext", "Built in
   * Rust". Three or four words each. They are the claims a reader checks
   * before deciding whether the subhead is worth their time, and they are
   * capped at four because a fifth turns a glance into a list.
   */
  highlights: z
    .array(
      z.object({
        icon: z.string().min(1).optional(),
        label: z.string().min(1),
      }),
    )
    .max(4)
    .default([]),

  /** Which side the visual sits on. The copy takes the other. */
  media: z.enum(['left', 'right']).default('right'),

  /** An image, or a terminal transcript. */
  visual: asset.optional(),
  terminal: z
    .object({
      title: z.string().min(1).default('terminal'),
      body: z.string().min(1),
    })
    .optional(),
}).refine((section) => [section.visual, section.terminal].filter(Boolean).length === 1, {
  message: 'a hero-split must declare exactly one of: visual, terminal',
});

/**
 * Numbered steps beside a panel that shows their result.
 *
 * The pairing is the point. A list of steps tells you what to do and a picture
 * of the outcome tells you what you get, and each is weaker alone: steps with
 * no result leave you guessing what you are working toward, and a result with
 * no steps is a screenshot. Side by side, the reader checks one against the
 * other as they go.
 *
 * Distinct from `timeline`, which looks similar and answers a different
 * question. A timeline is dated and its order is history — it happened in that
 * sequence. These are numbered and their order is instruction — do this, then
 * this. Merging them would mean a `date` that is sometimes an index.
 *
 * Numbers are rendered, never authored. An author writing "01", "02" into
 * titles is one inserted step away from renumbering by hand, and the list is
 * an `<ol>` — the sequence is already in the markup, and the markers only
 * draw what the element means.
 */
export const stepsPanelSchema = baseSectionSchema
  .extend({
    type: z.literal('steps-panel'),
    heading: z.string().optional(),
    subhead: z.string().optional(),

    /**
     * Two at minimum, because one step is not a sequence.
     *
     * Twelve at most, and the ceiling is about the panel rather than the list.
     * The panel is sticky, so it stays beside the steps however many there are
     * — but a legend that outruns a screen twice over has stopped being a
     * legend and become an article, and an article belongs in a guide.
     */
    steps: z
      .array(
        z.object({
          title: z.string().min(1),
          body: z.string().optional(),
        }),
      )
      .min(2)
      .max(12),

    /** Which side the panel sits on. The steps take the other. */
    media: z.enum(['left', 'right']).default('right'),

    /** An image, a terminal transcript, or a diagram. */
    visual: asset.optional(),
    terminal: z
      .object({
        title: z.string().min(1).default('terminal'),
        body: z.string().min(1),
      })
      .optional(),

    /**
     * Mermaid source, drawn in the browser and themed like every other diagram
     * on the site.
     *
     * A sequence diagram beside a numbered legend is the pairing this section
     * was built for, and it is the case a screenshot serves worst: an image
     * carries one palette into a site with fourteen, cannot do dark mode, is
     * unreadable to a screen reader, and is wrong the day a step changes.
     */
    diagram: z.string().min(1).optional(),

    /** A line under the panel, for what the panel cannot say itself. */
    caption: z.string().optional(),
  })
  .refine(
    (section) =>
      [section.visual, section.terminal, section.diagram].filter(Boolean).length === 1,
    { message: 'a steps-panel must declare exactly one of: visual, terminal, diagram' },
  );

/**
 * A row of cards, each carrying a snippet of configuration.
 *
 * For the page that has to compare options which differ mainly in what you
 * write down — CI modes, deployment targets, auth strategies. The prose says
 * what each one is for and the snippet says what it costs you, and a reader
 * comparing them is really comparing the snippets.
 *
 * `feature-grid` is the wrong home for this even though it looks close. Its
 * cards take an icon and a sentence, and the moment one of them holds twelve
 * lines of YAML the row stops being a grid of equals: a card sized by a code
 * block cannot sit beside a card sized by a sentence. This section makes the
 * code the reason the cards exist and sizes them all to the largest.
 *
 * The code is not highlighted. Shiki runs at build time over MDX, and reaching
 * it from a YAML section would mean either shipping a highlighter to the
 * browser or threading a build step through the content loader — for snippets
 * that are mostly keys and paths, where colour adds little.
 */
export const codeCardsSchema = baseSectionSchema.extend({
  type: z.literal('code-cards'),
  heading: z.string().optional(),
  subhead: z.string().optional(),
  eyebrow: z.string().optional(),

  /**
   * Two to four. One card is not a comparison, and a fifth pushes every card
   * below a width where its snippet wraps into nonsense.
   */
  cards: z
    .array(
      z.object({
        eyebrow: z.string().optional(),
        title: z.string().min(1),
        body: z.string().optional(),
        /** Marks the one being recommended. At most one, unenforced. */
        featured: z.boolean().default(false),
        code: z.object({
          title: z.string().min(1).optional(),
          body: z.string().min(1),
        }),
        href: z.string().optional(),
      }),
    )
    .min(2)
    .max(4),

  /** A closing line under the row — the caveat that applies to all of them. */
  footnote: z.string().optional(),
});

/**
 * Two axes crossed, and a sentence in every cell.
 *
 * For the page that has to say "these choices are independent" — four CI modes
 * against two encryption modes, five environments against three capabilities.
 * The claim such a page is really making is that the grid is *full*: every
 * combination is legal. A list of the eight combinations does not make that
 * claim, and a reader has to count to check it. A grid makes it visible.
 *
 * A real `<table>` with real header cells, so a screen reader announces "row
 * passphrase, column portal-key" when it reads a cell — which is the entire
 * content of that cell's meaning, and exactly what a grid of divs loses.
 */
export const matrixSchema = baseSectionSchema
  .extend({
    type: z.literal('matrix'),
    heading: z.string().optional(),
    subhead: z.string().optional(),
    eyebrow: z.string().optional(),

    /** Names for the two axes, shown in the corner cell. */
    rowAxis: z.string().optional(),
    columnAxis: z.string().optional(),

    columns: z.array(z.object({ label: z.string().min(1) })).min(2).max(5),

    rows: z
      .array(
        z.object({
          label: z.string().min(1),
          /** A clause under the row's name — what this row *is*. */
          description: z.string().optional(),
          /** One per column, in order. */
          cells: z.array(z.string().min(1)).min(2).max(5),
        }),
      )
      .min(2)
      .max(6),

    footnote: z.string().optional(),
  })
  .refine((section) => section.rows.every((row) => row.cells.length === section.columns.length), {
    message: 'every row needs exactly one cell per column',
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

/**
 * A grid of full-bleed promo tiles: image behind, copy over it, one or two
 * links. The landing-page building block for a product family.
 *
 * Distinct from `bento-grid`, where the image sits *below* the copy in a card.
 * Here the image is the tile, which is what gives a product page its weight.
 *
 * A tile carries `links` rather than being one big link, because a tile that
 * is itself a link cannot contain links — set `href` for the whole-tile form,
 * and then `links` is dropped.
 */
export const promoGridSchema = baseSectionSchema.extend({
  type: z.literal('promo-grid'),
  heading: z.string().optional(),
  gap: z.enum(['none', 'tight', 'normal']).default('normal'),
  items: z
    .array(
      z.object({
        eyebrow: z.string().optional(),
        title: z.string().min(1),
        body: z.string().optional(),
        image: asset.optional(),
        video: asset.optional(),
        alt: z.string().default(''),
        /** Full width, or half so two sit side by side. */
        width: z.enum(['full', 'half']).default('full'),
        /** Where the copy sits in the tile. */
        align: z.enum(['top', 'center', 'bottom']).default('top'),
        /**
         * Which part of the image to keep in frame. Copy sits over the tile, so
         * without this the subject and the words end up in the same place —
         * `bottom` is the usual answer when the copy is at the top.
         */
        focus: z.enum(['top', 'center', 'bottom']).default('center'),
        tone: z.enum(['light', 'dark', 'auto']).default('auto'),
        scrim: z.number().min(0).max(1).default(0),
        height: z.enum(['short', 'tall', 'full']).default('tall'),
        links: z
          .array(z.object({ label: z.string().min(1), href: z.string().min(1) }))
          .max(2)
          .default([]),
        href: z.string().optional(),
      }),
    )
    .min(1),
});

/** Registration order is the order shown in "valid types" build errors. */
const ALL = [
  { type: 'hero', schema: heroSchema, providesPageHeading: true },
  { type: 'hero-split', schema: heroSplitSchema, providesPageHeading: true },
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
  { type: 'steps-panel', schema: stepsPanelSchema },
  { type: 'code-cards', schema: codeCardsSchema },
  { type: 'matrix', schema: matrixSchema },
  { type: 'team-grid', schema: teamGridSchema },
  { type: 'gallery', schema: gallerySchema },
  { type: 'promo-grid', schema: promoGridSchema },
] as const;

export function registerBuiltinSections(): void {
  for (const def of ALL) registerSection(def);
}

// Built-ins register on import so any consumer of the registry sees them.
registerBuiltinSections();

export type HeroSection = z.infer<typeof heroSchema>;
export type HeroSplitSection = z.infer<typeof heroSplitSchema>;
export type StepsPanelSection = z.infer<typeof stepsPanelSchema>;
export type CodeCardsSection = z.infer<typeof codeCardsSchema>;
export type MatrixSection = z.infer<typeof matrixSchema>;
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
export type PromoGridSection = z.infer<typeof promoGridSchema>;

export type AnySection =
  | HeroSection
  | HeroSplitSection
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
  | StepsPanelSection
  | CodeCardsSection
  | MatrixSection
  | TeamGridSection
  | GallerySection
  | PromoGridSection;
