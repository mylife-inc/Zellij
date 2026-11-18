import { expect, it } from 'vitest';
import { composeSchema } from './compose-schema';
import { schemaReferences } from '../content/reference-schema';

const items = [
  {id: 'silver', label: 'Silver', heading: 'Quiet clarity.', body: 'Read [the guide](/guide)', swatch: '#ddd', media: {image: 'assets/silver.svg', alt: 'Silver speaker'}, facts: [{label: 'Weight', value: '**560 g**'}], cta: {label: 'Explore', href: '/guide'}},
  {id: 'copper', label: 'Copper', heading: 'A warmer view.', body: 'A different finish.', swatch: '#b97650'},
];
const selector = {type: 'selector', label: 'Choose a finish', initial: 'copper', presentation: 'swatches', items};
const section = (node: unknown) => ({type: 'compose', version: 1, label: 'Choices', children: [{type: 'group', children: [node]}]});
it('accepts independent selector/explorer nodes and resolves defaults inside nested groups', () => {
  const value = composeSchema.parse(section(selector));
  const node = (value.children[0] as {children: unknown[]}).children[0];
  expect(node).toMatchObject({initial: 'copper', itemHeadingLevel: 3, mediaSide: 'end', ratio: 'landscape', fit: 'contain'});
  expect(composeSchema.safeParse(section({type: 'explorer', label: 'Features', presentation: 'list', items})).success).toBe(true);
});
it('rejects duplicate IDs/labels, unknown initial choices and incomplete swatches', () => {
  for (const invalid of [
    {...selector, initial: 'missing'}, {...selector, items: [items[0], {...items[1], id: 'silver'}]},
    {...selector, items: [items[0], {...items[1], label: 'Silver'}]},
    {...selector, items: [items[0], {...items[1], swatch: undefined}]},
    {...selector, items: [items[0], {...items[1], swatch: 'url(bad)'}]},
    {...selector, items: [items[0], {...items[1], swatch: '#00000000'}]},
    {...selector, items: [items[0]]}, {...selector, items: Array(9).fill(items[0])},
    {...selector, itemHeadingLevel: 1}, {...selector, items: [{...items[0], id: 'bad id'}, items[1]]},
    {...selector, items: [{...items[0], media: {image: 'assets/a.svg', alt: ''}}, items[1]]},
    {...selector, items: [{...items[0], facts: Array(9).fill({label:'Weight',value:'560 g'})}, items[1]]},
  ]) expect(composeSchema.safeParse(section(invalid)).success).toBe(false);
});
it('keeps panel media, body, facts and CTA references at their authored locations', () => {
  const value = composeSchema.parse(section(selector));
  expect(schemaReferences(composeSchema, value)).toEqual(expect.arrayContaining([
    {kind: 'asset', field: 'children[0].children[0].items[0].media.image', value: 'assets/silver.svg'},
    {kind: 'inline', field: 'children[0].children[0].items[0].body', value: items[0]!.body},
    {kind: 'inline', field: 'children[0].children[0].items[0].facts[0].value', value: '**560 g**'},
    {kind: 'href', field: 'children[0].children[0].items[0].cta.href', value: '/guide'},
  ]));
});
