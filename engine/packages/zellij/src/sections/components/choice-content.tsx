import type { ChoiceNode } from '../choice-schema';
import { CtaLink, InlineMarkdown, SectionImage } from '../shared';
import { ChoiceControl } from './choice-control';

/** All authored content is rendered on the server; only selection runs on the client. */
export function ChoiceContent({ node, basePath }: { node: ChoiceNode; basePath: string }) {
  return <ChoiceControl kind={node.type} label={node.label} presentation={node.presentation}
    initial={node.initial} mediaSide={node.mediaSide} headingLevel={node.itemHeadingLevel}
    items={node.items.map(item => ({id: item.id, label: item.label, heading: item.heading,
      swatch: 'swatch' in item ? item.swatch : undefined,
      body: <p className="zellij-choice-body"><InlineMarkdown text={item.body} /></p>,
      media: item.media ? <figure className="zellij-choice-media" data-fit={node.fit}>
        <SectionImage src={item.media.image} alt={item.media.alt} basePath={basePath} loading="eager"
          ratio={{landscape: '16 / 10', square: '1', portrait: '3 / 4'}[node.ratio]} />
        {item.media.caption ? <figcaption><InlineMarkdown text={item.media.caption} /></figcaption> : null}
      </figure> : undefined,
      facts: item.facts ? <dl className="zellij-choice-facts">{item.facts.map((fact, i) => <div key={i}><dt>{fact.label}</dt><dd><InlineMarkdown text={fact.value} /></dd></div>)}</dl> : undefined,
      cta: item.cta ? <CtaLink cta={item.cta} /> : undefined,
    }))} />;
}
