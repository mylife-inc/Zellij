import Image from 'next/image';
import Link from 'next/link';
import type { ReactNode } from 'react';

import type { SectionProps } from '../types';
import type { PromoGridSection } from '../schemas';
import { assetUrl, isExternal, withBasePath } from '../../content/paths';
import { InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { PanelVideo } from './panel-video';

/**
 * A grid of full-bleed promo tiles — the landing-page building block for a
 * product family. Full-width tiles stack; half-width ones pair up on a wide
 * viewport and stack below it.
 */
export function PromoGrid({ section, site, defaultMotion }: SectionProps<PromoGridSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="promo-grid"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-promo"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <div className="zellij-promo-grid" data-gap={section.gap}>
        {section.items.map((item, index) => {
          const inner = (
            <>
              <div className="zellij-promo-media">
                {item.video ? (
                  <PanelVideo
                    src={withBasePath(assetUrl(item.video), basePath)}
                    label={item.alt}
                  />
                ) : item.image ? (
                  <Image
                    className="zellij-promo-image"
                    src={withBasePath(assetUrl(item.image), basePath)}
                    alt={item.alt}
                    fill
                    sizes={item.width === 'half' ? '(max-width: 900px) 100vw, 50vw' : '100vw'}
                  />
                ) : null}
              </div>

              {item.scrim > 0 ? (
                <div
                  className="zellij-promo-scrim"
                  style={{ opacity: item.scrim }}
                  data-align={item.align}
                  aria-hidden="true"
                />
              ) : null}

              <div className="zellij-promo-copy" data-align={item.align}>
                {item.eyebrow ? <p className="zellij-promo-eyebrow">{item.eyebrow}</p> : null}
                <p className="zellij-promo-title">{item.title}</p>
                {item.body ? (
                  <p className="zellij-promo-body">
                    <InlineMarkdown text={item.body} />
                  </p>
                ) : null}

                {/*
                 * Dropped when the tile itself is a link: a link inside a link
                 * is not valid HTML, and browsers recover from it unpredictably.
                 * Authors pick one form or the other.
                 */}
                {item.links.length > 0 && !item.href ? (
                  <p className="zellij-promo-links">
                    {item.links.map((link) => (
                      <PromoLink key={link.href} href={link.href}>
                        {link.label}
                        <span aria-hidden="true"> ›</span>
                      </PromoLink>
                    ))}
                  </p>
                ) : null}
              </div>
            </>
          );

          const attrs = {
            className: 'zellij-promo-tile',
            'data-width': item.width,
            'data-height': item.height,
            'data-tone': item.tone,
            'data-focus': item.focus,
          };

          return (
            <Reveal
              key={`${item.title}-${index}`}
              motion={motion}
              index={index}
              className="zellij-promo-cell"
              data-width={item.width}
            >
              {item.href ? (
                <PromoLink {...attrs} href={item.href}>
                  {inner}
                </PromoLink>
              ) : (
                <div {...attrs}>{inner}</div>
              )}
            </Reveal>
          );
        })}
      </div>
    </SectionShell>
  );
}

function PromoLink({
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
