import { unified, type PluggableList } from 'unified';
import remarkParse from 'remark-parse';
import remarkMdx from 'remark-mdx';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeSlug from 'rehype-slug';
import { toString } from 'hast-util-to-string';
import type { Root, RootContent } from 'hast';

export interface Heading {
  id: string;
  text: string;
  depth: number;
  start: number;
  end: number;
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
  const processor = headingProcessor(options.remarkPlugins ?? []);
  const tree = processor.runSync(processor.parse(source));
  const headings: Heading[] = [];
  function visit(node: Root | RootContent): void {
    if (node.type === 'element' && /^h[1-6]$/.test(node.tagName)) {
      const start = node.position?.start.offset;
      const end = node.position?.end.offset;
      // Generated headings (e.g. the GFM footnote label) have no source passage.
      if (start !== undefined && end !== undefined) headings.push({
        id: String(node.properties.id), text: toString(node),
        depth: Number(node.tagName[1]), start, end,
      });
    }
    if ('children' in node) for (const child of node.children) visit(child);
  }
  visit(tree);
  return headings;
}
