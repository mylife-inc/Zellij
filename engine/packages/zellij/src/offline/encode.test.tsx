import { useId, type ReactElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { encode } from './encode';
import { decodeNode } from './decode';
import { setFileLocation } from './location';
import { CodeBlock } from '../mdx/code-block';
it('resolves async server content and preserves interactive component boundaries', async () => {
  setFileLocation({ route: '/', basePath: '', routes: ['/'] });
  async function Content() { return <article><CodeBlock><code>echo hello</code></CodeBlock></article>; }
  const wire = await encode(<Content />);
  expect(JSON.stringify(wire)).toContain('$CodeBlock');
  expect(renderToStaticMarkup(decodeNode(wire))).toContain('echo hello');
});
it('assigns distinct stable IDs to separate server components using hooks', async () => {
  function Label() { const id = useId(); return <span id={id}>Label</span>; }
  const wire = await encode([<Label key="first" />, <Label key="second" />]);
  const html = renderToStaticMarkup(decodeNode(wire));
  const ids = [...html.matchAll(/id="([^"]+)"/g)].map(match => match[1]);
  expect(new Set(ids).size).toBe(2);
  expect(renderToStaticMarkup(decodeNode(wire))).toBe(html);
});
it('rejects executable props instead of silently losing custom behaviour', async () => {
  await expect(encode(<div onClick={() => {}} />)).rejects.toThrow('serializable');
});

it('preserves absent keys separately from an authored literal "null" key', async () => {
  setFileLocation({ route: '/', basePath: '', routes: ['/'] });
  const original = [<h2>One heading</h2>, <p>One subheading</p>, <span key="null">Literal key</span>];
  const decoded = decodeNode(await encode(original)) as ReactElement[];
  expect(decoded.map(element => element.key)).toEqual(original.map(element => element.key));
  expect(decoded[0]!.key).toBeNull();
  expect(decoded[1]!.key).toBeNull();
  expect(decoded[2]!.key).toBe('null');
});
