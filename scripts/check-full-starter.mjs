// Validate native --full output with the actual engine. --build is CI-only and
// additionally exports through the real container; the default needs no runtime.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';

const [binary, target, mode] = process.argv.slice(2);
assert(binary && target, 'Pass a zel binary and a new destination directory');
assert(!mode || mode === '--build');
const repo = fileURLToPath(new URL('../', import.meta.url));
const zel = path.resolve(binary);
const root = path.resolve(target);
const nodeCli = path.join(repo, 'engine/packages/zellij/bin/zellij.mjs');
const schema = JSON.parse(execFileSync(zel, ['context', '--json'], { encoding: 'utf8' }));
const run = (command, args, input) => execFileSync(command, args, { input, stdio: 'pipe', timeout: mode ? 600000 : 30000 });
const yaml = async file => YAML.parse(await readFile(file, 'utf8'));
for (const [guide, interactive] of [[false, false], [true, false], [true, true]]) {
  const site = path.join(root, interactive ? 'Wizard 日本' : guide ? 'Manual 日本' : 'docs/zellij');
  if (interactive) run(zel, ['nd', site, '-i'], 'Acme: "Docs" # 日本\ncupertino\nfull\nyes\n');
  else run(zel, ['nd', site, '--full', '--name', 'Acme: "Docs" # 日本', ...(guide ? ['--guide', '--theme', 'cupertino'] : [])]);
  const content = path.join(site, 'content');
  run(process.execPath, [nodeCli, 'check', content, '--strict', '--deep']);
  const pages = await yaml(path.join(content, 'pages.yaml'));
  const settings = await yaml(path.join(content, 'site.yaml'));
  assert.equal(settings.name, 'Acme: "Docs" # 日本');
  assert.equal(settings.theme, guide ? 'cupertino' : 'paper');
  assert.equal(pages.length, schema.sectionTypes.length + 2);
  const catalogue = await yaml(path.join(content, 'sections/examples/catalogue.yaml'));
  const filenames = await readdir(path.join(content, 'sections/examples'));
  assert.deepEqual(filenames.sort(), ['catalogue', ...schema.sectionTypes].map(type => type + '.yaml').sort());
  for (const type of schema.sectionTypes) {
    const section = await yaml(path.join(content, `sections/examples/${type}.yaml`));
    assert.equal(section.type, type);
    assert(!JSON.stringify(section).includes('TODO'));
    assert(!JSON.stringify(section).includes('replace-me'));
    assert(catalogue.body.includes(`](/example-${type})`));
    assert.deepEqual(pages.find(page => page.name === `example-${type}`).sections, [`examples/${type}`]);
  }
  for (const ext of ['svg', 'mp4']) assert.deepEqual(
    await readFile(path.join(content, `assets/section-example.${ext}`)),
    await readFile(path.join(repo, `cli/crates/zellij-scaffold/assets/section-example.${ext}`)),
  );
  if (mode && interactive) {
    const output = path.join(root, 'exported');
    run(zel, ['check', site, '--strict', '--deep']);
    run(zel, ['build', site, '--out', output]);
    for (const name of ['index', 'examples', 'guide', ...schema.sectionTypes.map(type => `example-${type}`)]) {
      const html = await readFile(path.join(output, name + '.html'), 'utf8');
      assert.match(html, /<!DOCTYPE html>/i);
      if (name.startsWith('example-')) assert(html.includes(`data-zellij-section="${name.slice(8)}"`), name);
    }
    const catalogueHtml = await readFile(path.join(output, 'examples.html'), 'utf8');
    for (const type of schema.sectionTypes) assert(catalogueHtml.includes(`href="/example-${type}"`));
    for (const ext of ['svg', 'mp4']) assert.deepEqual(
      await readFile(path.join(output, `_zellij/assets/section-example.${ext}`)),
      await readFile(path.join(content, `assets/section-example.${ext}`)),
    );
    console.log('Full starter wizard exports all 26 routes, linked examples and exact local media.');
  }
}
console.log(`Full starter: ${schema.sectionTypes.length} section types, catalogue links, local media, Unicode metadata and guide variants pass strict/deep engine checks.`);
