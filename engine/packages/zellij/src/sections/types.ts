import type { MotionPreset } from '../config/schema';
import type { Site } from '../content/site';
import type { AnySection } from './schemas';

export interface SectionProps<T extends AnySection = AnySection> {
  section: T;
  site: Site;
  /** Page-level motion preset; a section may override it in its own YAML. */
  defaultMotion: MotionPreset;
}
