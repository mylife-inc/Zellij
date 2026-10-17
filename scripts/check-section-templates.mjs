// Exercise the Rust-to-engine boundary, including the committed embedded JSON.
// Usage: node scripts/check-section-templates.mjs /path/to/zel /new/content/root
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { stringify } from 'yaml';

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
await writeFile(path.join(content, 'site.yaml'), stringify({ name: 'Section templates', home: 'hero', motion: 'none' }));
await writeFile(path.join(content, 'pages.yaml'), stringify(pages));
// Media templates deliberately name replaceable authored assets. Supply real
// fixtures so static export tests rendering as well as schema acceptance.
await copyFile(path.join(repo, 'examples/showcase/content/assets/logo.svg'), path.join(content, 'assets/replace-me.svg'));
await copyFile(path.join(repo, 'scripts/fixtures/template.mp4'), path.join(content, 'assets/replace-me.mp4'));
execFileSync(process.execPath, [path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), 'check', content], { stdio: 'inherit' });
console.log(`Validated ${pages.length} templates emitted by zel.`);
