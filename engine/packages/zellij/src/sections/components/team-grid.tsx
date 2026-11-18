import type { SectionProps } from '../types';
import type { TeamGridSection } from '../schemas';
import { isExternal } from '../../content/paths';
import { InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/** People: a board, a faculty, a team, a list of contributors. */
export function TeamGrid({ section, site, defaultMotion, headingLevel }: SectionProps<TeamGridSection>) {
  const motion = section.motion ?? defaultMotion;
  const ItemHeading = (headingLevel ? `h${headingLevel + (section.heading ? 1 : 0)}` : 'h3') as 'h2' | 'h3' | 'h4' | 'h5';
  const { basePath } = site.config;

  return (
    <SectionShell
      type="team-grid"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-team"
    >
      <SectionHeader level={headingLevel}
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <ul className="zellij-team-list" data-columns={section.columns}>
        {section.people.map((person, index) => (
          <Reveal as="li" key={person.name} motion={motion} index={index} className="zellij-team-card">
            {person.photo ? (
              <div className="zellij-team-photo">
                <SectionImage
                  basePath={basePath}
                  src={person.photo}
                  // A portrait's alt is the person, not a description of a photo.
                  alt={person.name}
                  ratio="1 / 1"
                  sizes="(max-width: 700px) 50vw, 260px"
                />
              </div>
            ) : null}

            <ItemHeading className="zellij-team-name">{person.name}</ItemHeading>
            {person.role ? <p className="zellij-team-role">{person.role}</p> : null}
            {person.bio ? (
              <p className="zellij-team-bio">
                <InlineMarkdown text={person.bio} />
              </p>
            ) : null}

            {person.links.length > 0 ? (
              <ul className="zellij-team-links">
                {person.links.map((link) => (
                  <li key={link.href}>
                    <a
                      className="zellij-team-link"
                      href={link.href}
                      {...(isExternal(link.href)
                        ? { target: '_blank', rel: 'noreferrer noopener' }
                        : {})}
                    >
                      {/* Named per person: a page of "Website" links is unusable
                          when read out of context by a screen reader. */}
                      <span className="zellij-visually-hidden">{person.name} — </span>
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            ) : null}
          </Reveal>
        ))}
      </ul>
    </SectionShell>
  );
}
