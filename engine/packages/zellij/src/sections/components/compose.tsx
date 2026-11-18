import type { CSSProperties, ReactNode } from 'react';
import type { Site } from '../../content/site';
import { SectionContent } from './section-content';
import type { SectionProps } from '../types';
import type { ComposeLayout, ComposeNode, ComposeSection } from '../compose-schema';
import type { MotionPreset } from '../../config/schema';
import { CtaLink, InlineMarkdown, SectionImage, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { GradientReveal } from '../../motion/gradient-reveal';
import { MediaAccordionContent } from './media-accordion';
import { VideoContent } from './video-content';
import { StoryContent } from './story-content';
import { ChoiceContent } from './choice-content';
import { ComposeContent } from './compose-content';
import { ComposeBackdropMedia } from './compose-backdrop';
import { ComposeScrollStage } from './compose-scroll';
import { assetHref } from '../../content/asset-href';

function layoutProps(layout?: ComposeLayout) {
  return { 'data-layout': layout?.kind ?? 'stack', 'data-align': layout?.align ?? 'start',
    'data-gap': layout?.gap ?? 'medium', 'data-ratio': layout?.kind === 'split' ? layout.ratio : undefined,
    style: { '--zellij-compose-columns': layout?.kind === 'grid' ? layout.columns : 2 } as CSSProperties };
}

function Node({ node, basePath, motion, titleGradient, site }: {
  node: ComposeNode; site: Site; basePath: string; motion: MotionPreset; titleGradient?: readonly string[];
}) {
  let content: ReactNode;
  switch (node.type) {
    case 'group': content = <div className="zellij-compose-group zellij-compose-scope"
      data-surface={node.surface} data-padding={node.padding} data-radius={node.radius}>
      <Layout layout={node.layout} basePath={basePath}>
        {node.children.map((child, i) => <Node site={site} key={i} node={child} basePath={basePath} motion={motion} titleGradient={titleGradient} />)}
      </Layout>
    </div>;
      if (node.scroll) content = <ComposeScrollStage settings={node.scroll} motion={motion}>{content}</ComposeScrollStage>;
      break;
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
    case 'section': content = <SectionContent node={node} site={site} motion={motion} />; break;
    case 'story': content = <StoryContent node={node} basePath={basePath} motion={motion} />; break;
    case 'selector': case 'explorer': content = <ChoiceContent node={node} basePath={basePath} />; break;
    case 'prose': case 'list': case 'quote': case 'icon': case 'code': case 'table': case 'disclosure': case 'carousel': case 'gallery':
      content = <ComposeContent node={node} basePath={basePath} />; break;
    case 'divider': content = <hr className="zellij-compose-divider" />; break;
    case 'media-accordion': content = <MediaAccordionContent node={node} basePath={basePath} motion={motion} />; break;
    case 'video': content = <VideoContent {...node} basePath={basePath} composed
      caption={node.caption ? <InlineMarkdown text={node.caption} /> : undefined} />; break;
  }
  if (node.type !== 'divider' && node.type !== 'story' && node.type !== 'section' && node.effect === 'reveal') content = <Reveal motion={motion} className="zellij-compose-reveal">{content}</Reveal>;
  return node.type !== 'story' && node.type !== 'section' && node.place ? <div className="zellij-compose-placement" data-place={node.place}>{content}</div> : content;
}

function Layout({ layout, basePath, children }: { layout?: ComposeLayout; basePath: string; children: ReactNode }) {
  if (layout?.kind !== 'layers') return <div className="zellij-compose-layout" {...layoutProps(layout)}>{children}</div>;
  const backdrop = layout.backdrop;
  const media = backdrop.kind === 'image' ? {...backdrop, src: assetHref(backdrop.src, basePath)}
    : {...backdrop, src: assetHref(backdrop.src, basePath), poster: assetHref(backdrop.poster, basePath)};
  return <div className="zellij-compose-layers" data-tone={layout.tone} data-height={layout.height}
    style={{'--zellij-compose-scrim': layout.scrim} as CSSProperties}>
    <ComposeBackdropMedia backdrop={media}>
      <div className="zellij-compose-scrim" aria-hidden="true" />
      <div className="zellij-compose-layout" {...layoutProps(layout)}>{children}</div>
    </ComposeBackdropMedia>
  </div>;
}

export function Compose({ section, site, defaultMotion }: SectionProps<ComposeSection>) {
  const motion = section.motion ?? defaultMotion;
  const content = <Layout layout={section.layout} basePath={site.config.basePath}>
    {section.children.map((node, i) => <Node site={site} key={i} node={node} basePath={site.config.basePath} motion={motion} titleGradient={section.titleGradient} />)}
  </Layout>;
  return <SectionShell type="compose" id={section.id} motion={motion} background={section.background} padding={section.padding}>
    {section.scroll ? <ComposeScrollStage settings={section.scroll} motion={motion}>{content}</ComposeScrollStage> : content}
  </SectionShell>;
}
