import type { CSSProperties, ReactNode } from 'react';
import type { SectionProps } from '../types';
import type { ComposeLayout, ComposeNode, ComposeSection } from '../compose-schema';
import type { MotionPreset } from '../../config/schema';
import { CtaLink, InlineMarkdown, SectionImage, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { GradientReveal } from '../../motion/gradient-reveal';
import { MediaAccordionContent } from './media-accordion';

function layoutProps(layout?: ComposeLayout) {
  return { 'data-layout': layout?.kind ?? 'stack', 'data-align': layout?.align ?? 'start',
    'data-gap': layout?.gap ?? 'medium', 'data-ratio': layout?.kind === 'split' ? layout.ratio : undefined,
    style: { '--zellij-compose-columns': layout?.kind === 'grid' ? layout.columns : 2 } as CSSProperties };
}

function Node({ node, basePath, motion, titleGradient }: {
  node: ComposeNode; basePath: string; motion: MotionPreset; titleGradient?: readonly string[];
}) {
  let content: ReactNode;
  switch (node.type) {
    case 'group': content = <div className="zellij-compose-layout zellij-compose-group" {...layoutProps(node.layout)}
      data-surface={node.surface} data-padding={node.padding} data-radius={node.radius}>
      {node.children.map((child, i) => <Node key={i} node={child} basePath={basePath} motion={motion} titleGradient={titleGradient} />)}
    </div>; break;
    case 'heading': {
      const Tag = `h${node.level}` as 'h1' | 'h2' | 'h3' | 'h4';
      const gradient = gradientTextProps(node.gradient?.colors ?? titleGradient).style?.backgroundImage
        ?.replaceAll('var(--zellij-fg)', 'var(--zellij-compose-fg)')
        .replaceAll('var(--zellij-muted)', 'var(--zellij-compose-muted)');
      content = <Tag className="zellij-compose-heading" data-scale={node.scale} data-weight={node.weight}>
        {gradient ? <GradientReveal text={node.text} gradient={String(gradient)}
          motion={node.gradient?.effect === 'reveal' ? motion : 'none'} /> : node.text}
      </Tag>; break;
    }
    case 'text': content = <p className="zellij-compose-text" data-size={node.size} data-tone={node.tone}><InlineMarkdown text={node.text} /></p>; break;
    case 'image': content = <figure className="zellij-compose-image">
      <SectionImage src={node.image} alt={node.alt} basePath={basePath} ratio={{ landscape: '16 / 10', square: '1', portrait: '3 / 4' }[node.ratio]} />
      {node.caption ? <figcaption><InlineMarkdown text={node.caption} /></figcaption> : null}
    </figure>; break;
    case 'button': content = <div className="zellij-compose-action"><CtaLink cta={node} /></div>; break;
    case 'stat': content = <dl className="zellij-compose-stat"><dt>{node.label}</dt><dd>{node.value}</dd></dl>; break;
    case 'divider': content = <hr className="zellij-compose-divider" />; break;
    case 'media-accordion': content = <MediaAccordionContent node={node} basePath={basePath} motion={motion} />; break;
  }
  return node.type !== 'divider' && node.effect === 'reveal'
    ? <Reveal motion={motion} className="zellij-compose-reveal">{content}</Reveal> : content;
}

export function Compose({ section, site, defaultMotion }: SectionProps<ComposeSection>) {
  const motion = section.motion ?? defaultMotion;
  return <SectionShell type="compose" id={section.id} motion={motion} background={section.background} padding={section.padding}>
    <div className="zellij-compose-layout" {...layoutProps(section.layout)}>
      {section.children.map((node, i) => <Node key={i} node={node} basePath={site.config.basePath} motion={motion} titleGradient={section.titleGradient} />)}
    </div>
  </SectionShell>;
}
