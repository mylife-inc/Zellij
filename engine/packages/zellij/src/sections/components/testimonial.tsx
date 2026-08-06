import type { SectionProps } from '../types';
import type { TestimonialSection } from '../schemas';
import { SectionHeader, SectionShell } from '../shared';
import { assetHref } from '../../content/asset-href';
import { Reveal } from '../../motion/reveal';
import { QuoteCard, type QuoteModel } from './quote-card';
import { TestimonialCarousel } from './testimonial-carousel';

export function Testimonial({ section, site, defaultMotion }: SectionProps<TestimonialSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  const quotes = section.quotes.map((quote) => ({
    ...quote,
    // Prefixed here: the optimizer resolves its `url` against the origin
    // root, so an unprefixed one fetches the 404 page (see SectionImage).
    avatarUrl: quote.avatar ? assetHref(quote.avatar, basePath) : undefined,
  }));

  return (
    <SectionShell
      type="testimonial"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-testimonial"
    >
      <SectionHeader {...(section.heading ? { heading: section.heading } : {})} motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      {/* A single quote needs no carousel chrome (spec §5.2 #7). */}
      {quotes.length === 1 ? (
        <Reveal motion={motion}>
          <QuoteCard quote={quotes[0]!} />
        </Reveal>
      ) : (
        <TestimonialCarousel quotes={quotes} />
      )}
    </SectionShell>
  );
}

export { QuoteCard, type QuoteModel };
