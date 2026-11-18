import type { SectionProps } from '../types';
import type { PricingTableSection } from '../schemas';
import { CtaLink, SectionHeader, SectionShell } from '../shared';
import { Icon } from '../../icons';
import { Reveal } from '../../motion/reveal';

export function PricingTable({ section, defaultMotion, headingLevel }: SectionProps<PricingTableSection>) {
  const motion = section.motion ?? defaultMotion;
  const ItemHeading = (headingLevel ? `h${headingLevel + (section.heading ? 1 : 0)}` : 'h3') as 'h2' | 'h3' | 'h4' | 'h5';

  return (
    <SectionShell
      type="pricing-table"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-pricing"
    >
      <SectionHeader level={headingLevel}
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      {/* A lone tier centres rather than stretching, so a single "Contact us"
          plan still looks deliberate (spec §5.2 #8). */}
      <ul className="zellij-pricing-list" data-count={section.tiers.length}>
        {section.tiers.map((tier, index) => (
          <Reveal
            as="li"
            key={tier.name}
            motion={motion}
            index={index}
            className="zellij-pricing-tier"
          >
            <div className="zellij-pricing-card" data-highlighted={tier.highlighted}>
              {tier.highlighted ? (
                <p className="zellij-pricing-flag">Most popular</p>
              ) : null}

              <ItemHeading className="zellij-pricing-name">{tier.name}</ItemHeading>

              <p className="zellij-pricing-price">
                <span className="zellij-pricing-amount">{tier.price}</span>
                {tier.period ? (
                  <span className="zellij-pricing-period">{tier.period}</span>
                ) : null}
              </p>

              {tier.description ? (
                <p className="zellij-pricing-description">{tier.description}</p>
              ) : null}

              {tier.cta ? <CtaLink cta={tier.cta} className="zellij-pricing-cta" /> : null}

              {tier.features.length > 0 ? (
                <ul className="zellij-pricing-features">
                  {tier.features.map((feature) => (
                    <li key={feature}>
                      <Icon name="check" context="pricing-table feature" size={16} />
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
              ) : null}
            </div>
          </Reveal>
        ))}
      </ul>
    </SectionShell>
  );
}
