import { z } from 'zod';
import { assetReferenceSchema, hrefReferenceSchema, inlineReferenceSchema } from '../content/reference-schema';

const item = z.object({
  id: z.string().regex(/^[a-z][a-z0-9-]*$/), label: z.string().min(1),
  heading: z.string().min(1), body: inlineReferenceSchema.min(1),
  media: z.object({ image: assetReferenceSchema.min(1), alt: z.string().min(1), caption: inlineReferenceSchema.optional() }).optional(),
  facts: z.array(z.object({ label: z.string().min(1), value: inlineReferenceSchema.min(1) })).min(1).max(8).optional(),
  cta: z.object({ label: z.string().min(1), href: hrefReferenceSchema.min(1), style: z.enum(['primary', 'secondary', 'ghost']).default('secondary') }).optional(),
});
const fields = {
  label: z.string().min(1), initial: z.string().min(1).optional(),
  itemHeadingLevel: z.number().int().min(2).max(4).default(3),
  mediaSide: z.enum(['start', 'end']).default('end'),
  ratio: z.enum(['landscape', 'square', 'portrait']).default('landscape'),
  fit: z.enum(['contain', 'cover']).default('contain'),
};
export const selectorNodeSchema = z.object({
  type: z.literal('selector'), ...fields,
  presentation: z.enum(['segmented', 'swatches']).default('segmented'),
  items: z.array(item.extend({swatch: z.string().regex(/^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/).optional()})).min(2).max(8),
});
export const explorerNodeSchema = z.object({
  type: z.literal('explorer'), ...fields,
  presentation: z.enum(['tabs', 'list']).default('tabs'),
  items: z.array(item).min(2).max(8),
});
export type ChoiceNode = z.infer<typeof selectorNodeSchema> | z.infer<typeof explorerNodeSchema>;
