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

it.each(['', '/products/docs'])('renders public asset links under basePath %j with encoded filenames, queries and fragments intact', async (basePath) => {
  const site = await loadSite(new FolderContentSource({
    contentDir: path.resolve('../../../examples/showcase/content'),
    basePath,
  }));
  const { content } = await compileMDX({
    source: '[Notes](/_zellij/assets/imported/docs/a%20b.txt?raw=1#part)\n\n[Page](/guide/tables)\n\n[Section](#part)\n\n[Remote](https://example.com/notes.txt)',
    components: mdxComponents(site),
    options: await mdxOptions(),
  });
  const html = renderToStaticMarkup(content);
  expect(html).toContain(`href="${basePath}/_zellij/assets/imported/docs/a%20b.txt?raw=1#part"`);
  expect(html).toContain('href="/guide/tables"');
  expect(html).toContain('href="#part"');
  expect(html).toContain('href="https://example.com/notes.txt" target="_blank" rel="noreferrer noopener"');
}, 30_000);
