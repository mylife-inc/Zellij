import type { SectionProps } from '../types';
import type { VideoEmbedSection } from '../schemas';
import { SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { VideoContent } from './video-content';

export function VideoEmbed({ section, site, defaultMotion }: SectionProps<VideoEmbedSection>) {
  const motion = section.motion ?? defaultMotion;

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
        <VideoContent {...section} basePath={site.config.basePath} label={section.heading ?? section.caption ?? 'Video'} />
      </Reveal>
    </SectionShell>
  );
}
