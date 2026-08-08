import type { SectionProps } from '../types';
import type { CtaBannerSection } from '../schemas';
import { CtaGroup, InlineMarkdown, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { CopyCommand } from './copy-command';

export function CtaBanner({ section, defaultMotion }: SectionProps<CtaBannerSection>) {
  const motion = section.motion ?? defaultMotion;

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
        <h2 {...gradientTextProps(section.titleGradient, 'zellij-cta-banner-headline')}>
          {section.headline}
        </h2>
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
