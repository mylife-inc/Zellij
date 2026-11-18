import Link from 'next/link';
import type { SectionProps } from '../types';
import type { FeatureGridSection } from '../schemas';
import { InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Icon } from '../../icons';
import { Reveal } from '../../motion/reveal';

export function FeatureGrid({ section, defaultMotion, headingLevel }: SectionProps<FeatureGridSection>) {
  const motion = section.motion ?? defaultMotion;
  const ItemHeading = (headingLevel ? `h${headingLevel + (section.heading ? 1 : 0)}` : 'h3') as 'h2' | 'h3' | 'h4' | 'h5';

  return (
    <SectionShell
      type="feature-grid"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-feature-grid"
    >
      <SectionHeader level={headingLevel}
        {...(section.eyebrow ? { eyebrow: section.eyebrow } : {})}
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <ul className="zellij-feature-list" data-columns={section.columns}>
        {section.items.map((item, index) => {
          const body = (
            <>
              {item.icon ? (
                <span className="zellij-feature-icon">
                  <Icon
                    name={item.icon}
                    context={`feature-grid item "${item.title}"`}
                    size={22}
                  />
                </span>
              ) : null}
              <ItemHeading className="zellij-feature-title">{item.title}</ItemHeading>
              {item.body ? (
                <p className="zellij-feature-body">
                  <InlineMarkdown text={item.body} />
                </p>
              ) : null}
            </>
          );

          return (
            <Reveal as="li" key={item.title} motion={motion} index={index} className="zellij-feature-card">
              {item.href ? (
                <Link className="zellij-feature-link" href={item.href}>
                  {body}
                </Link>
              ) : (
                body
              )}
            </Reveal>
          );
        })}
      </ul>
    </SectionShell>
  );
}
