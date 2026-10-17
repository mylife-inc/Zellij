// Offline verification and packaging. Network is never needed by this script.
// --write regenerates fonts.css and licenses.txt after a snapshot update.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cp, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '../src/themes/fonts');
const manifest = JSON.parse(await readFile(path.join(root, 'manifest.json'), 'utf8'));
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const expectedFiles = new Set();
let css = '/* Generated from the pinned snapshot by scripts/prepare-fonts.mjs --write. */\n';
const variables = [];
const notices = [];

async function verified(entry) {
  assert(!path.isAbsolute(entry.path) && !entry.path.split(/[\\/]/).includes('..'));
  const bytes = await readFile(path.join(root, entry.path));
  assert.equal(hash(bytes), entry.sha256, `Font snapshot changed: ${entry.path}`);
  return bytes;
}

assert.equal(manifest.version, 1);
for (const family of manifest.families) {
  const upstream = (await verified(family.css)).toString('utf8');
  const license = (await verified(family.license)).toString('utf8').split(/\r?\n/).map((line) => line.trimEnd()).join('\n');
  const sources = new Map();
  for (const file of family.files) {
    const bytes = await verified(file);
    assert.equal(bytes.length, file.bytes);
    assert.equal(bytes.toString('ascii', 0, 4), 'wOF2', `Invalid WOFF2: ${file.path}`);
    sources.set(file.url, file.path);
    expectedFiles.add(path.basename(file.path));
  }
  const used = new Set();
  const local = upstream.replace(/url\(([^)]+)\)/g, (_, url) => {
    assert(sources.has(url), `Unpinned font URL: ${url}`);
    used.add(url);
    return `url('./${sources.get(url)}')`;
  }).replaceAll(`font-family: '${family.name}';`, `font-family: 'Zellij ${family.name}';`);
  assert.equal(used.size, sources.size, `Unused files for ${family.name}`);
  assert(!/https?:|@import/i.test(local), 'Font CSS must be entirely local');
  notices.push(`${family.name}\n${'-'.repeat(family.name.length)}\n${license.trim()}`);
  css += `\n/* ${family.name} — license notices in licenses.txt. */\n${local}`;
  const fallback = family.fallback;
  css += `\n@font-face {
  font-family: 'Zellij ${family.name} Fallback';
  src: local('${fallback.fallbackFont}');
  ascent-override: ${fallback.ascentOverride};
  descent-override: ${fallback.descentOverride};
  line-gap-override: ${fallback.lineGapOverride};
  size-adjust: ${fallback.sizeAdjust};
}\n`;
  variables.push(`  ${family.variable}: 'Zellij ${family.name}', 'Zellij ${family.name} Fallback';`);
}
assert.deepEqual(new Set(await readdir(path.join(root, 'files'))), expectedFiles, 'Remove obsolete font files');
// A real url() emits the notice file into the deployed site's assets. Keeping
// it only in a comment is insufficient: Turbopack strips even /*! comments.
css += `\n.zellij-fonts {\n${variables.join('\n')}\n  --zellij-font-license-url: url('./licenses.txt');\n}\n`;
const cssFile = path.join(root, 'fonts.css');
const noticeFile = path.join(root, 'licenses.txt');
const noticeText = `${notices.join('\n\n')}\n`;
if (process.argv.includes('--write')) {
  await writeFile(cssFile, css);
  await writeFile(noticeFile, noticeText);
} else {
  assert.equal(await readFile(cssFile, 'utf8'), css, 'Regenerate fonts.css with --write');
  assert.equal(await readFile(noticeFile, 'utf8'), noticeText, 'Regenerate licenses.txt with --write');
}

// The unbundled ESM/CJS modules retain the same relative CSS import in dist.
if (!process.argv.includes('--write')) await cp(root, path.resolve(root, '../../../dist/themes/fonts'), { recursive: true });
console.log(`[zellij] Verified ${manifest.families.length} local font families, ${expectedFiles.size} WOFF2 files and their licenses.`);
