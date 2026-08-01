import Link from 'next/link';

import type { SectionProps } from '../types';
import type { BentoGridSection } from '../schemas';
import { Icon } from '../../icons';
import { isExternal } from '../../content/paths';
import { InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/**
 * An asymmetric tile grid. Tiles claim one, two or four cells on a wide
 * viewport and collapse to a single column on a narrow one.
 */
export function BentoGrid({ section, site, defaultMotion }: SectionProps<BentoGridSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="bento-grid"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-bento"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <ul className="zellij-bento-grid">
        {section.items.map((item, index) => {
          const body = (
            <>
              {item.icon ? (
                <span className="zellij-bento-icon">
                  <Icon name={item.icon} context={`bento-grid › ${item.title}`} size={22} />
                </span>
              ) : null}
              <h3 className="zellij-bento-title">{item.title}</h3>
              {item.body ? (
                <p className="zellij-bento-body">
                  <InlineMarkdown text={item.body} />
                </p>
              ) : null}
              {item.image ? (
                <div className="zellij-bento-media">
                  <SectionImage
                    basePath={basePath}
                    src={item.image}
                    alt=""
                    ratio="16 / 10"
                    sizes="(max-width: 900px) 100vw, 45vw"
                  />
                </div>
              ) : null}
            </>
          );

          const attrs = {
            className: 'zellij-bento-tile',
            'data-span': item.span,
            'data-background': item.background ?? 'default',
          };

          return (
            <Reveal
              as="li"
              key={`${item.title}-${index}`}
              motion={motion}
              index={index}
              className="zellij-bento-cell"
              data-span={item.span}
            >
              {item.href ? (
                isExternal(item.href) ? (
                  <a {...attrs} href={item.href} target="_blank" rel="noreferrer noopener">
                    {body}
                  </a>
                ) : (
                  <Link {...attrs} href={item.href}>
                    {body}
                  </Link>
                )
              ) : (
                <div {...attrs}>{body}</div>
              )}
            </Reveal>
          );
        })}
      </ul>
    </SectionShell>
  );
}
