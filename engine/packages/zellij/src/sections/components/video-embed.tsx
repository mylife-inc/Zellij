import type { SectionProps } from '../types';
import type { VideoEmbedSection } from '../schemas';
import { SectionHeader, SectionShell } from '../shared';
import { isExternal } from '../../content/paths';
import { assetHref } from '../../content/asset-href';
import { Reveal } from '../../motion/reveal';
import { VideoPlayer } from './video-player';

/** Recognises YouTube and Vimeo URLs and returns a privacy-friendly embed URL. */
function toEmbedUrl(src: string): string | undefined {
  const youtube = /(?:youtube\.com\/(?:watch\?v=|embed\/)|youtu\.be\/)([\w-]{6,})/.exec(src);
  if (youtube) return `https://www.youtube-nocookie.com/embed/${youtube[1]}`;

  const vimeo = /vimeo\.com\/(?:video\/)?(\d+)/.exec(src);
  if (vimeo) return `https://player.vimeo.com/video/${vimeo[1]}`;

  return undefined;
}

export function VideoEmbed({ section, defaultMotion }: SectionProps<VideoEmbedSection>) {
  const motion = section.motion ?? defaultMotion;
  const embedUrl = isExternal(section.src) ? toEmbedUrl(section.src) : undefined;

  return (
    <SectionShell
      type="video-embed"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-video"
    >
      <SectionHeader {...(section.heading ? { heading: section.heading } : {})} motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <Reveal motion={motion}>
        <figure className="zellij-video-figure">
          <VideoPlayer
            {...(embedUrl ? { embedUrl } : { fileUrl: assetHref(section.src) })}
            {...(section.poster ? { poster: assetHref(section.poster) } : {})}
            label={section.heading ?? section.caption ?? 'Video'}
          />
          {section.caption ? (
            <figcaption className="zellij-video-caption">{section.caption}</figcaption>
          ) : null}
        </figure>
      </Reveal>
    </SectionShell>
  );
}
