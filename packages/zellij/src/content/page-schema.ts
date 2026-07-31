import { z } from 'zod';
import { motionPresetSchema, pageLayoutSchema, themeNameSchema } from '../config/schema';
import { getSection, registeredSectionTypes } from '../sections/registry';
import { ZellijContentError, formatZodError } from '../errors';
// Value import (not type-only) so the built-in section schemas actually register
// before any content is validated. A type-only import would be erased.
import { registerBuiltinSections, type AnySection } from '../sections/schemas';

registerBuiltinSections();

/** Page-level fields — spec §5.4. Sections are validated separately, per type. */
export const marketingPageFrontmatterSchema = z.object({
  title: z.string().min(1),
  description: z.string().min(1),
  route: z.string().optional(),
  layout: pageLayoutSchema.default('stack'),
  theme: themeNameSchema.optional(),
  motion: motionPresetSchema.optional(),
  navHighlight: z.string().optional(),
  /** Validated per-section by `validateSections`, so we accept unknown shapes here. */
  sections: z.array(z.record(z.string(), z.unknown())).default([]),
});

export type MarketingPageFrontmatter = z.infer<typeof marketingPageFrontmatterSchema>;

/**
 * Validates each section against its registered schema. An unknown `type` is a
 * build error listing every valid type (spec §5.3).
 */
export function validateSections(
  file: string,
  rawSections: ReadonlyArray<Record<string, unknown>>,
): AnySection[] {
  const out: AnySection[] = [];
  const problems: string[] = [];

  rawSections.forEach((raw, index) => {
    const type = raw['type'];

    if (typeof type !== 'string' || type.length === 0) {
      problems.push(`  · sections[${index}]: missing required "type" field`);
      return;
    }

    const definition = getSection(type);
    if (!definition) {
      problems.push(
        `  · sections[${index}]: unknown section type "${type}".\n` +
          `    Valid types: ${registeredSectionTypes().join(', ')}`,
      );
      return;
    }

    const result = definition.schema.safeParse(raw);
    if (!result.success) {
      const detail = formatZodError(`${file} · sections[${index}] (${type})`, result.error);
      problems.push(detail.message.split('\n').slice(1).join('\n'));
      return;
    }

    out.push(result.data as AnySection);
  });

  if (problems.length > 0) {
    throw new ZellijContentError(
      `Invalid sections in ${file}:\n${problems.join('\n')}`,
      file,
    );
  }

  return out;
}

/** The guide sidebar tree — spec §7.1. */
export type SidebarEntry = string | { label: string; items: SidebarEntry[] };

export const sidebarSchema: z.ZodType<SidebarEntry[]> = z.lazy(() =>
  z.array(
    z.union([
      z.string().min(1),
      z.object({
        label: z.string().min(1),
        items: sidebarSchema,
      }),
    ]),
  ),
);

/** Guide MDX frontmatter. Title falls back to the first H1 when omitted. */
export const guideFrontmatterSchema = z.object({
  title: z.string().min(1).optional(),
  description: z.string().optional(),
  route: z.string().optional(),
});
export type GuideFrontmatter = z.infer<typeof guideFrontmatterSchema>;
