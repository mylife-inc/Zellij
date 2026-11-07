import { unified, type PluggableList } from 'unified';
import remarkParse from 'remark-parse';
import remarkMdx from 'remark-mdx';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeSlug from 'rehype-slug';
import { toString } from 'hast-util-to-string';
import type { Root, RootContent } from 'hast';
import type { Root as MdastRoot, RootContent as MdastContent } from 'mdast';

export interface SourceRange {
  start: number;
  end: number;
}

export interface Heading {
  id: string;
  text: string;
  depth: number;
  start: number;
  end: number;
}

// remark-rehype passes literal JSX through to MDX's renderer. These nodes are
// not HAST elements, but an authored <h2 id="..."> is still a real heading.
interface LiteralHeading {
  type: string;
  name?: string;
  attributes?: Array<{ type: string; name?: string; value?: unknown }>;
}

function explicitHeading(node: Root | RootContent): { id: string; depth: number } | undefined {
  const jsx = node as LiteralHeading;
  if (!['mdxJsxFlowElement', 'mdxJsxTextElement'].includes(jsx.type)
    || !jsx.name || !/^h[1-6]$/.test(jsx.name)) return;
  // A spread or expression can override the ID. Do not execute it or guess.
  if (jsx.attributes?.some(attribute => attribute.type !== 'mdxJsxAttribute')) return;
  const ids = jsx.attributes?.filter(attribute => attribute.name === 'id');
  if (ids?.length !== 1 || typeof ids[0]!.value !== 'string' || !ids[0]!.value) return;
  return { id: ids[0]!.value, depth: Number(jsx.name[1]) };
}

// Parse, but never evaluate, MDX. Preserve MDX nodes just as its renderer does
// while converting Markdown headings to HTML and assigning rehype-slug IDs.
function headingProcessor(remarkPlugins: PluggableList) {
  return unified()
    .use(remarkParse)
    .use(remarkMdx)
    .use(remarkGfm)
    .use(remarkPlugins)
    .use(remarkRehype, {
      passThrough: ['mdxjsEsm', 'mdxFlowExpression', 'mdxJsxFlowElement', 'mdxJsxTextElement', 'mdxTextExpression'],
    })
    .use(rehypeSlug);
}

/** Shared by search and the TOC; IDs come from the renderer's slug plugin. */
export interface HeadingOptions {
  /** Extra parser plugins, such as the renderer's optional math parser. */
  remarkPlugins?: PluggableList;
}

/** Includes generated IDs such as GFM footnotes, using the same slug pass. */
export function readAnchorIds(source: string, options: HeadingOptions = {}): string[] {
  const processor = headingProcessor(options.remarkPlugins ?? []);
  const tree = processor.runSync(processor.parse(source));
  const ids: string[] = [];
  function visit(node: Root | RootContent): void {
    if (node.type === 'element' && typeof node.properties.id === 'string') ids.push(node.properties.id);
    if ('children' in node) for (const child of node.children) visit(child);
  }
  visit(tree);
  return ids;
}

export function readHeadings(source: string, options: HeadingOptions = {}): Heading[] {
  return readMdxStructure(source, options).headings;
}

/** Parse one complete document so search can use code and heading boundaries
 * without reparsing section slices that may contain only part of a component. */
export function readMdxStructure(source: string, options: HeadingOptions = {}): {
  headings: Heading[];
  codeRanges: SourceRange[];
} {
  const processor = headingProcessor(options.remarkPlugins ?? []);
  const parsed = processor.parse(source);
  const codeRanges: SourceRange[] = [];
  function captureCode(node: MdastRoot | MdastContent): void {
    if (node.type === 'code' || node.type === 'inlineCode') {
      const start = node.position?.start.offset;
      const end = node.position?.end.offset;
      if (start !== undefined && end !== undefined) codeRanges.push({ start, end });
      return;
    }
    if ('children' in node) for (const child of node.children) captureCode(child);
  }
  // Capture original syntax before plugins transform it into highlighted code,
  // diagrams or HTML. The same parser extensions are used for rendering/TOC.
  captureCode(parsed);
  const tree = processor.runSync(parsed);
  const headings: Heading[] = [];
  function visit(node: Root | RootContent): void {
    const heading = node.type === 'element' && /^h[1-6]$/.test(node.tagName)
      ? { id: String(node.properties.id), depth: Number(node.tagName[1]) }
      : explicitHeading(node);
    if (heading) {
      const start = node.position?.start.offset;
      const end = node.position?.end.offset;
      // Generated headings (e.g. the GFM footnote label) have no source passage.
      if (start !== undefined && end !== undefined) headings.push({
        ...heading, text: toString(node), start, end,
      });
    }
    if ('children' in node) for (const child of node.children) visit(child);
  }
  visit(tree);
  return { headings, codeRanges };
}
