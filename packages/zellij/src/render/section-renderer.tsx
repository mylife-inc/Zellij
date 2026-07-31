import type { Site } from '../content/site';
import type { AnySection } from '../sections/schemas';
import type { MotionPreset } from '../config/schema';
import { SECTION_COMPONENTS } from '../sections/components';
import { registeredSectionTypes } from '../sections/registry';
import { ZellijContentError } from '../errors';

export interface SectionRendererProps {
  section: AnySection;
  site: Site;
  defaultMotion: MotionPreset;
}

/**
 * Dispatches a validated section to its component through the registry, so v2
 * can add sections without touching this file (spec §5.3).
 *
 * Content is validated long before it reaches here, so an unknown type at this
 * point means a schema was registered without a component — an engine bug, not
 * an authoring mistake, and worth failing loudly.
 */
export function SectionRenderer({ section, site, defaultMotion }: SectionRendererProps) {
  const Component = SECTION_COMPONENTS[section.type];

  if (!Component) {
    throw new ZellijContentError(
      `No component registered for section type "${section.type}".\n` +
        `  Registered schemas: ${registeredSectionTypes().join(', ')}\n` +
        `  Components: ${Object.keys(SECTION_COMPONENTS).sort().join(', ')}`,
    );
  }

  return <Component section={section as never} site={site} defaultMotion={defaultMotion} />;
}
