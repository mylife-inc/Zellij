import type { SectionProps } from '../types';
import type { LogoStripSection } from '../schemas';
import { SectionShell } from '../shared';
import { assetUrl, withBasePath } from '../../content/paths';
import { Reveal } from '../../motion/reveal';

export function LogoStrip({ section, site, defaultMotion }: SectionProps<LogoStripSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="logo-strip"
      background={section.background}
      padding={section.padding}
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
            {/*
              A plain <img>, not next/image, and deliberately so. Logos are
              typically SVG, which next/image leaves unoptimized — and for
              unoptimized images it emits `src` verbatim, without the basePath
              prefix, so they 404 on a reverse-proxied deployment. Optimization
              would buy nothing for a 2KB vector anyway.
              eslint-disable-next-line @next/next/no-img-element
            */}
            <img
              src={withBasePath(assetUrl(logo), basePath)}
              alt=""
              width={140}
              height={34}
              loading="lazy"
              decoding="async"
              className="zellij-logo-image"
            />
          </Reveal>
        ))}
      </ul>
    </SectionShell>
  );
}
