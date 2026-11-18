import Link from 'next/link';
import type { StoryNode } from '../story-schema';
import type { MotionPreset } from '../../config/schema';
import { assetHref } from '../../content/asset-href';
import { CtaLink, InlineMarkdown, SectionImage } from '../shared';
import { StoryScroll } from './story-scroll';

export function StoryContent({ node, basePath, motion }: { node: StoryNode; basePath: string; motion: MotionPreset }) {
  const Heading = `h${node.sceneHeadingLevel}` as 'h2' | 'h3' | 'h4';
  return <StoryScroll label={node.label} distance={node.distance} mediaSide={node.mediaSide} motion={motion}>
    {node.scenes.map((scene, i) => <div className="zellij-story-scene" key={i}>
      <div className="zellij-story-copy">
        <p className="zellij-story-number" aria-hidden="true">{String(i + 1).padStart(2, '0')} / {String(node.scenes.length).padStart(2, '0')}</p>
        <Heading className="zellij-compose-heading" data-scale="title">{scene.heading}</Heading>
        <p className="zellij-compose-text" data-size="lead"><InlineMarkdown text={scene.body} /></p>
        {scene.cta ? <div className="zellij-compose-action"><CtaLink cta={scene.cta} /></div> : null}
      </div>
      <figure className="zellij-story-media">
        {scene.media.kind === 'image' ? <SectionImage src={scene.media.src} alt={scene.media.alt} basePath={basePath} ratio="16 / 10" loading="eager" /> : <>
          <video data-story-video="" data-start={scene.media.start} data-end={scene.media.end}
            src={assetHref(scene.media.src, basePath)} poster={assetHref(scene.media.poster, basePath)}
            aria-label={scene.media.label} controls playsInline muted preload="none">
            {scene.media.tracks?.map(track => <track key={`${track.srclang}-${track.kind}`} {...track} src={assetHref(track.src, basePath)} />)}
          </video>
          <figcaption><a href={assetHref(scene.media.src, basePath)}>Open video: {scene.media.label}</a>
            {scene.media.transcript ? <> · <Link href={scene.media.transcript}>Read transcript</Link></> : null}
          </figcaption>
        </>}
      </figure>
    </div>)}
  </StoryScroll>;
}
