import { z } from 'zod';
import { assetReferenceSchema, hrefReferenceSchema, inlineReferenceSchema } from '../content/reference-schema';
import { videoFields } from './video-schema';

/** A finite, ordered narrative, not a general animation timeline. */
export const storyNodeSchema = z.object({
  type: z.literal('story'), label: z.string().min(1),
  sceneHeadingLevel: z.number().int().min(2).max(4).default(3),
  distance: z.number().min(0.5).max(2).default(1),
  mediaSide: z.enum(['start', 'end']).default('end'),
  scenes: z.array(z.object({
    heading: z.string().min(1), body: inlineReferenceSchema.min(1),
    media: z.union([
      z.object({ kind: z.literal('image'), src: assetReferenceSchema.min(1), alt: z.string().min(1) }),
      z.object({ kind: z.literal('video'), ...videoFields, poster: assetReferenceSchema.min(1), label: z.string().min(1),
        start: z.number().min(0).max(3600).default(0), end: z.number().min(0).max(3600) }),
    ]),
    cta: z.object({ label: z.string().min(1), href: hrefReferenceSchema.min(1), style: z.enum(['primary', 'secondary', 'ghost']).default('secondary') }).optional(),
  })).min(2).max(8),
});
export type StoryNode = z.infer<typeof storyNodeSchema>;
