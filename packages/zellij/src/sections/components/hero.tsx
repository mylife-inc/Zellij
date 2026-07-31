import type { SectionProps } from '../types';
import type { HeroSection } from '../schemas';
import { CtaGroup, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

export function Hero({ section, defaultMotion }: SectionProps<HeroSection>) {
  const motion = section.motion ?? defaultMotion;
  const hasVisual = Boolean(section.visual);

  return (
    <SectionShell
      type="hero"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-hero"
      full={section.visualStyle === 'full-bleed'}
    >
      <div
        className={section.visualStyle === 'full-bleed' ? 'zellij-container' : undefined}
        data-has-visual={hasVisual}
      >
        <div className="zellij-hero-copy">
          {section.badge ? (
            <Reveal motion={motion} as="p" className="zellij-badge">
              {section.badge}
            </Reveal>
          ) : null}

          <Reveal motion={motion} as="h1" className="zellij-hero-headline" index={section.badge ? 1 : 0}>
            {section.headline}
          </Reveal>

          {section.subhead ? (
            <Reveal motion={motion} as="p" className="zellij-hero-subhead" index={2}>
              {section.subhead}
            </Reveal>
          ) : null}

          {section.cta.length > 0 ? (
            <Reveal motion={motion} index={3}>
              <CtaGroup cta={section.cta} />
            </Reveal>
          ) : null}
        </div>
      </div>

      {section.visual ? (
        <Reveal
          motion={motion}
          index={4}
          className={`zellij-hero-visual zellij-hero-visual-${section.visualStyle}`}
        >
          <SectionImage
            src={section.visual}
            alt=""
            ratio="16 / 9"
            // The hero image is the LCP element on most landing pages.
            priority
            sizes="(max-width: 900px) 100vw, 1100px"
          />
        </Reveal>
      ) : null}
    </SectionShell>
  );
}
