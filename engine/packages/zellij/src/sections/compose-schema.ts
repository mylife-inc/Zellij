import { z } from 'zod';
import { baseSectionSchema } from './registry';
import { assetReferenceSchema, hrefReferenceSchema, inlineReferenceSchema } from '../content/reference-schema';
import { satisfiesAuthoringRules } from './authoring-rules';
import { mediaAccordionNodeSchema } from './media-accordion-schema';

const spacing = z.enum(['none', 'small', 'medium', 'large']);
const alignment = z.enum(['start', 'center', 'end', 'stretch']);
const layoutFields = { gap: spacing.optional(), align: alignment.optional() };
export const composeLayoutSchema = z.union([
  z.object({ kind: z.literal('stack'), ...layoutFields }),
  z.object({ kind: z.literal('grid'), columns: z.number().int().min(2).max(4).default(3), ...layoutFields }),
  z.object({ kind: z.literal('split'), ratio: z.enum(['equal', 'wide-start', 'wide-end']).default('equal'), ...layoutFields }),
]);
export type ComposeLayout = z.infer<typeof composeLayoutSchema>;
const common = { effect: z.enum(['none', 'reveal']).optional() };
const color = z.union([
  z.enum(['fg', 'muted', 'accent', 'accent-fg', 'inverted-fg']),
  z.string().regex(/^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/),
]);
const leaves = [
  z.object({ type: z.literal('heading'), ...common, text: z.string().min(1),
    level: z.number().int().min(1).max(4).default(2),
    scale: z.enum(['display', 'title', 'subtitle']).default('title'),
    weight: z.enum(['regular', 'semibold', 'bold']).default('bold'),
    gradient: z.object({ colors: z.array(color).min(2).max(3), effect: z.enum(['none', 'reveal']).default('none') }).optional() }),
  z.object({ type: z.literal('text'), ...common, text: inlineReferenceSchema.min(1),
    size: z.enum(['small', 'body', 'lead']).default('body'), tone: z.enum(['normal', 'muted']).default('normal') }),
  z.object({ type: z.literal('image'), ...common, image: assetReferenceSchema.min(1), alt: z.string().min(1),
    ratio: z.enum(['landscape', 'square', 'portrait']).default('landscape'), caption: inlineReferenceSchema.optional() }),
  z.object({ type: z.literal('button'), ...common, label: z.string().min(1), href: hrefReferenceSchema.min(1),
    style: z.enum(['primary', 'secondary', 'ghost']).default('primary') }),
  z.object({ type: z.literal('stat'), ...common, value: z.string().min(1), label: z.string().min(1) }),
  z.object({ type: z.literal('divider') }),
  mediaAccordionNodeSchema.extend(common),
] as const;
const leafSchema = z.union(leaves);
export type ComposeLeaf = z.infer<typeof leafSchema>;
const groupFields = z.object({ type: z.literal('group'), ...common,
  layout: composeLayoutSchema.optional(),
  surface: z.enum(['plain', 'tinted', 'inverted', 'accent']).default('plain'),
  padding: spacing.default('none'), radius: z.enum(['none', 'medium', 'large']).default('none') });
export type ComposeNode = ComposeLeaf | (z.infer<typeof groupFields> & { children: ComposeNode[] });

// Finite expansion is deliberate: one group branch per level, no $ref/$defs.
// Both the engine and native authoring consume the same bounded JSON schema.
export const COMPOSE_GROUP_DEPTH = 3;
function nodes(groups: number): z.ZodType<ComposeNode> {
  if (!groups) return leafSchema;
  return z.union([...leaves, groupFields.extend({ children: z.array(nodes(groups - 1)).min(1).max(24) })]);
}
export const composeSchema = baseSectionSchema.extend({
  type: z.literal('compose'), version: z.literal(1), label: z.string().min(1),
  layout: composeLayoutSchema.optional(), children: z.array(nodes(COMPOSE_GROUP_DEPTH)).min(1).max(24),
}).refine(value => satisfiesAuthoringRules('compose', value), {
  message: 'A composition supports at most 128 nodes and one level-1 heading.',
});
export type ComposeSection = z.infer<typeof composeSchema>;

export function composeHasPageHeading(section: Record<string, unknown>): boolean {
  const visit = (nodes: readonly ComposeNode[]): boolean => nodes.some(node =>
    node.type === 'heading' ? node.level === 1 : node.type === 'group' && visit(node.children));
  return visit(section.children as ComposeNode[]);
}
