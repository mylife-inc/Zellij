import type { SectionProps } from '../types';
import type { ZigzagSection } from '../schemas';
import { CtaLink, InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

export function Zigzag({ section, defaultMotion }: SectionProps<ZigzagSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="zigzag"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-zigzag"
    >
      <SectionHeader {...(section.heading ? { heading: section.heading } : {})} motion={motion} />

      <div className="zellij-zigzag-rows">
        {section.rows.map((row, index) => (
          <div className="zellij-zigzag-row" data-align={row.align} key={row.title}>
            {/* Text and image translate from opposite sides, matching the row's
                alignment (spec §6.5). */}
            <Reveal
              motion={motion}
              className="zellij-zigzag-copy"
              direction={row.align === 'image-right' ? 'right' : 'left'}
            >
              <h3 className="zellij-zigzag-title">{row.title}</h3>
              {row.body ? (
                <p className="zellij-zigzag-body">
                  <InlineMarkdown text={row.body} />
                </p>
              ) : null}
              {row.cta ? <CtaLink cta={row.cta} className="zellij-zigzag-cta" /> : null}
            </Reveal>

            {row.image ? (
              <Reveal
                motion={motion}
                className="zellij-zigzag-media"
                direction={row.align === 'image-right' ? 'left' : 'right'}
              >
                <SectionImage
                  src={row.image}
                  alt=""
                  ratio="3 / 2"
                  sizes="(max-width: 900px) 100vw, 50vw"
                  {...(index === 0 ? {} : {})}
                />
              </Reveal>
            ) : null}
          </div>
        ))}
      </div>
    </SectionShell>
  );
}
