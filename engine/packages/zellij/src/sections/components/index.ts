import type { ComponentType } from 'react';
import type { SectionProps } from '../types';

import { BentoGrid } from './bento-grid';
import { CardCarousel } from './card-carousel';
import { CtaBanner } from './cta-banner';
import { Faq } from './faq';
import { Gallery } from './gallery';
import { FeatureGrid } from './feature-grid';
import { Hero } from './hero';
import { LogoStrip } from './logo-strip';
import { MarkdownProse } from './markdown-prose';
import { MediaPanel } from './media-panel';
import { PricingTable } from './pricing-table';
import { PromoGrid } from './promo-grid';
import { Showcase } from './showcase';
import { StatsBand } from './stats-band';
import { TeamGrid } from './team-grid';
import { Testimonial } from './testimonial';
import { Timeline } from './timeline';
import { VideoEmbed } from './video-embed';
import { Zigzag } from './zigzag';

/**
 * Type → component. Kept beside the schemas rather than inside them so the
 * content loader (a build-time module) never pulls React into its graph.
 */
export const SECTION_COMPONENTS: Record<string, ComponentType<SectionProps<never>>> = {
  hero: Hero as ComponentType<SectionProps<never>>,
  'logo-strip': LogoStrip as ComponentType<SectionProps<never>>,
  'feature-grid': FeatureGrid as ComponentType<SectionProps<never>>,
  zigzag: Zigzag as ComponentType<SectionProps<never>>,
  showcase: Showcase as ComponentType<SectionProps<never>>,
  'stats-band': StatsBand as ComponentType<SectionProps<never>>,
  testimonial: Testimonial as ComponentType<SectionProps<never>>,
  'pricing-table': PricingTable as ComponentType<SectionProps<never>>,
  faq: Faq as ComponentType<SectionProps<never>>,
  'cta-banner': CtaBanner as ComponentType<SectionProps<never>>,
  'markdown-prose': MarkdownProse as ComponentType<SectionProps<never>>,
  'video-embed': VideoEmbed as ComponentType<SectionProps<never>>,
  'media-panel': MediaPanel as ComponentType<SectionProps<never>>,
  'card-carousel': CardCarousel as ComponentType<SectionProps<never>>,
  'bento-grid': BentoGrid as ComponentType<SectionProps<never>>,
  timeline: Timeline as ComponentType<SectionProps<never>>,
  'team-grid': TeamGrid as ComponentType<SectionProps<never>>,
  gallery: Gallery as ComponentType<SectionProps<never>>,
  'promo-grid': PromoGrid as ComponentType<SectionProps<never>>,
};

export {
  Hero,
  LogoStrip,
  FeatureGrid,
  Zigzag,
  Showcase,
  StatsBand,
  Testimonial,
  PricingTable,
  Faq,
  CtaBanner,
  MarkdownProse,
  VideoEmbed,
  MediaPanel,
  CardCarousel,
  BentoGrid,
  Timeline,
  TeamGrid,
  Gallery,
  PromoGrid,
};
