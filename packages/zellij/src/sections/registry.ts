import { z } from 'zod';
import { motionPresetSchema } from '../config/schema';

/**
 * Fields every section shares. `id` renders an anchor target so `/features#pricing`
 * works as a nav or CTA destination (spec §5.4).
 */
export const baseSectionSchema = z.object({
  type: z.string().min(1),
  id: z
    .string()
    .regex(/^[a-z0-9][a-z0-9-]*$/, 'must be a lowercase slug usable as a URL anchor')
    .optional(),
  motion: motionPresetSchema.optional(),
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
}

const registry = new Map<string, SectionDefinition>();

export function registerSection(definition: SectionDefinition): void {
  registry.set(definition.type, definition);
}

export function getSection(type: string): SectionDefinition | undefined {
  return registry.get(type);
}

export function registeredSectionTypes(): string[] {
  return [...registry.keys()].sort();
}

export function clearRegistry(): void {
  registry.clear();
}
