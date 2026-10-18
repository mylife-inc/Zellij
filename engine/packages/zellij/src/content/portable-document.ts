import { z } from 'zod';
import type { Element, Root, Text } from 'hast';

/** Internal bridge emitted by the native parser, never executable MDX. Only
 * inert Markdown nodes/properties are accepted before the rendering plugins run. */
export type PortableNode = Text | (Omit<Element, 'children'> & { children: PortableNode[] });
const url = z.string().refine((value) =>
  !/[\u0000-\u0020\\]/.test(value) &&
  (value.startsWith('/') && !value.startsWith('//') || value.startsWith('#') || value.startsWith('?') || /^(https?:|mailto:|tel:)/i.test(value)),
'expected a safe absolute URL, site path, or fragment');
const properties = z.object({
  id: z.string().min(1).optional(),
  href: url.optional(), src: url.optional(),
  title: z.string().optional(), alt: z.string().optional(),
  className: z.array(z.string()).optional(),
  start: z.number().int().nonnegative().optional(),
  align: z.enum(['left', 'center', 'right']).optional(),
  type: z.literal('checkbox').optional(),
  checked: z.boolean().optional(), disabled: z.literal(true).optional(),
  dataLang: z.string().optional(),
}).strict();
const node: z.ZodType<PortableNode> = z.lazy(() => z.union([
  z.object({ type: z.literal('text'), value: z.string() }).strict(),
  z.object({
    type: z.literal('element'),
    tagName: z.enum(['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'blockquote', 'ol', 'ul', 'li', 'div', 'sup', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'em', 'strong', 'del', 'a', 'img', 'pre', 'code', 'br', 'hr', 'input']),
    properties, children: z.array(node),
  }).strict(),
]));
export const portableDocumentSchema = z.object({
  version: z.literal(1),
  title: z.string().min(1), description: z.string(),
  route: z.string().startsWith('/'), text: z.string(),
  headings: z.array(z.object({ id: z.string().min(1), text: z.string(), depth: z.number().int().min(1).max(6) }).strict()),
  body: z.object({ type: z.literal('root'), children: z.array(node) }).strict(),
}).strict();
export type PortableDocument = z.infer<typeof portableDocumentSchema>;

export function walkPortable(tree: Root | PortableNode, visit: (node: PortableNode) => void): void {
  if (tree.type !== 'root') visit(tree);
  if ('children' in tree) for (const child of tree.children) walkPortable(child as PortableNode, visit);
}
