import Image from 'next/image';
import type { SectionProps } from '../types';
import type { LogoStripSection } from '../schemas';
import { SectionShell } from '../shared';
import { assetUrl } from '../../content/paths';
import { Reveal } from '../../motion/reveal';

export function LogoStrip({ section, defaultMotion }: SectionProps<LogoStripSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="logo-strip"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-logo-strip"
    >
      {section.heading ? (
        <Reveal motion={motion} as="p" className="zellij-logo-strip-heading">
          {section.heading}
        </Reveal>
      ) : null}

      <ul className="zellij-logo-strip-list">
        {section.logos.map((logo, index) => (
          <Reveal as="li" key={logo} motion={motion} index={index} className="zellij-logo-item">
            <Image
              src={assetUrl(logo)}
              alt=""
              width={140}
              height={34}
              className="zellij-logo-image"
            />
          </Reveal>
        ))}
      </ul>
    </SectionShell>
  );
}
