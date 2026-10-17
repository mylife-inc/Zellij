import { renderToStaticMarkup } from 'react-dom/server';
import { compileMDX } from 'next-mdx-remote/rsc';
import { expect, it } from 'vitest';
import { mdxOptions, supportsMaths } from './pipeline';

it('renders inline and display formulas through the installed math dependencies', async () => {
  expect(await supportsMaths()).toBe(true);
  const { content } = await compileMDX({
    source: 'Inline $x^2$ and a fraction:\n\n$$\n\\frac{a}{b}\n$$',
    options: await mdxOptions(),
  });
  const html = renderToStaticMarkup(content);
  expect(html.match(/<mjx-container/g)).toHaveLength(2);
  expect(html).toContain('<svg');
  expect(html).not.toContain('data-mjx-error');
}, 30_000);
