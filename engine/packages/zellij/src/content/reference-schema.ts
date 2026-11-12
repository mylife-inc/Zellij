import { z } from 'zod';

/** Reference semantics live beside validation, including in the generated schema. */
export const assetReferenceSchema = z.string().meta({ zellijReference: 'asset' });
export const hrefReferenceSchema = z.string().meta({ zellijReference: 'href' });
export const inlineReferenceSchema = z.string().meta({ zellijReference: 'inline' });
export const mdxReferenceSchema = z.string().meta({ zellijReference: 'mdx' });

export type ReferenceKind = 'asset' | 'href' | 'inline' | 'mdx';
export interface SchemaReference { kind: ReferenceKind; value: string; field: string }
type Shape = {
  const?: unknown;
  zellijReference?: ReferenceKind;
  properties?: Record<string, Shape>;
  items?: Shape;
  anyOf?: Shape[];
  oneOf?: Shape[];
};

/** Walk parsed values using the schema's annotations, never guessed field names. */
export function schemaReferences(schema: z.ZodType, value: unknown): SchemaReference[] {
  const json = z.toJSONSchema(schema, { io: 'output', unrepresentable: 'any' }) as Shape;
  const found = new Map<string, SchemaReference>();
  function walk(shape: Shape, input: unknown, field: string): void {
    if (input === undefined || input === null) return;
    // Parsed discriminated objects carry their literal selector. Inspect only
    // matching alternatives: a heading's plain `text` is not a text node's
    // inline Markdown, even though both fields have the same name.
    if (typeof input === 'object' && !Array.isArray(input) && shape.properties) {
      const object = input as Record<string, unknown>;
      if (Object.entries(shape.properties).some(([key, child]) =>
        Object.hasOwn(child, 'const') && Object.hasOwn(object, key) && object[key] !== child.const)) return;
    }
    if (shape.zellijReference && typeof input === 'string') {
      found.set(`${field}:${shape.zellijReference}`, { kind: shape.zellijReference, value: input, field });
    }
    if (Array.isArray(input) && shape.items) {
      input.forEach((item, index) => walk(shape.items!, item, `${field}[${index}]`));
    } else if (typeof input === 'object' && shape.properties) {
      for (const [name, child] of Object.entries(shape.properties)) {
        walk(child, (input as Record<string, unknown>)[name], field ? `${field}.${name}` : name);
      }
    }
    for (const option of shape.anyOf ?? shape.oneOf ?? []) walk(option, input, field);
  }
  walk(json, value, '');
  return [...found.values()];
}
