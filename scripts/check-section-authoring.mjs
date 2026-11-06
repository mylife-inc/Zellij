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
  assert.equal(registeredSectionTypes().length, 23);
  const templates = buildSectionTemplates();
  const schema = JSON.parse(execFileSync(binary, ['context', '--json'], { encoding: 'utf8' }));
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
  }
  for (const [kind, args] of [
    ['hero', ['--set', 'background.gradient.0=accent', '--set', 'background.gradient.1=surface', '--count', 'cta=2', '--set', 'cta.1.label=Read']],
    ['hero-split', ['--set', 'visual=assets/hero.svg']],
    ['steps-panel', ['--set', 'diagram=flowchart LR; A-->B']],
    ['media-panel', ['--set', 'video=assets/movie.mp4', '--set', 'scrim=0.5']],
    ['cta-banner', ['--count', 'cta=2', '--set', 'cta.1.label=Read']],
    ['pricing-table', ['--tiers', '3', '--set', 'tiers.1.name=Team', '--highlight', 'Team']],
  ]) getSection(kind).schema.parse(YAML.parse(execute(kind, args)));
  const prompt = spawnSync(binary, ['ns', '--interactive'], { input: 'hero\nAuthored interactively\n\n', encoding: 'utf8', timeout: 10000 });
  assert.equal(prompt.status, 0, prompt.stderr);
  assert.match(prompt.stderr, /Section type/);
  assert.equal(YAML.parse(prompt.stdout).headline, 'Authored interactively');
  getSection('hero').schema.parse(YAML.parse(prompt.stdout));
  console.log('All 23 authored section types, Unicode text, nested arrays, media choices, pricing and interactive output pass the engine schemas.');
} finally { await rm(temporary, { recursive: true, force: true }); }
