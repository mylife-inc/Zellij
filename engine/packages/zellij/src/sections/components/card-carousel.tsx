import Link from 'next/link';
import type { ReactNode } from 'react';

import type { SectionProps } from '../types';
import type { CardCarouselSection } from '../schemas';
import { isExternal } from '../../content/paths';
import { InlineMarkdown, SectionImage, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { CardCarouselTrack } from './card-carousel-track';

/**
 * A horizontally scrolling row of cards, with the heading and an optional
 * trailing link sharing the top row.
 */
export function CardCarousel({ section, site, defaultMotion }: SectionProps<CardCarouselSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="card-carousel"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-card-carousel"
      // Cards bleed past the container so the next one peeks in from the edge.
      full
    >
      <div className="zellij-container">
        <Reveal motion={motion} className="zellij-carousel-head">
          <div>
            {section.heading ? (
              <h2 {...gradientTextProps(section.titleGradient, 'zellij-section-heading')}>
                {section.heading}
              </h2>
            ) : null}
            {section.subhead ? <p className="zellij-section-subhead">{section.subhead}</p> : null}
          </div>

          {section.link ? (
            <CardLink className="zellij-carousel-link" href={section.link.href}>
              {section.link.label}
              <span aria-hidden="true"> ›</span>
            </CardLink>
          ) : null}
        </Reveal>
      </div>

      <div className="zellij-carousel-rail" data-card-width={section.cardWidth}>
        <CardCarouselTrack label={section.heading ?? 'Cards'}>
          {section.items.map((item, index) => {
            const body = (
              <>
                <div className="zellij-card-copy">
                  {item.eyebrow ? <p className="zellij-card-eyebrow">{item.eyebrow}</p> : null}
                  <p className="zellij-card-title">{item.title}</p>
                  {item.body ? (
                    <p className="zellij-card-body">
                      <InlineMarkdown text={item.body} />
                    </p>
                  ) : null}
                </div>

                {item.image ? (
                  <div className="zellij-card-media">
                    <SectionImage
                      basePath={basePath}
                      src={item.image}
                      alt=""
                      ratio="4 / 3"
                      sizes="(max-width: 700px) 80vw, 380px"
                    />
                  </div>
                ) : null}
              </>
            );

            const shared = {
              className: 'zellij-scroll-card',
              'data-zellij-card': true,
              'data-background': item.background ?? 'default',
            };

            return (
              <Reveal
                key={`${item.title}-${index}`}
                motion={motion}
                index={index}
                as="article"
                className="zellij-scroll-card-shell"
              >
                {item.href ? (
                  <CardLink {...shared} href={item.href}>
                    {body}
                  </CardLink>
                ) : (
                  <div {...shared}>{body}</div>
                )}
              </Reveal>
            );
          })}
        </CardCarouselTrack>
      </div>
    </SectionShell>
  );
}

/** Internal links route; external ones leave in a new tab. */
function CardLink({
  href,
  children,
  ...rest
}: { href: string; children: ReactNode } & Record<string, unknown>) {
  if (isExternal(href)) {
    return (
      <a {...rest} href={href} target="_blank" rel="noreferrer noopener">
        {children}
      </a>
    );
  }
  return (
    <Link {...rest} href={href}>
      {children}
    </Link>
  );
}
