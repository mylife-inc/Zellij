// Exercise the Rust-to-engine boundary, including the committed embedded JSON.
// Usage: node scripts/check-section-templates.mjs /path/to/zel /new/content/root
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse, stringify } from 'yaml';

const repo = fileURLToPath(new URL('../', import.meta.url));
const [binary, target] = process.argv.slice(2);
assert(binary && target, 'Pass the zel binary and a new content directory');
const content = path.resolve(target);
await mkdir(content); // Refuse to overwrite a caller's existing content.
await mkdir(path.join(content, 'sections'));
await mkdir(path.join(content, 'assets'));
const schema = JSON.parse(await readFile(path.join(repo, 'cli/crates/zellij-core/generated/schema.json'), 'utf8'));
const pages = [];
for (const type of schema.sectionTypes) {
  const file = path.join(content, 'sections', `${type}.yaml`);
  execFileSync(path.resolve(binary), ['new', 'section', type, '--out', file], { stdio: 'pipe' });
  assert.equal(await readFile(file, 'utf8'), schema.sectionTemplates[type]);
  pages.push({ name: type, title: type, description: `Generated ${type} template`, sections: [type] });
}
// The page wizard must preserve every selection and its order, while leaving
// registration to the author. Exercise the same source on all native CI OSes.
const originalPages = '# Keep authored comments and page order.\n' + stringify(pages);
await writeFile(path.join(content, 'pages.yaml'), originalPages);
const selected = [...schema.sectionTypes].reverse();
const output = execFileSync(path.resolve(binary), ['np', '-i', '--dir', content], {
  input: `wizard\n${selected.join(',')}\n`, encoding: 'utf8', stdio: 'pipe', timeout: 30000,
});
assert.equal(await readFile(path.join(content, 'pages.yaml'), 'utf8'), originalPages);
const report = output.split('Add this to pages.yaml:\n\n')[1];
assert(report, 'Page registration report missing');
const [entry] = parse(report.split(/\r?\n/).map(line => line.replace(/^    /, '')).join('\n'));
assert.equal(entry.name, 'wizard');
assert.deepEqual(entry.sections, selected.map(type => `wizard/${type}`));
for (const type of selected) assert.equal(
  await readFile(path.join(content, `sections/wizard/${type}.yaml`), 'utf8'),
  schema.sectionTemplates[type],
);
pages.push(entry); // Apply the report explicitly, as the author would.
// Exercise complete object/list input through a real hosted export too.
const structuredFile = path.join(content, 'sections', 'structured.yaml');
execFileSync(path.resolve(binary), ['ns', 'hero', '--out', structuredFile,
  '--set', 'headline=Structured authoring 日本', '--set', 'id=structured-demo',
  '--set-json', 'background={"gradient":["accent","surface"]}',
  '--set-json', 'cta=[{"label":"Old label","href":"#structured-demo","style":"ghost"}]',
  '--set', 'cta.0.label=Read structured content', '--unset', 'cta.0.style',
], { stdio: 'pipe' });
const structured = parse(await readFile(structuredFile, 'utf8'));
assert.deepEqual(structured.cta, [{ label: 'Read structured content', href: '#structured-demo' }]);
assert.deepEqual(structured.background, { gradient: ['accent', 'surface'] });
pages.push({ name: 'structured', title: 'Structured fields', sections: ['structured'] });
await writeFile(path.join(content, 'site.yaml'), stringify({ name: 'Section templates', home: 'hero', motion: 'none' }));
await writeFile(path.join(content, 'pages.yaml'), stringify(pages));
// Media templates deliberately name replaceable authored assets. Supply real
// fixtures so static export tests rendering as well as schema acceptance.
await copyFile(path.join(repo, 'examples/showcase/content/assets/logo.svg'), path.join(content, 'assets/replace-me.svg'));
await copyFile(path.join(repo, 'scripts/fixtures/template.mp4'), path.join(content, 'assets/replace-me.mp4'));
execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'check', content], { stdio: 'inherit' });
console.log(`Validated ${schema.sectionTypes.length} templates, a wizard page with all ordered section references and a structured-field page; authored registration stayed unchanged.`);
