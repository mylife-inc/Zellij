import Link from 'next/link';
import type { SectionProps } from '../types';
import type { CodeCardsSection } from '../schemas';
import { InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/**
 * Cards that exist because of the code in them.
 *
 * The card is an `<article>` and its snippet a `<pre>`, so the code survives
 * being copied, read aloud and searched. Nothing here is a picture of code.
 *
 * A whole-card link is a `<Link>` wrapped around the title rather than around
 * the card: a card containing selectable code cannot itself be a link, because
 * selecting a line of it would navigate away mid-drag.
 */
export function CodeCards({ section, defaultMotion }: SectionProps<CodeCardsSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="code-cards"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-code-cards"
    >
      <SectionHeader
        {...(section.eyebrow ? { eyebrow: section.eyebrow } : {})}
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <div className="zellij-code-cards-row" data-count={section.cards.length}>
        {section.cards.map((card, index) => (
          <Reveal
            as="article"
            key={card.title}
            motion={motion}
            index={index}
            className="zellij-code-card"
            {...(card.featured ? { 'data-featured': 'true' } : {})}
          >
            {card.eyebrow ? <p className="zellij-code-card-eyebrow">{card.eyebrow}</p> : null}

            <h3 className="zellij-code-card-title">
              {card.href ? <Link href={card.href}>{card.title}</Link> : card.title}
            </h3>

            {card.body ? (
              <p className="zellij-code-card-body">
                <InlineMarkdown text={card.body} />
              </p>
            ) : null}

            <div className="zellij-code-card-code">
              {card.code.title ? (
                <p className="zellij-code-card-filename">{card.code.title}</p>
              ) : null}
              <pre>
                <code>{card.code.body.replace(/\n$/, '')}</code>
              </pre>
            </div>
          </Reveal>
        ))}
      </div>

      {section.footnote ? (
        <Reveal motion={motion} as="p" className="zellij-code-cards-footnote" index={1}>
          <InlineMarkdown text={section.footnote} />
        </Reveal>
      ) : null}
    </SectionShell>
  );
}
