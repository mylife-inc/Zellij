import Image from 'next/image';
import Link from 'next/link';

import type { SectionProps } from '../types';
import type { MediaPanelSection } from '../schemas';
import { isExternal } from '../../content/paths';
import { assetHref } from '../../content/asset-href';
import { CtaGroup, InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { PanelVideo } from './panel-video';

/**
 * A rounded panel filled edge to edge with one image or video, with the copy
 * laid over it. The whole panel may be a link, in which case it wraps the
 * content rather than nesting a link inside a link.
 */
export function MediaPanel({ section, site, defaultMotion }: SectionProps<MediaPanelSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;
  const hasCopy = Boolean(section.eyebrow || section.title || section.body || section.cta.length);

  // A plain <video> is not touched by Next, so its URLs need the prefix applied
  // here — the same trap that made the nav logo 404 under a basePath.
  const media = section.video ? (
    <PanelVideo
      src={assetHref(section.video, basePath)}
      {...(section.poster ? { poster: assetHref(section.poster, basePath) } : {})}
      label={section.alt}
    />
  ) : (
    <Image
      className="zellij-panel-image"
      src={assetHref(section.image!, basePath)}
      alt={section.alt}
      fill
      sizes="(max-width: 900px) 100vw, 1200px"
    />
  );

  const inner = (
    <>
      <div className="zellij-panel-media">{media}</div>
      {section.scrim > 0 ? (
        <div
          className="zellij-panel-scrim"
          style={{ opacity: section.scrim }}
          aria-hidden="true"
          data-align={section.align}
        />
      ) : null}

      {hasCopy ? (
        <div className="zellij-panel-copy" data-align={section.align}>
          {section.eyebrow ? <p className="zellij-eyebrow">{section.eyebrow}</p> : null}
          {section.title ? <p className="zellij-panel-title">{section.title}</p> : null}
          {section.body ? (
            <p className="zellij-panel-body">
              <InlineMarkdown text={section.body} />
            </p>
          ) : null}
          {/* A panel that is itself a link must not contain buttons that are
              also links, so the CTAs are dropped in that case. */}
          {section.cta.length > 0 && !section.href ? <CtaGroup cta={section.cta} /> : null}
        </div>
      ) : null}
    </>
  );

  return (
    <SectionShell
      type="media-panel"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-media-panel"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <Reveal motion={motion} className="zellij-panel" data-ratio={section.ratio} data-tone={section.tone}>
        {section.href ? (
          isExternal(section.href) ? (
            <a
              className="zellij-panel-surface"
              href={section.href}
              target="_blank"
              rel="noreferrer noopener"
            >
              {inner}
            </a>
          ) : (
            <Link className="zellij-panel-surface" href={section.href}>
              {inner}
            </Link>
          )
        ) : (
          <div className="zellij-panel-surface">{inner}</div>
        )}
      </Reveal>
    </SectionShell>
  );
}
