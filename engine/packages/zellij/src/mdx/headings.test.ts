import { renderToStaticMarkup } from 'react-dom/server';
import { compileMDX } from 'next-mdx-remote/rsc';
import { expect, it } from 'vitest';
import { mdxOptions } from './pipeline';
import { readHeadings } from './headings';
import { extractToc } from '../guide/toc';
import { indexGuidePage } from '../search/build-index';

it.each([
  [
    '## [Install](/target) _now_ &amp; `cli`', '', 'First passage.', '',
    '## [Install](/different-target) _now_ &amp; `cli`', '', 'Second passage.', '',
    'Setext heading', '--------------', '', 'Setext passage.', '',
    '   ## Indented heading', '', 'Indented passage.', '',
    '> ## Quoted heading', '>', '> Quoted passage.', '',
    '````md', '```', '## Not a heading', '````', '',
    '## After the fence', '', 'Final passage.',
  ].join('\n'),
  '## $x_y_z$ and $\\sum_{k=1}^{n} k$\n\nA mathematical heading.\n',
])('keeps search and TOC IDs identical to rendered Markdown/MDX', async (source) => {
  const options = await mdxOptions();
  const headingOptions = { remarkPlugins: options.mdxOptions.remarkPlugins };
  const { content } = await compileMDX({ source, options });
  const html = renderToStaticMarkup(content);
  const ids = [...html.matchAll(/<h[23] id="([^"]*)"/g)].map((match) => match[1]);
  expect(ids.length).toBeGreaterThan(0);
  expect(extractToc(source, headingOptions).map((heading) => heading.id)).toEqual(ids);
  const entries = indexGuidePage({
    title: 'Test', route: '/manual', guide: '/manual', sourceFile: 'manual/index.mdx', body: source,
  }, headingOptions);
  expect(entries.slice(1).map((entry) => entry.route)).toEqual(ids.map((id) => `/manual#${id}`));
  expect(entries.map((entry) => entry.title)).not.toContain('Not a heading');
}, 30_000);

it('parses MDX expressions without evaluating authored JavaScript', () => {
  const source = 'export const value = (() => { throw new Error("must not run"); })();\n\n## Safe\n\nText.';
  expect(readHeadings(source).map((heading) => heading.id)).toEqual(['safe']);
});
