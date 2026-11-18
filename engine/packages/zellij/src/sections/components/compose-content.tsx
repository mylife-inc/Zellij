import Link from 'next/link';
import type { CSSProperties } from 'react';
import type { ComposeLeaf } from '../compose-schema';
import { assetHref } from '../../content/asset-href';
import { Icon } from '../../icons';
import { InlineMarkdown, SectionImage } from '../shared';
import { GalleryLightbox } from './gallery-lightbox';
import { CardCarouselTrack } from './card-carousel-track';
import { ComposeProse } from './compose-prose';

type ContentNode = Extract<ComposeLeaf, {type: 'prose' | 'list' | 'quote' | 'icon' | 'code' | 'table' | 'disclosure' | 'carousel' | 'gallery'}>;
export function ComposeContent({ node, basePath }: { node: ContentNode; basePath: string }) {
  switch (node.type) {
    case 'prose': return <ComposeProse body={node.body} basePath={basePath} />;
    case 'list': {
      const Tag = node.ordered ? 'ol' : 'ul';
      return <Tag className="zellij-compose-list">{node.items.map((item, i) => <li key={i}><InlineMarkdown text={item} /></li>)}</Tag>;
    }
    case 'quote': return <figure className="zellij-compose-quote"><blockquote><InlineMarkdown text={node.text} /></blockquote><figcaption>{node.author}</figcaption></figure>;
    case 'icon': return <span className="zellij-compose-icon" role="img" aria-label={node.label}><Icon name={node.name} context="compose icon" size={{small: 24, medium: 40, large: 64}[node.size]} /></span>;
    case 'code': return <figure className="zellij-compose-code"><figcaption>{node.title}</figcaption><pre tabIndex={0} aria-label={node.title}><code className={node.language ? `language-${node.language}` : undefined}>{node.code}</code></pre></figure>;
    case 'table': return <div className="zellij-compose-table-wrap" tabIndex={0} role="region" aria-label={node.caption}><table className="zellij-compose-table"><caption>{node.caption}</caption><thead><tr><th scope="col">{node.rowLabel}</th>{node.columns.map(column => <th scope="col" key={column}>{column}</th>)}</tr></thead><tbody>{node.rows.map((row, i) => <tr key={i}><th scope="row">{row.label}</th>{row.cells.map((cell, j) => <td key={j}><InlineMarkdown text={cell} /></td>)}</tr>)}</tbody></table></div>;
    case 'disclosure': return <details className="zellij-compose-disclosure" open={node.open}><summary>{node.summary}</summary><div><InlineMarkdown text={node.body} /></div></details>;
    case 'carousel': return <div className="zellij-compose-carousel"><CardCarouselTrack label={node.label}>{node.items.map((item, i) => <article className="zellij-compose-carousel-card" data-zellij-card key={i}>
      {item.image ? <SectionImage src={item.image} alt={item.alt} basePath={basePath} /> : null}
      <h3>{item.href ? <Link href={item.href}>{item.title}</Link> : item.title}</h3>
      {item.body ? <p><InlineMarkdown text={item.body} /></p> : null}
    </article>)}</CardCarouselTrack></div>;
    case 'gallery': {
      const images = node.items.map(item => ({src: assetHref(item.image, basePath), alt: item.alt, ...(item.caption ? {caption: item.caption} : {})}));
      return <div className="zellij-compose-gallery" style={{'--zellij-compose-gallery-columns': node.columns} as CSSProperties}><GalleryLightbox images={images} label={node.label}>
        <ul>{images.map((item, i) => <li key={i}><figure><a href={item.src} data-lightbox-index={i} aria-label={`View ${item.alt} full size`}><SectionImage src={node.items[i]!.image} alt={item.alt} basePath={basePath} /></a>{item.caption ? <figcaption>{item.caption}</figcaption> : null}</figure></li>)}</ul>
      </GalleryLightbox></div>;
    }
  }
}
