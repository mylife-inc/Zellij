import type { SectionProps } from '../types';
import type { ShowcaseSection } from '../schemas';
import { CtaLink, InlineMarkdown, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/** The Apple "learn more" building block (spec §5.2 #5). */
export function Showcase({ section, defaultMotion }: SectionProps<ShowcaseSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="showcase"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      className="zellij-showcase"
    >
      <div className="zellij-showcase-inner">
        <Reveal motion={motion} className="zellij-showcase-copy">
          {section.eyebrow ? <p className="zellij-eyebrow">{section.eyebrow}</p> : null}
          <h2 className="zellij-showcase-headline">{section.headline}</h2>
          {section.body ? (
            <p className="zellij-showcase-body">
              <InlineMarkdown text={section.body} />
            </p>
          ) : null}
          {section.cta ? <CtaLink cta={section.cta} /> : null}
        </Reveal>

        {section.visual ? (
          <Reveal motion={motion} index={1} className="zellij-showcase-media">
            <SectionImage
              src={section.visual}
              alt=""
              ratio="16 / 9"
              sizes="(max-width: 900px) 100vw, 900px"
            />
          </Reveal>
        ) : null}
      </div>
    </SectionShell>
  );
}
