import type { SectionProps } from '../types';
import type { CtaBannerSection } from '../schemas';
import { CtaGroup, InlineMarkdown, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { CopyCommand } from './copy-command';

export function CtaBanner({ section, defaultMotion, headingLevel }: SectionProps<CtaBannerSection>) {
  const motion = section.motion ?? defaultMotion;
  const Heading = `h${headingLevel ?? 2}` as 'h2' | 'h3' | 'h4';

  // The schema accepts a single CTA or a list; normalise to a list.
  const actions = section.cta ? (Array.isArray(section.cta) ? section.cta : [section.cta]) : [];

  return (
    <SectionShell
      type="cta-banner"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-cta-banner"
    >
      <Reveal motion={motion} className="zellij-cta-banner-inner">
        <Heading {...gradientTextProps(section.titleGradient, 'zellij-cta-banner-headline')}>
          {section.headline}
        </Heading>
        {section.body ? (
          <p className="zellij-cta-banner-body">
            <InlineMarkdown text={section.body} />
          </p>
        ) : null}
        {section.command ? <CopyCommand command={section.command} /> : null}
        <CtaGroup cta={actions} />
      </Reveal>
    </SectionShell>
  );
}
