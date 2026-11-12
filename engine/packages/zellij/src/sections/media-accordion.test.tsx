import { expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { mediaAccordionSchema } from './media-accordion-schema';
import { composeSchema } from './compose-schema';
import { schemaReferences } from '../content/reference-schema';
import { validateSection } from '../content/model-schema';
import { MediaAccordion } from './components/media-accordion';
import { Compose } from './components/compose';
import type { Site } from '../content/site';

const item = { title: 'A useful detail', body: 'Read **carefully**, then [continue](/guide).', image: 'assets/detail.svg', alt: 'A folded sheet', caption: 'The [full story](/story).' };
const input = { type: 'media-accordion', label: 'Details', items: [item, { ...item, title: 'A second perspective' }] };
const site = { config: { basePath: '/product' } } as Site;

it('shares bounded items and reference validation between the section and composed node', () => {
  const section = mediaAccordionSchema.parse(input);
  const composed = composeSchema.parse({ type: 'compose', version: 1, label: 'Story', children: [input] });
  expect(composed.children[0]).toEqual(section);
  for (const items of [[], [item], Array.from({ length: 9 }, () => item)]) {
    expect(mediaAccordionSchema.safeParse({ ...input, items }).success).toBe(false);
    expect(composeSchema.safeParse({ ...composed, children: [{ ...input, items }] }).success).toBe(false);
  }
  expect(mediaAccordionSchema.safeParse({ ...input, itemHeadingLevel: 2.5 }).success).toBe(false);
  expect(schemaReferences(composeSchema, composed)).toEqual(expect.arrayContaining([
    { kind: 'asset', value: 'assets/detail.svg', field: 'children[0].items[0].image' },
    { kind: 'inline', value: item.body, field: 'children[0].items[0].body' },
    { kind: 'inline', value: item.caption, field: 'children[0].items[0].caption' },
  ]));
});

it('reports unsupported item fields at their authored location', () => {
  const warnings: string[] = [];
  validateSection('details.yaml', { ...input, items: [{ ...item, animationCode: 'never-print-this' }, item] }, text => warnings.push(text));
  expect(warnings).toHaveLength(1);
  expect(warnings[0]).toContain('items[0].animationCode');
  expect(warnings[0]).not.toContain('never-print-this');
});

it('server renders every story and image in order without inactive controls or hidden content', () => {
  const section = mediaAccordionSchema.parse({ ...input, heading: 'A closer look' });
  const html = renderToStaticMarkup(<MediaAccordion section={section} site={site} defaultMotion="none" />);
  expect(html).toContain('role="group" aria-label="Details"');
  expect(html).toContain('<strong>carefully</strong>');
  expect(html).toContain('/product/_zellij/assets/detail.svg');
  expect(html).toContain('<h2');
  expect(html).not.toContain('<button');
  expect(html).not.toContain(' hidden=');
  expect(html.match(/alt="A folded sheet"/g)).toHaveLength(2);
  expect(html.match(/loading="eager"/g)).toHaveLength(2);
  expect(html.indexOf('A useful detail')).toBeLessThan(html.indexOf('A second perspective'));
});

it('multiple composed accordions have independent control identities and preserve surrounding content', () => {
  const section = composeSchema.parse({ type: 'compose', version: 1, label: 'Together', children: [input, { type: 'text', text: 'Between the stories.' }, input] });
  const html = renderToStaticMarkup(<Compose section={section} site={site} defaultMotion="none" />);
  const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
  expect(new Set(ids).size).toBe(ids.length);
  expect(ids.length).toBeGreaterThan(4);
  expect(html).toContain('Between the stories.');
});

it('renders authored media placement, aspect ratio, fit and heading level', () => {
  const section = mediaAccordionSchema.parse({ ...input, mediaSide: 'start', ratio: 'portrait', fit: 'cover', itemHeadingLevel: 4 });
  const html = renderToStaticMarkup(<MediaAccordion section={section} site={site} defaultMotion="none" />);
  expect(html).toContain('data-media-side="start"');
  expect(html).toContain('aspect-ratio:3 / 4');
  expect(html.match(/data-fit="cover"/g)).toHaveLength(2);
  expect(html.match(/<h4\b/g)).toHaveLength(2);
});
