import type { SectionProps } from '../types';
import type { TimelineSection } from '../schemas';
import { Icon } from '../../icons';
import { InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/**
 * A dated sequence. An ordered list, because that is what it is — the rail and
 * markers are decoration over real list semantics.
 */
export function Timeline({ section, site, defaultMotion }: SectionProps<TimelineSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="timeline"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-timeline"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <ol className="zellij-timeline-list">
        {section.items.map((item, index) => (
          <Reveal
            as="li"
            key={`${item.date}-${item.title}`}
            motion={motion}
            index={index}
            direction="left"
            className="zellij-timeline-item"
          >
            <span className="zellij-timeline-marker" aria-hidden="true">
              {item.icon ? (
                <Icon name={item.icon} context={`timeline › ${item.title}`} size={14} />
              ) : null}
            </span>

            <div className="zellij-timeline-copy">
              {/* The date leads, so scanning the column reads as a sequence. */}
              <p className="zellij-timeline-date">{item.date}</p>
              <h3 className="zellij-timeline-title">{item.title}</h3>
              {item.body ? (
                <p className="zellij-timeline-body">
                  <InlineMarkdown text={item.body} />
                </p>
              ) : null}
              {item.image ? (
                <div className="zellij-timeline-media">
                  <SectionImage
                    basePath={basePath}
                    src={item.image}
                    alt=""
                    ratio="16 / 9"
                    sizes="(max-width: 900px) 100vw, 40vw"
                  />
                </div>
              ) : null}
            </div>
          </Reveal>
        ))}
      </ol>
    </SectionShell>
  );
}
