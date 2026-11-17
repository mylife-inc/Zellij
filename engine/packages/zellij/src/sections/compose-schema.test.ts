import { expect, it } from 'vitest';
import { z } from 'zod';
import { composeSchema, composeHasPageHeading } from './compose-schema';
import { schemaReferences } from '../content/reference-schema';
import { validateSection } from '../content/model-schema';
import { buildSchemaJson, buildContext } from '../cli/context';

const text = { type: 'text', text: 'A readable idea.' };
const group = (children: unknown[]) => ({ type: 'group', children });
const section = (children: unknown[]) => ({ type: 'compose', version: 1, label: 'Overview', children });

it('accepts three nested groups and rejects a fourth without recursive schema references', () => {
  const children = [group([group([group([text])])])];
  expect(composeSchema.safeParse(section(children)).success).toBe(true);
  expect(composeSchema.safeParse(section([group(children)])).success).toBe(false);
  const json = JSON.stringify(z.toJSONSchema(composeSchema, { io: 'input' }));
  expect(json).not.toContain('"$ref"');
  expect(json).not.toContain('"$defs"');
  expect(json.length).toBeLessThan(50000);
});

it('bounds total nodes independently of per-container size and counts primary headings at every depth', () => {
  const seven = Array.from({ length: 7 }, () => group(Array.from({ length: 17 }, () => text)));
  expect(composeSchema.safeParse(section([...seven, text, text])).success).toBe(true); // 128
  expect(composeSchema.safeParse(section([...seven, text, text, text])).success).toBe(false);
  const h1 = { type: 'heading', level: 1, text: 'Only once' };
  const parsed = composeSchema.parse(section([group([h1]), text]));
  expect(composeHasPageHeading(parsed)).toBe(true);
  expect(composeHasPageHeading(composeSchema.parse(section([text])))).toBe(false);
  expect(composeSchema.safeParse(section([h1, group([h1])])).success).toBe(false);
});

it('collects nested assets, links and inline Markdown at their complete authored paths', () => {
  const value = composeSchema.parse(section([group([group([
    { type: 'image', image: 'assets/diagram.svg', alt: 'Architecture', caption: 'Read [the guide](/guide)' },
    { type: 'button', label: 'Start', href: '/guide/start' },
    { type: 'text', text: 'Read **carefully** and [continue](/guide/next)' },
  ])])]));
  expect(schemaReferences(composeSchema, value)).toEqual(expect.arrayContaining([
    { kind: 'asset', value: 'assets/diagram.svg', field: 'children[0].children[0].children[0].image' },
    { kind: 'href', value: '/guide/start', field: 'children[0].children[0].children[1].href' },
    { kind: 'inline', value: 'Read [the guide](/guide)', field: 'children[0].children[0].children[0].caption' },
  ]));
});

it('reports unknown nested fields without exposing their values', () => {
  const warnings: string[] = [];
  validateSection('compose.yaml', section([group([{ ...text, unsafeScript: 'never-print-me' }])]), warning => warnings.push(warning));
  expect(warnings).toHaveLength(1);
  expect(warnings[0]).toContain('children[0].children[0].unsafeScript');
  expect(warnings[0]).not.toContain('never-print-me');
});

it('exports a complete bounded schema, node alternatives and native authoring limits', () => {
  const json = JSON.parse(buildSchemaJson());
  expect(json.sectionTypes).toContain('compose');
  expect(json.sectionAuthoringRules.compose.boundedTree.maxNodes).toBe(128);
  const context = buildContext();
  expect(context).toContain('**`children (option 8)`**');
  expect(context).toContain('at most three groups');
  expect(composeSchema.safeParse(section([{ type: 'heading', text: 'Hello', gradient: { colors: ['accent', 'url(bad)'] } }])).success).toBe(false);
  expect(composeSchema.safeParse(section([{ type: 'heading', text: 'Hello', level: 1.5 }])).success).toBe(false);
});

it('validates video labels and bounded caption tracks and preserves nested reference locations', () => {
  const video = { type: 'video', label: 'A demonstration', src: 'assets/demo.mp4', poster: 'assets/poster.svg',
    tracks: [{ src: 'assets/captions.vtt', srclang: 'en-GB', label: 'English' }], transcript: '/transcript',
    caption: 'Watch **carefully** and [read more](/guide)' };
  const value = composeSchema.parse(section([group([video])]));
  const refs = schemaReferences(composeSchema, value);
  expect(refs).toEqual(expect.arrayContaining([
    { kind: 'asset', value: 'assets/demo.mp4', field: 'children[0].children[0].src' },
    { kind: 'asset', value: 'assets/poster.svg', field: 'children[0].children[0].poster' },
    { kind: 'asset', value: 'assets/captions.vtt', field: 'children[0].children[0].tracks[0].src' },
    { kind: 'href', value: '/transcript', field: 'children[0].children[0].transcript' },
    { kind: 'inline', value: video.caption, field: 'children[0].children[0].caption' },
  ]));
  for (const invalid of [{ ...video, label: '' }, { ...video, src: '' },
    { ...video, tracks: Array(9).fill(video.tracks[0]) },
    { ...video, tracks: [{ ...video.tracks[0], srclang: 'not a language' }] }]) {
    expect(composeSchema.safeParse(section([invalid])).success).toBe(false);
  }
});
