import { z } from 'zod';
import { logoStripSchema, featureGridSchema, statsBandSchema, pricingTableSchema, ctaBannerSchema, bentoGridSchema, timelineSchema, teamGridSchema } from './reusable-section-schemas';

// Composition owns the frame, anchor and motion. Content retains the built-in
// schema and reference annotations, with finite collection limits for embedding.
const frame = { id: true, background: true, motion: true, padding: true } as const;
export const reusedSectionSchema = z.union([
  logoStripSchema.omit(frame).extend({logos: logoStripSchema.shape.logos.max(16)}).strict(),
  featureGridSchema.omit(frame).extend({items: featureGridSchema.shape.items.max(12)}).strict(),
  statsBandSchema.omit(frame).strict(),
  pricingTableSchema.omit(frame).extend({tiers: z.array(pricingTableSchema.shape.tiers.element.extend({features: z.array(z.string()).max(24).default([])})).min(1).max(4)}).strict(),
  ctaBannerSchema.omit(frame).strict(),
  bentoGridSchema.omit(frame).extend({items: bentoGridSchema.shape.items.max(12)}).strict(),
  timelineSchema.omit(frame).extend({items: timelineSchema.shape.items.max(12)}).strict(),
  teamGridSchema.omit(frame).extend({people: z.array(teamGridSchema.shape.people.element.extend({links: z.array(teamGridSchema.shape.people.element.shape.links.unwrap().element).max(8).default([])})).min(1).max(12)}).strict(),
]);
export const sectionNodeSchema = z.object({
  type: z.literal('section'), label: z.string().min(1),
  headingLevel: z.number().int().min(2).max(4).default(2),
  content: reusedSectionSchema,
}).strict();
export type SectionNode = z.infer<typeof sectionNodeSchema>;
