import type { Site } from '../content/site';
import type { AnySection } from '../sections/schemas';
import type { MotionPreset } from '../config/schema';

export interface SectionProps<T extends AnySection = AnySection> {
  section: T;
  site: Site;
  defaultMotion: MotionPreset;
}

/**
 * Dispatches a validated section to its component via the registry (spec §5.3).
 * M1 renders a labelled placeholder; M3 attaches the real 12 components. Because
 * dispatch is registry-driven, that swap touches no code here.
 */
export function SectionRenderer({ section, site, defaultMotion }: SectionProps) {
  const motion = section.motion ?? defaultMotion;

  return (
    <section
      {...(section.id ? { id: section.id } : {})}
      className="zellij-section zellij-section-placeholder"
      data-zellij-section={section.type}
      data-zellij-motion={motion}
    >
      <div className="zellij-container">
        <p className="zellij-placeholder">
          <strong>{section.type}</strong>
          {section.id ? <span> · #{section.id}</span> : null}
          <span> — renders in M3</span>
        </p>
        <pre className="zellij-debug">{summarize(section, site)}</pre>
      </div>
    </section>
  );
}

/** A compact, readable preview of the validated section data. */
function summarize(section: AnySection, _site: Site): string {
  const entries = Object.entries(section).filter(([key]) => key !== 'type');
  return entries
    .map(([key, value]) => {
      if (typeof value === 'string') return `${key}: ${value}`;
      if (Array.isArray(value)) return `${key}: ${value.length} item(s)`;
      return `${key}: ${JSON.stringify(value)}`;
    })
    .join('\n');
}
