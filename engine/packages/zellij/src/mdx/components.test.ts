import { renderToStaticMarkup } from 'react-dom/server';
import { compileMDX } from 'next-mdx-remote/rsc';
import { expect, it } from 'vitest';
import { mdxComponents } from './components';
import { mdxOptions } from './pipeline';
import { FolderContentSource, loadSite } from '../content';
import path from 'node:path';

it('renders imported Markdown images beneath the site basePath, preserving external images and alt text', async () => {
  const site = await loadSite(new FolderContentSource({
    contentDir: path.resolve('../../../examples/showcase/content'),
    basePath: '/products/docs',
  }));
  const { content } = await compileMDX({
    source: '![Local **diagram**](/_zellij/assets/imported/docs/a%20b.svg?view=1#icon)\n\n![Remote](https://example.com/a.png)',
    components: mdxComponents(site),
    options: await mdxOptions(),
  });
  const html = renderToStaticMarkup(content);
  expect(html).toContain('src="/products/docs/_zellij/assets/imported/docs/a%20b.svg?view=1#icon"');
  expect(html).toContain('alt="Local diagram"');
  expect(html).toContain('src="https://example.com/a.png"');
}, 30_000);
