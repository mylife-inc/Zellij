import type { SectionProps } from '../types';
import type { ShowcaseSection } from '../schemas';
import { CtaLink, InlineMarkdown, SectionImage, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { ScrollStage } from '../../motion/scroll-stage';

/** The Apple "learn more" building block (spec §5.2 #5). */
export function Showcase({ section, site, defaultMotion }: SectionProps<ShowcaseSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="showcase"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-showcase"
    >
      {/* Under `expressive` this pins the panel and ramps the visual as the
          page scrolls; under any other preset it is a plain block. Pinning is
          skipped without a visual — there would be nothing to ramp, and the
          reserved scroll distance would just be empty space. */}
      <ScrollStage
        motion={motion}
        enabled={Boolean(section.visual)}
        className="zellij-showcase-inner"
      >
        <Reveal motion={motion} className="zellij-showcase-copy">
          {section.eyebrow ? <p className="zellij-eyebrow">{section.eyebrow}</p> : null}
          <h2 {...gradientTextProps(section.titleGradient, 'zellij-showcase-headline')}>
            {section.headline}
          </h2>
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
              basePath={basePath}
              src={section.visual}
              alt=""
              ratio="16 / 9"
              sizes="(max-width: 900px) 100vw, 900px"
            />
          </Reveal>
        ) : null}
      </ScrollStage>
    </SectionShell>
  );
}
