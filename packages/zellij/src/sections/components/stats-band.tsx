import type { SectionProps } from '../types';
import type { StatsBandSection } from '../schemas';
import { SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { CountUp } from '../../motion/count-up';

export function StatsBand({ section, defaultMotion }: SectionProps<StatsBandSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="stats-band"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background="tinted"
      className="zellij-stats"
    >
      <SectionHeader {...(section.heading ? { heading: section.heading } : {})} motion={motion} />

      <dl className="zellij-stats-list" data-count={section.stats.length}>
        {section.stats.map((stat, index) => (
          <Reveal motion={motion} index={index} className="zellij-stat" key={stat.label}>
            <dt className="zellij-stat-value">
              {/* Counts up only under `expressive`; otherwise renders the
                  literal value (spec §6.5). */}
              <CountUp
                value={stat.value}
                motion={motion}
                {...(stat.prefix ? { prefix: stat.prefix } : {})}
                {...(stat.suffix ? { suffix: stat.suffix } : {})}
              />
            </dt>
            <dd className="zellij-stat-label">{stat.label}</dd>
          </Reveal>
        ))}
      </dl>
    </SectionShell>
  );
}
