import { writeFile } from 'node:fs/promises';
import { z } from 'zod';
import { buildSectionTemplates } from './section-templates';

import { siteConfigSchema } from '../config/schema';
import { menuItemSchema, pageEntrySchema, sidebarFileSchema } from '../content/model-schema';
import { getSection, registeredSectionTypes } from '../sections/registry';
import '../sections/schemas';

/**
 * Writes a single self-contained description of Zellij's content model, for an
 * AI agent to work from.
 *
 * Generated rather than written. An agent given a hand-maintained reference
 * produces confident, wrong YAML the moment a section gains a field — and
 * because it looks right, the error surfaces as a failed build in someone
 * else's repository rather than as an obviously stale document. Everything a
 * schema already knows is read back out of the schema.
 */

type JsonSchema = {
  type?: string | string[];
  properties?: Record<string, JsonSchema>;
  required?: string[];
  items?: JsonSchema;
  anyOf?: JsonSchema[];
  oneOf?: JsonSchema[];
  enum?: unknown[];
  const?: unknown;
  default?: unknown;
  description?: string;
  minItems?: number;
  maxItems?: number;
  minimum?: number;
  maximum?: number;
  additionalProperties?: boolean | JsonSchema;
};

function toJson(schema: z.ZodType): JsonSchema {
  // `io: 'input'` describes what an author writes rather than what the loader
  // hands back: defaults are optional on the way in and present on the way out,
  // and it is the way in that a content file has to satisfy.
  return z.toJSONSchema(schema, { io: 'input', unrepresentable: 'any' }) as JsonSchema;
}

/** A one-line type for a field, in the vocabulary a YAML author thinks in. */
function describe(schema: JsonSchema | undefined): string {
  if (!schema) return 'any';

  if (schema.const !== undefined) return `\`${JSON.stringify(schema.const)}\``;
  if (schema.enum) return schema.enum.map((value) => `\`${String(value)}\``).join(' \\| ');

  if (schema.anyOf ?? schema.oneOf) {
    const options = (schema.anyOf ?? schema.oneOf ?? []).filter(
      (option) => option.type !== 'null',
    );
    const described = [...new Set(options.map((option) => describe(option)))];
    return described.join(' \\| ');
  }

  if (schema.type === 'array') {
    const bounds =
      schema.minItems || schema.maxItems
        ? ` (${[
            schema.minItems ? `min ${schema.minItems}` : '',
            schema.maxItems ? `max ${schema.maxItems}` : '',
          ]
            .filter(Boolean)
            .join(', ')})`
        : '';
    return `list of ${describe(schema.items)}${bounds}`;
  }

  if (schema.type === 'object') {
    const keys = Object.keys(schema.properties ?? {});
    return keys.length > 0 ? `object { ${keys.join(', ')} }` : 'object';
  }

  if (Array.isArray(schema.type)) return schema.type.filter((t) => t !== 'null').join(' \\| ');
  return schema.type ?? 'any';
}

/** The object schema behind a field, if it has one worth expanding. */
function objectBehind(schema: JsonSchema): JsonSchema | undefined {
  if (schema.type === 'object' && schema.properties) return schema;
  if (schema.type === 'array' && schema.items) return objectBehind(schema.items);

  for (const option of schema.anyOf ?? schema.oneOf ?? []) {
    const found = objectBehind(option);
    if (found) return found;
  }

  return undefined;
}

/**
 * A markdown table of a schema's fields, followed by a table for each nested
 * object it contains.
 *
 * The nesting is the point. `items` on a promo grid *is* the section, and a row
 * reading "list of object { eyebrow, title, … }" tells an agent the names of
 * the keys and nothing about what may go in them — which is exactly enough to
 * write something plausible and invalid.
 */
function fieldTable(
  schema: JsonSchema,
  skip: readonly string[] = [],
  prefix = '',
  depth = 0,
): string {
  const properties = schema.properties ?? {};
  const required = new Set(schema.required ?? []);
  const nested: Array<[string, JsonSchema]> = [];

  const rows = Object.entries(properties)
    .filter(([name]) => !skip.includes(name))
    .map(([name, field]) => {
      const flag = required.has(name) ? '**required**' : '';
      const fallback =
        field.default !== undefined ? `default \`${JSON.stringify(field.default)}\`` : '';
      const notes = [flag, fallback].filter(Boolean).join(', ');

      const inner = objectBehind(field);
      if (inner && depth < 2) nested.push([name, inner]);

      return `| \`${name}\` | ${describe(field)} | ${notes} |`;
    });

  if (rows.length === 0) return '_No fields._\n';

  const table = ['| field | type | |', '| --- | --- | --- |', ...rows].join('\n') + '\n';

  const expansions = nested.map(([name, inner]) => {
    const path = prefix ? `${prefix}.${name}` : name;
    return `\n**\`${path}\`**\n\n${fieldTable(inner, [], path, depth + 1)}`;
  });

  return table + expansions.join('');
}

/** Fields every section shares, documented once rather than in all nineteen tables. */
const BASE_FIELDS = ['type', 'id', 'motion', 'background', 'padding', 'titleGradient'];

function sectionReference(): string {
  const out: string[] = [];

  for (const type of registeredSectionTypes()) {
    const definition = getSection(type);
    if (!definition) continue;

    out.push(`### \`${type}\`\n`);
    out.push(fieldTable(toJson(definition.schema as z.ZodType), BASE_FIELDS));
    out.push('');
  }

  return out.join('\n');
}

const PREAMBLE = `# Zellij content model — reference for an AI agent

Zellij turns a folder of structured content into a complete website: marketing
pages built from typed sections, and documentation built from MDX. You write
YAML and MDX; there is no code to write and no template to fill in.

Your job, given a codebase or an existing set of documentation, is to produce a
**content folder** that Zellij can build. Nothing else — no React, no Next.js
app, no configuration beyond the files described here.

## Rules that matter more than the details

1. **Never invent a field or a section type.** Anything absent from this
   document will fail the build. The valid section types are listed below and
   that list is complete.
2. **A section file holds its own text.** Pages reference sections by name and
   may override presentation, never content.
3. **Every internal link must resolve.** Links are validated at build time and a
   broken one fails the build.
4. **Validate before you finish** by running \`zellij check <folder>\`. It names
   the file behind any problem. Treat a clean run as the definition of done.

## The folder

    content/
    ├── site.yaml          identity, theme, nav and footer          (optional)
    ├── menu.yaml          the top navigation                       (optional)
    ├── pages.yaml         marketing pages, each a list of sections (optional)
    ├── sections/          one file per section, referenced by path
    ├── examples/          sections shown in docs as source + preview (optional)
    ├── assets/            images and video, referenced as assets/…
    └── <guide folders>/   any folder holding a _sidebar.yaml

Every file is optional. A folder containing nothing but one guide folder builds
a complete site.

### Naming and routes

- A guide folder mounts at its own name. A leading number is an ordering hint
  and is stripped: \`01-guide/\` serves at \`/guide\`.
- \`index.mdx\` is the folder's own page; \`quickstart.mdx\` becomes
  \`/guide/quickstart\`.
- A section is referenced by its path below \`sections/\` without the
  extension: \`sections/home/hero.yaml\` is \`home/hero\`.
`;

const AUTHORING = `## Guide pages

A guide folder is any folder containing \`_sidebar.yaml\`. Pages are \`.mdx\`
files with frontmatter:

\`\`\`mdx
---
title: Quickstart
description: Get running in ten minutes.
---

## A heading

Prose, with the components below available without importing them.
\`\`\`

### Components available in MDX

\`<Callout type="info|tip|warning|danger" title="…">\`, \`<Steps>\` / \`<Step>\`,
\`<Tabs>\` / \`<Tab label="…">\`, \`<CodeGroup>\`, \`<Card>\` / \`<CardGrid>\`,
\`<Screenshot src="assets/…" alt="…">\`, \`<SectionPreview example="…">\`,
\`<Theorem kind="…" number="…" title="…">\` and \`<Proof>\`.

Fenced code blocks take \`title="file.ts"\` and \`{2,5-6}\` line highlighting.
LaTeX goes between \`$…$\` inline or \`$$…$$\` display. A \`mermaid\` fence is
drawn as a diagram.

### Writing style for generated documentation

Write what the code does and why, not what its identifiers are called. A page
that restates the function signature adds nothing a reader could not get from
the file. Prefer a short page that answers one question to a long page that
lists everything.
`;

const CLOSING = `## A minimal, complete example

\`\`\`yaml
# content/site.yaml
name: Acme
theme: cupertino
home: home
\`\`\`

\`\`\`yaml
# content/pages.yaml
- name: home
  title: Acme
  description: One sentence about the product.
  layout: landing
  sections: [home/hero]
\`\`\`

\`\`\`yaml
# content/sections/home/hero.yaml
type: hero
headline: Ship documentation with the product
subhead: A folder of content, and the site builds itself.
cta:
  - { label: Read the guide, href: /guide, style: primary }
\`\`\`

\`\`\`yaml
# content/01-guide/_sidebar.yaml
title: Guide
nav:
  - index
  - quickstart
\`\`\`

That is a complete site: a landing page and a guide.

## When you are done

Run \`zellij check <folder> --deep --strict\` to check static references and
MDX syntax and fail on warnings. Then build the site. Checking does not execute
MDX or prove that every component, expression, math formula or diagram renders.
`;

/** Assembles the whole document. */
export function buildContext(): string {
  const sidebar = toJson(sidebarFileSchema as unknown as z.ZodType);
  // `_sidebar.yaml` is a union of "bare list" and "mapping"; the mapping is the
  // half with fields worth tabulating.
  const sidebarMapping =
    (sidebar.anyOf ?? sidebar.oneOf ?? []).find((option) => option.type === 'object') ?? sidebar;

  return [
    PREAMBLE,
    '## `site.yaml`\n',
    fieldTable(toJson(siteConfigSchema as unknown as z.ZodType)),
    '\n## `menu.yaml`\n',
    'A list of items. Each names exactly one target — `page`, `url` or `href` —',
    'or carries `items` to become a dropdown. A child with its own `items` makes',
    'the parent a full-width panel.\n',
    fieldTable(toJson(menuItemSchema as unknown as z.ZodType)),
    '\n## `pages.yaml`\n',
    'A list of pages. `sections` names files below `sections/`, in the order they',
    'should appear. An entry may be a plain name, or a mapping of `section:` plus',
    'presentation overrides.\n',
    fieldTable(toJson(pageEntrySchema as unknown as z.ZodType)),
    '\n## `_sidebar.yaml`\n',
    'Either a bare list of entries, or a mapping with the fields below. An entry',
    'is a page name, or `{ label, items }` to group — groups nest to any depth.\n',
    fieldTable(sidebarMapping),
    '\n' + AUTHORING,
    `\n## Section types\n\nThere are ${registeredSectionTypes().length}. Every section also accepts the shared fields \`id\` (URL anchor), \`background\`, \`padding\`, \`motion\` and \`titleGradient\`.\n`,
    sectionReference(),
    CLOSING,
  ].join('\n');
}

/**
 * The same knowledge as the prose, in a form a program can use.
 *
 * `zel explain` and `zel new section` have to know a section's fields to
 * describe or scaffold one, and the CLI is a Rust binary that cannot run a zod
 * schema. This is emitted here, committed, and compiled into that binary — so
 * the tool that scaffolds a section and the engine that validates it are
 * reading the same definition rather than two that agree until one changes.
 */
export function buildSchemaJson(): string {
  const sections = Object.fromEntries(
    registeredSectionTypes().map((type) => {
      const definition = getSection(type);
      return [type, definition ? toJson(definition.schema as z.ZodType) : {}];
    }),
  );

  return `${JSON.stringify(
    {
      $comment:
        'Generated by `zellij context --json`. Do not edit — regenerate from the schemas.',
      sectionTypes: registeredSectionTypes(),
      sectionTemplates: buildSectionTemplates(),
      sections,
      files: {
        'site.yaml': toJson(siteConfigSchema as unknown as z.ZodType),
        'menu.yaml': toJson(menuItemSchema as unknown as z.ZodType),
        'pages.yaml': toJson(pageEntrySchema as unknown as z.ZodType),
        '_sidebar.yaml': toJson(sidebarFileSchema as unknown as z.ZodType),
      },
    },
    null,
    2,
  )}\n`;
}

export async function writeContext(target?: string, format: 'markdown' | 'json' = 'markdown'): Promise<string> {
  const body = format === 'json' ? buildSchemaJson() : buildContext();

  if (target) {
    await writeFile(target, body, 'utf8');
    return target;
  }

  process.stdout.write(body);
  return '(stdout)';
}
