import type { SectionProps } from '../types';
import type { MediaAccordionNode, MediaAccordionSection } from '../media-accordion-schema';
import type { MotionPreset } from '../../config/schema';
import { InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { MediaAccordionControl } from './media-accordion-control';

export function MediaAccordionContent({ node, basePath, motion }: {
  node: MediaAccordionNode; basePath: string; motion: MotionPreset;
}) {
  return <MediaAccordionControl label={node.label} mediaSide={node.mediaSide} headingLevel={node.itemHeadingLevel} motion={motion}
    items={node.items.map(item => ({ title: item.title,
      body: <p><InlineMarkdown text={item.body} /></p>,
      media: <figure className="zellij-media-accordion-figure" data-fit={node.fit}>
        <SectionImage src={item.image} alt={item.alt} basePath={basePath} loading="eager"
          ratio={{ landscape: '16 / 10', square: '1', portrait: '3 / 4' }[node.ratio]} />
      </figure>,
      caption: item.caption ? <InlineMarkdown text={item.caption} /> : undefined,
    }))} />;
}

export function MediaAccordion({ section, site, defaultMotion }: SectionProps<MediaAccordionSection>) {
  const motion = section.motion ?? defaultMotion;
  return <SectionShell type="media-accordion" id={section.id} background={section.background} padding={section.padding} motion={motion}>
    <SectionHeader eyebrow={section.eyebrow} heading={section.heading} subhead={section.subhead} titleGradient={section.titleGradient} motion={motion} />
    <MediaAccordionContent node={section} basePath={site.config.basePath} motion={motion} />
  </SectionShell>;
}
