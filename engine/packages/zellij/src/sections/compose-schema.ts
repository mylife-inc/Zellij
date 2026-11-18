import { z } from 'zod';
import { baseSectionSchema } from './registry';
import { assetReferenceSchema, hrefReferenceSchema, inlineReferenceSchema, mdxReferenceSchema } from '../content/reference-schema';
import { satisfiesAuthoringRules } from './authoring-rules';
import { mediaAccordionNodeSchema } from './media-accordion-schema';
import { videoFields } from './video-schema';

const spacing = z.enum(['none', 'small', 'medium', 'large']);
const alignment = z.enum(['start', 'center', 'end', 'stretch']);
const layoutFields = { gap: spacing.optional(), align: alignment.optional() };
const placement = z.enum(['top-start', 'top-center', 'top-end', 'center-start', 'center', 'center-end', 'bottom-start', 'bottom-center', 'bottom-end']);
const transform = z.object({ y: z.number().min(-80).max(80).default(0), scale: z.number().min(0.85).max(1.15).default(1), opacity: z.number().min(0.5).max(1).default(1) });
export const composeScrollSchema = z.object({
  pin: z.boolean().default(true), distance: z.number().min(0.25).max(2).default(1),
  start: z.number().min(0).max(0.95).default(0), end: z.number().min(0.05).max(1).default(1),
  from: transform, to: transform,
});
export type ComposeScroll = z.infer<typeof composeScrollSchema>;
const backdropSchema = z.union([
  z.object({ kind: z.literal('image'), src: assetReferenceSchema.min(1), alt: z.string().default('') }),
  z.object({ kind: z.literal('video'), src: assetReferenceSchema.min(1), poster: assetReferenceSchema.min(1), label: z.string().min(1) }),
]);
export type ComposeBackdrop = z.infer<typeof backdropSchema>;
export const composeLayoutSchema = z.union([
  z.object({ kind: z.literal('stack'), ...layoutFields }),
  z.object({ kind: z.literal('grid'), columns: z.number().int().min(2).max(4).default(3), ...layoutFields }),
  z.object({ kind: z.literal('layers'), backdrop: backdropSchema, tone: z.enum(['dark', 'light']).default('dark'), scrim: z.number().min(0).max(1).default(0.65), height: z.enum(['compact', 'medium', 'tall']).default('medium'), ...layoutFields }),
  z.object({ kind: z.literal('split'), ratio: z.enum(['equal', 'wide-start', 'wide-end']).default('equal'), ...layoutFields }),
]);
export type ComposeLayout = z.infer<typeof composeLayoutSchema>;
const common = { place: placement.optional(), effect: z.enum(['none', 'reveal']).optional() };
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
  z.object({ type: z.literal('divider'), place: placement.optional() }),
  mediaAccordionNodeSchema.extend(common),
  z.object({ type: z.literal('video'), ...common, ...videoFields,
    label: z.string().min(1), caption: inlineReferenceSchema.optional() }),
  z.object({ type: z.literal('prose'), ...common, body: mdxReferenceSchema.min(1) }),
  z.object({ type: z.literal('list'), ...common, ordered: z.boolean().default(false), items: z.array(inlineReferenceSchema.min(1)).min(1).max(24) }),
  z.object({ type: z.literal('quote'), ...common, text: inlineReferenceSchema.min(1), author: z.string().min(1) }),
  z.object({ type: z.literal('icon'), ...common, name: z.string().min(1), label: z.string().min(1), size: z.enum(['small', 'medium', 'large']).default('medium') }),
  z.object({ type: z.literal('code'), ...common, title: z.string().min(1), code: z.string().min(1), language: z.string().regex(/^[a-zA-Z0-9_+-]+$/).optional() }),
  z.object({ type: z.literal('table'), ...common, caption: z.string().min(1), rowLabel: z.string().min(1), columns: z.array(z.string().min(1)).min(1).max(8), rows: z.array(z.object({ label: z.string().min(1), cells: z.array(inlineReferenceSchema).min(1).max(8) })).min(1).max(40) }),
  z.object({ type: z.literal('disclosure'), ...common, summary: z.string().min(1), body: inlineReferenceSchema.min(1), open: z.boolean().default(false) }),
  z.object({ type: z.literal('carousel'), ...common, label: z.string().min(1), items: z.array(z.object({ title: z.string().min(1), body: inlineReferenceSchema.optional(), image: assetReferenceSchema.optional(), alt: z.string().default(''), href: hrefReferenceSchema.optional() })).min(2).max(12) }),
  z.object({ type: z.literal('gallery'), ...common, label: z.string().min(1), columns: z.number().int().min(2).max(4).default(3), items: z.array(z.object({ image: assetReferenceSchema.min(1), alt: z.string().min(1), caption: z.string().optional() })).min(1).max(12) }),
] as const;
const leafSchema = z.union(leaves);
export type ComposeLeaf = z.infer<typeof leafSchema>;
const groupFields = z.object({ type: z.literal('group'), ...common,
  layout: composeLayoutSchema.optional(), scroll: composeScrollSchema.optional(),
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
  layout: composeLayoutSchema.optional(), scroll: composeScrollSchema.optional(), children: z.array(nodes(COMPOSE_GROUP_DEPTH)).min(1).max(24),
}).refine(value => satisfiesAuthoringRules('compose', value), {
  message: 'Invalid composition: check node/heading limits, layer placement, table cells, scroll ranges and nested stages.',
});
export type ComposeSection = z.infer<typeof composeSchema>;

export function composeHasPageHeading(section: Record<string, unknown>): boolean {
  const visit = (nodes: readonly ComposeNode[]): boolean => nodes.some(node =>
    node.type === 'heading' ? node.level === 1 : node.type === 'group' && visit(node.children));
  return visit(section.children as ComposeNode[]);
}
