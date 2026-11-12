import { z } from 'zod';
import { baseSectionSchema } from './registry';
import { assetReferenceSchema, inlineReferenceSchema } from '../content/reference-schema';

/** Shared by the concise section and the interactive composition node. */
export const mediaAccordionFields = {
  label: z.string().min(1),
  mediaSide: z.enum(['start', 'end']).default('end'),
  ratio: z.enum(['landscape', 'square', 'portrait']).default('landscape'),
  fit: z.enum(['contain', 'cover']).default('contain'),
  itemHeadingLevel: z.number().int().min(2).max(4).default(3),
  items: z.array(z.object({
    title: z.string().min(1), body: inlineReferenceSchema.min(1),
    image: assetReferenceSchema.min(1), alt: z.string().min(1),
    caption: inlineReferenceSchema.optional(),
  })).min(2).max(8),
};
export const mediaAccordionNodeSchema = z.object({ type: z.literal('media-accordion'), ...mediaAccordionFields });
export const mediaAccordionSchema = baseSectionSchema.extend({
  type: z.literal('media-accordion'), ...mediaAccordionFields,
  eyebrow: z.string().optional(), heading: z.string().optional(), subhead: inlineReferenceSchema.optional(),
});
export type MediaAccordionNode = z.infer<typeof mediaAccordionNodeSchema>;
export type MediaAccordionSection = z.infer<typeof mediaAccordionSchema>;
