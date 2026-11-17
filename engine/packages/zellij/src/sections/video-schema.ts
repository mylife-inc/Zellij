import { z } from 'zod';
import { assetReferenceSchema, hrefReferenceSchema } from '../content/reference-schema';

/** Shared by a full video section and a video inside a composition. */
export const videoFields = {
  src: assetReferenceSchema.min(1),
  poster: assetReferenceSchema.min(1).optional(),
  tracks: z.array(z.object({
    src: assetReferenceSchema.min(1),
    kind: z.enum(['captions', 'subtitles']).default('captions'),
    srclang: z.string().regex(/^[a-zA-Z]{2,8}(?:-[a-zA-Z0-9]{1,8})*$/),
    label: z.string().min(1),
  })).max(8).optional(),
  transcript: hrefReferenceSchema.min(1).optional(),
};
export type VideoTrack = NonNullable<z.infer<typeof videoFields.tracks>>[number];
