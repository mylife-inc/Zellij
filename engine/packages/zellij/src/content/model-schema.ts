import { z } from 'zod';
import { motionPresetSchema, pageLayoutSchema, themeNameSchema } from '../config/schema';
import { getSection, registeredSectionTypes } from '../sections/registry';
import { ZellijContentError, formatZodError } from '../errors';
// Value import (not type-only) so the built-in section schemas actually register
// before any content is validated. A type-only import would be erased.
import { registerBuiltinSections, type AnySection } from '../sections/schemas';

registerBuiltinSections();

/* ------------------------------------------------------------------ menu.yaml */

/**
 * A menu entry targets exactly one of: an internal page, an external URL, or a
 * list of children. `page` is validated against the site's pages at build time;
 * `url` is not, since the engine has no way to check it.
 */
const menuTargetFields = {
  name: z.string().min(1),
  page: z.string().min(1).optional(),
  url: z.string().min(1).optional(),
  description: z.string().optional(),
  icon: z.string().optional(),
  /** External items open in a new tab by default; set false to navigate in place. */
  newTab: z.boolean().optional(),
};

const exactlyOneTarget = (item: { page?: string; url?: string; items?: unknown }): boolean =>
  [item.page, item.url, item.items].filter(Boolean).length === 1;

const TARGET_MESSAGE = 'a menu item must declare exactly one of: page, url, items';

export const menuLeafSchema = z
  .object(menuTargetFields)
  .refine((item) => exactlyOneTarget(item), { message: TARGET_MESSAGE });

/** A heading inside a drawer, grouping leaves under a label. */
export const menuGroupSchema = z.object({
  name: z.string().min(1),
  items: z.array(menuLeafSchema).min(1),
});

export const menuItemSchema = z
  .object({
    ...menuTargetFields,
    /** Render the drawer as a full-width panel rather than a compact dropdown. */
    wide: z.boolean().optional(),
    items: z.array(z.union([menuGroupSchema, menuLeafSchema])).min(1).optional(),
  })
  .refine((item) => exactlyOneTarget(item), { message: TARGET_MESSAGE });

export type AuthoredMenuItem = z.infer<typeof menuItemSchema>;

/** `menu.yaml` is a bare list; a `menu:` mapping is also accepted. */
export const menuFileSchema = z.union([
  z.array(menuItemSchema),
  z.object({ menu: z.array(menuItemSchema) }).transform((doc) => doc.menu),
]);

/* ----------------------------------------------------------------- pages.yaml */

/**
 * Presentation a page may override when it references a section — design §6.
 * Content fields are deliberately absent: a section file is the one place its
 * text lives, which is the whole reason sections are files.
 */
export const sectionOverrideSchema = z.object({
  id: z
    .string()
    .regex(/^[a-z0-9][a-z0-9-]*$/, 'must be a lowercase slug usable as a URL anchor')
    .optional(),
  background: z
    .union([
      z.enum(['default', 'tinted', 'inverted', 'accent']),
      z.object({ gradient: z.array(z.string().min(1)).min(2).max(3) }),
    ])
    .optional(),
  motion: motionPresetSchema.optional(),
  padding: z.enum(['none', 'tight', 'normal', 'loose']).optional(),
  titleGradient: z.array(z.string().min(1)).min(2).max(3).optional(),
});

export const sectionRefSchema = z.union([
  z.string().min(1),
  sectionOverrideSchema.extend({ section: z.string().min(1) }),
]);
export type SectionRef = z.infer<typeof sectionRefSchema>;

export const pageEntrySchema = z.object({
  name: z.string().min(1),
  title: z.string().min(1).optional(),
  description: z.string().optional(),
  route: z.string().optional(),
  layout: pageLayoutSchema.default('stack'),
  theme: themeNameSchema.optional(),
  motion: motionPresetSchema.optional(),
  navHighlight: z.string().optional(),
  sections: z.array(sectionRefSchema).default([]),
});
export type AuthoredPage = z.infer<typeof pageEntrySchema>;

export const pagesFileSchema = z.union([
  z.array(pageEntrySchema),
  z.object({ pages: z.array(pageEntrySchema) }).transform((doc) => doc.pages),
]);

/** Normalizes both reference forms to `{ section, ...overrides }`. */
export function normalizeSectionRef(ref: SectionRef): { section: string } & Record<string, unknown> {
  return typeof ref === 'string' ? { section: ref } : ref;
}

/* -------------------------------------------------------------- _sidebar.yaml */

/** The navigation tree inside a guide folder — spec §7.1. */
export type SidebarEntry = string | { label: string; items: SidebarEntry[] }
  | { label: string; page: string } | { label: string; href: string };

export const sidebarNavSchema: z.ZodType<SidebarEntry[]> = z.lazy(() =>
  z.array(
    z.union([
      z.string().min(1),
      z.object({ label: z.string().min(1), page: z.string().min(1) }).strict(),
      z.object({ label: z.string().min(1), href: z.url({ protocol: /^https?$/ }) }).strict(),
      z.object({
        label: z.string().min(1),
        items: sidebarNavSchema,
      }).strict(),
    ]),
  ),
);

/**
 * `_sidebar.yaml` in either form — design §8.
 *
 * The bare list is the original and stays valid. The mapping form exists so a
 * folder can carry its own label, ordering and route without depending on what
 * it happens to be called on disk.
 */
export const sidebarFileSchema = z.union([
  sidebarNavSchema.transform((nav) => ({ nav }) as AuthoredSidebar),
  z
    .object({
      title: z.string().min(1).optional(),
      order: z.number().optional(),
      route: z.string().min(1).optional(),
      description: z.string().optional(),
      nav: sidebarNavSchema.default([]),
    })
    .transform((doc) => doc as AuthoredSidebar),
]);

export interface AuthoredSidebar {
  title?: string;
  order?: number;
  route?: string;
  description?: string;
  nav: SidebarEntry[];
}

/* ------------------------------------------------------------- section files */

/** Frontmatter on a `.md`/`.mdx` section: presentation, plus an optional type. */
export const sectionFrontmatterSchema = sectionOverrideSchema.extend({
  type: z.string().min(1).optional(),
});

/** Guide page frontmatter. Title falls back to the first H1 when omitted. */
export const guideFrontmatterSchema = z.object({
  title: z.string().min(1).optional(),
  description: z.string().optional(),
  route: z.string().optional(),
});
export type GuideFrontmatter = z.infer<typeof guideFrontmatterSchema>;

/**
 * Validates one section against its registered schema. An unknown `type` is a
 * build error listing every valid type (spec §5.3).
 *
 * Keys the type does not declare pass through untouched: sections gain fields
 * far more often than the top-level schema does, and a content folder written
 * for a newer engine should still build on an older one (design §11).
 */
export function validateSection(where: string, raw: Record<string, unknown>): AnySection {
  const type = raw['type'];

  if (typeof type !== 'string' || type.length === 0) {
    throw new ZellijContentError(`${where}: missing required "type" field`, where);
  }

  const definition = getSection(type);
  if (!definition) {
    throw new ZellijContentError(
      `${where}: unknown section type "${type}".\n` +
        `  Valid types: ${registeredSectionTypes().join(', ')}`,
      where,
    );
  }

  const result = definition.schema.safeParse(raw);
  if (!result.success) throw formatZodError(`${where} (${type})`, result.error);
  return result.data as AnySection;
}

/**
 * Validates a list of inline sections, reporting every problem at once rather
 * than stopping at the first. Used by the legacy loader, where sections are
 * written inline in a page file.
 */
export function validateSections(
  file: string,
  rawSections: ReadonlyArray<Record<string, unknown>>,
): AnySection[] {
  const out: AnySection[] = [];
  const problems: string[] = [];

  rawSections.forEach((raw, index) => {
    try {
      out.push(validateSection(`${file} · sections[${index}]`, raw));
    } catch (err) {
      problems.push(`  · ${(err as Error).message.replace(/\n/g, '\n  ')}`);
    }
  });

  if (problems.length > 0) {
    throw new ZellijContentError(`Invalid sections in ${file}:\n${problems.join('\n')}`, file);
  }

  return out;
}
