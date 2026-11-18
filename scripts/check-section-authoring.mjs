// Native authoring acceptance against the actual engine's Zod schemas. No
// container or browser is involved; this also runs on Windows and macOS CI.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { build } from 'esbuild';
import YAML from 'yaml';

const binary = path.resolve(process.argv[2] ?? 'cli/target/debug/zel');
const root = fileURLToPath(new URL('../', import.meta.url));
const temporary = await mkdtemp(path.join(root, 'node_modules/.section-authoring-'));
try {
  const helper = path.join(temporary, 'schema.mjs');
  await build({ stdin: { contents: `
    export * from './engine/packages/zellij/src/sections/schemas';
    export { getSection, registeredSectionTypes } from './engine/packages/zellij/src/sections/registry';
    export { buildSectionTemplates } from './engine/packages/zellij/src/cli/section-templates';
  `, resolveDir: root }, bundle: true, packages: 'external', platform: 'node', format: 'esm', outfile: helper });
  const { getSection, registeredSectionTypes, buildSectionTemplates } = await import(pathToFileURL(helper));
  assert.equal(registeredSectionTypes().length, 25);
  const templates = buildSectionTemplates();
  const schema = JSON.parse(execFileSync(binary, ['context', '--json'], { encoding: 'utf8' }));
  const presetList = JSON.parse(execFileSync(binary, ['presets', '--json'], { encoding: 'utf8' })).presets;
  assert.deepEqual(presetList.map(p => p.name).sort(), Object.keys(schema.sectionPresets).sort());
  for (const [name, preset] of Object.entries(schema.sectionPresets)) {
    const original = execFileSync(binary, ['ns', '-p', name], { encoding: 'utf8' });
    assert.equal(original, preset.yaml);
    const edited = execFileSync(binary, ['ns', '-p', name, '--set', 'label=Edited preset 日本'], { encoding: 'utf8' });
    const value = YAML.parse(edited);
    assert.equal(value.label, 'Edited preset 日本');
    getSection(preset.sectionType).schema.parse(value);
  }
  const text = 'TRUE: \'quoted\' "double" \\ Unicode العربية 日本\nnext\r\nline\t\u0085\u2028\u2029\ufffe\uffff';
  const execute = (kind, args) => execFileSync(binary, ['ns', kind, ...args], { encoding: 'utf8', timeout: 10000 });
  const fields = (value, prefix = '') => Object.entries(value).flatMap(([key, entry]) => {
    if (key === 'type') return [];
    const field = prefix ? `${prefix}.${key}` : key;
    return entry && typeof entry === 'object' ? fields(entry, field) : typeof entry === 'string' ? ['--set', `${field}=${text}`] : [];
  });
  for (const kind of registeredSectionTypes()) {
    assert.equal(execute(kind, []), templates[kind]);
    const counts = Object.entries(YAML.parse(templates[kind])).flatMap(([key, value]) => {
      if (!Array.isArray(value)) return [];
      const count = Math.min(value.length + 1, schema.sections[kind].properties[key].maxItems ?? 256);
      return ['--count', `${key}=${count}`];
    });
    const expanded = YAML.parse(execute(kind, counts));
    const authored = YAML.parse(execute(kind, [...counts, ...fields(expanded)]));
    getSection(kind).schema.parse(authored);
    const compare = value => { for (const [key, entry] of Object.entries(value)) {
      if (key === 'type') continue;
      if (entry && typeof entry === 'object') compare(entry);
      else if (typeof entry === 'string') assert.equal(entry, text);
    } };
    compare(authored);
    const jsonFields = Object.entries(authored).filter(([key]) => key !== 'type')
      .flatMap(([key, value]) => ['--set-json', `${key}=${JSON.stringify(value)}`]);
    const structured = YAML.parse(execute(kind, jsonFields));
    assert.deepEqual(structured, authored, `${kind}: JSON replacement changed the supplied values`);
    getSection(kind).schema.parse(structured);
  }
  for (const [kind, args] of [
    ['hero', ['--set', 'background.gradient.0=accent', '--set', 'background.gradient.1=surface', '--count', 'cta=2', '--set', 'cta.1.label=Read']],
    ['hero-split', ['--set', 'visual=assets/hero.svg']],
    ['steps-panel', ['--set', 'diagram=flowchart LR; A-->B']],
    ['media-panel', ['--set', 'video=assets/movie.mp4', '--set', 'scrim=0.5']],
    ['cta-banner', ['--count', 'cta=2', '--set', 'cta.1.label=Read']],
    ['pricing-table', ['--tiers', '3', '--set', 'tiers.1.name=Team', '--highlight', 'Team']],
  ]) getSection(kind).schema.parse(YAML.parse(execute(kind, args)));
  const refined = YAML.parse(execute('hero', [
    '--set-json', 'cta=[{"label":"Old","href":"/guide","style":"ghost"}]',
    '--set', 'cta.0.label=Read 日本', '--unset', 'cta.0.style',
    '--set-json', 'background={"gradient":["accent","surface"]}',
  ]));
  assert.deepEqual(refined.cta, [{ label: 'Read 日本', href: '/guide' }]);
  getSection('hero').schema.parse(refined);
  const composition = YAML.parse(execute('compose', ['--set-json', 'children=' + JSON.stringify([
    { type: 'group', children: [{ type: 'video', src: 'assets/demo.mp4', label: 'A demonstration',
      tracks: [{ src: 'assets/demo.en.vtt', srclang: 'en', label: 'English' }], transcript: '/transcript' }] },
  ]), '--set', 'children.0.children.0.tracks.0.label=English captions']));
  getSection('compose').schema.parse(composition);
  assert.equal(composition.children[0].children[0].tracks[0].label, 'English captions');
  const richChildren = [{type: 'group', place: 'center', children: [
    {type: 'table', caption: 'Modes', rowLabel: 'Element', columns: ['Web'], rows: [{label: 'Text', cells: ['Readable']}]},
    {type: 'prose', body: '## A thought\n\nA clear paragraph.'},
    {type: 'code', title: 'Build', code: 'zel build'},
    {type: 'disclosure', summary: 'More', body: 'Read [the guide](/guide)'},
    {type: 'list', ordered: true, items: ['One', 'Two']},
    {type: 'quote', text: 'Choose carefully.', author: 'Studio'},
    {type: 'icon', name: 'book-open', label: 'Read'},
    {type: 'gallery', label: 'Images', items: [{image: 'assets/a.svg', alt: 'A'}]},
    {type: 'carousel', label: 'Cards', items: [{title: 'One'}, {title: 'Two'}]},
  ]}];
  const richLayout = {kind: 'layers', backdrop: {kind: 'image', src: 'assets/a.svg'}};
  const richScroll = {start: 0.2, end: 0.8, from: {y: 32}, to: {}};
  const richArgs = (children, layout, scroll) => ['ns', 'compose', '--set-json', 'children=' + JSON.stringify(children), '--set-json', 'layout=' + JSON.stringify(layout), '--set-json', 'scroll=' + JSON.stringify(scroll)];
  const rich = YAML.parse(execFileSync(binary, richArgs(richChildren, richLayout, richScroll), {encoding: 'utf8'}));
  getSection('compose').schema.parse(rich);
  for (const [children, layout, scroll] of [
    [richChildren, richLayout, {...richScroll, start: 0.9}],
    [richChildren, {kind: 'stack'}, richScroll],
    [[...richChildren, richChildren[0]], richLayout, richScroll],
  ]) {
    const candidate = {type: 'compose', version: 1, label: 'Invalid', children, layout, scroll};
    assert.equal(getSection('compose').schema.safeParse(candidate).success, false);
    assert.notEqual(spawnSync(binary, richArgs(children, layout, scroll)).status, 0);
  }
  const prompt = spawnSync(binary, ['ns', '--interactive'], { input: 'hero\nAuthored interactively\n\n', encoding: 'utf8', timeout: 10000 });
  assert.equal(prompt.status, 0, prompt.stderr);
  assert.match(prompt.stderr, /Section type/);
  assert.equal(YAML.parse(prompt.stdout).headline, 'Authored interactively');
  getSection('hero').schema.parse(YAML.parse(prompt.stdout));
  console.log('All 25 authored section types and four editable presets, complete JSON replacements, scalar refinements, optional-field removal, Unicode text, nested arrays, media choices, pricing and interactive output pass the engine schemas.');
} finally { await rm(temporary, { recursive: true, force: true }); }
