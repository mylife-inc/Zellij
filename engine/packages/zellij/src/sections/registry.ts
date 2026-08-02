import { z } from 'zod';
import { motionPresetSchema } from '../config/schema';

/**
 * A section's surface: a named theme surface, or a gradient between two or
 * three palette colours. Named colours resolve against the active theme's
 * tokens, so a gradient follows the theme rather than pinning a literal hex.
 */
export const gradientStopsSchema = z.array(z.string().min(1)).min(2).max(3);

export const backgroundSchema = z.union([
  z.enum(['default', 'tinted', 'inverted', 'accent']),
  z.object({ gradient: gradientStopsSchema }),
]);
export type SectionBackground = z.infer<typeof backgroundSchema>;

/**
 * Fields every section shares. `id` renders an anchor target so `/features#pricing`
 * works as a nav or CTA destination (spec §5.4).
 *
 * These are also exactly the fields a page may override when it references a
 * section by name (design §6) — presentation, never content.
 */
export const baseSectionSchema = z.object({
  type: z.string().min(1),
  id: z
    .string()
    .regex(/^[a-z0-9][a-z0-9-]*$/, 'must be a lowercase slug usable as a URL anchor')
    .optional(),
  motion: motionPresetSchema.optional(),
  background: backgroundSchema.optional(),
  padding: z.enum(['none', 'tight', 'normal', 'loose']).optional(),
  /**
   * Paints the section's leading heading with a gradient. Same stop vocabulary
   * as `background`: palette token names follow the theme, literal colours do
   * not. Sections with no heading ignore it.
   */
  titleGradient: gradientStopsSchema.optional(),
});

/**
 * A section definition pairs a zod schema with the component that renders it.
 * Registering is how v2 (or a wrapper app) adds sections without touching
 * dispatch logic (spec §5.3).
 */
export interface SectionDefinition<TSchema extends z.ZodTypeAny = z.ZodTypeAny> {
  readonly type: string;
  readonly schema: TSchema;
  /** Rendering component. Attached in the sections barrel; loaders only need the schema. */
  readonly component?: unknown;
  /**
   * True when the section renders the page's `<h1>`. A page that stacks only
   * sections without one — a grid of promo tiles, say — would otherwise ship no
   * top-level heading at all, which costs it both the document outline screen
   * readers navigate by and the heading search engines read as the page title.
   */
  readonly providesPageHeading?: boolean;
}

const registry = new Map<string, SectionDefinition>();

export function registerSection(definition: SectionDefinition): void {
  registry.set(definition.type, definition);
}

export function getSection(type: string): SectionDefinition | undefined {
  return registry.get(type);
}

/** Section types that render the page `<h1>` themselves. */
export function pageHeadingSectionTypes(): Set<string> {
  return new Set(
    [...registry.values()].filter((def) => def.providesPageHeading).map((def) => def.type),
  );
}

export function registeredSectionTypes(): string[] {
  return [...registry.keys()].sort();
}

export function clearRegistry(): void {
  registry.clear();
}
