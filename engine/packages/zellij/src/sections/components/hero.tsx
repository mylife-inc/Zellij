import type { SectionProps } from '../types';
import type { HeroSection } from '../schemas';
import { CtaGroup, SectionImage, SectionShell, gradientTextProps } from '../shared';
import { Reveal } from '../../motion/reveal';
import { Parallax } from '../../motion/scroll-stage';

export function Hero({ section, site, defaultMotion }: SectionProps<HeroSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;
  const hasVisual = Boolean(section.visual);

  return (
    <SectionShell
      type="hero"
      background={section.background}
      padding={section.padding}
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

          <Reveal
            motion={motion}
            as="h1"
            index={section.badge ? 1 : 0}
            {...gradientTextProps(section.titleGradient, 'zellij-hero-headline')}
          >
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
          {/* Gentle parallax under `expressive`; inert otherwise. */}
          <Parallax motion={motion} className="zellij-hero-parallax">
            <SectionImage
              basePath={basePath}
              src={section.visual}
              alt=""
              ratio="16 / 9"
              // The hero image is the LCP element on most landing pages.
              priority
              sizes="(max-width: 900px) 100vw, 1100px"
            />
          </Parallax>
        </Reveal>
      ) : null}
    </SectionShell>
  );
}
