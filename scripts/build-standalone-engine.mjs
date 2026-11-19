// Publication-time compilation. The downloaded engine never runs a bundler.
import { build } from 'esbuild';
import postcss from 'postcss';
import tailwind from '@tailwindcss/postcss';
import { mkdir, readFile, writeFile, cp, readdir } from 'node:fs/promises';
import path from 'node:path';
const repo = path.resolve(import.meta.dirname, '..');
const engine = path.join(repo, 'engine/packages/zellij');
const out = path.resolve(process.argv[2] ?? path.join(repo, 'dist/standalone-engine'));
// MathJax otherwise discovers its version by evaluating __dirname/require and
// reading a package.json outside the bundle. It supports this build-time value.
const mathjax = JSON.parse(await readFile(path.join(repo, 'node_modules/mathjax-full/package.json'), 'utf8'));
await mkdir(out, { recursive: true });
const alias = Object.fromEntries(['link', 'navigation', 'image'].map(name => [`next/${name}`, path.join(engine, `src/offline/${name}.${name === 'navigation' ? 'ts' : 'tsx'}`)]));
const result = await build({ entryPoints: [path.join(engine, 'src/offline/standalone.ts')], outfile: path.join(out, 'engine.mjs'), bundle: true,
  platform: 'node', format: 'esm', target: 'node24', jsx: 'automatic', minify: true, metafile: true,
  alias, loader: { '.css': 'empty' }, define: { 'process.env.NODE_ENV': '"production"', PACKAGE_VERSION: JSON.stringify(mathjax.version) },
  banner: { js: "import {createRequire as __createRequire} from 'node:module'; const require=__createRequire(import.meta.url);" } });
const css = await postcss([tailwind({ base: repo, optimize: true })]).process(
  await readFile(path.join(engine, 'src/styles/zellij.css'), 'utf8'), { from: path.join(engine, 'src/styles/zellij.css') });
await writeFile(path.join(out, 'styles.css'), css.css + '\n' + await readFile(path.join(engine, 'src/themes/fonts/fonts.css'), 'utf8'));
await cp(path.join(engine, 'src/themes/fonts/files'), path.join(out, 'files'), { recursive: true });
await cp(path.join(engine, 'src/themes/fonts/licenses.txt'), path.join(out, 'FONT-LICENSES.txt'));
for (const file of ['runtime.js', 'mermaid.js', 'THIRD-PARTY-NOTICES.txt']) await cp(path.join(engine, 'dist/offline', file), path.join(out, file));
// Retain complete dependency licenses for the distributed server bundle too.
const packages = new Set();
for (const input of Object.keys(result.metafile.inputs)) {
  if (!input.includes('node_modules/')) continue;
  let dir = path.dirname(path.resolve(input));
  while (dir !== path.dirname(dir)) {
    try {
      const manifest = JSON.parse(await readFile(path.join(dir, 'package.json'), 'utf8'));
      if (manifest.name) { packages.add(dir); break; }
    } catch { /* Keep walking through nested module-type markers. */ }
    dir = path.dirname(dir);
  }
}
let notices = '';
for (const dir of [...packages].sort()) {
  const manifest = JSON.parse(await readFile(path.join(dir, 'package.json'), 'utf8'));
  notices += `\n===== ${manifest.name}@${manifest.version} (${manifest.license ?? 'see package'}) =====\n`;
  for (const file of await readdir(dir)) if (/^(license|licence|copying|notice)(\.|$)/i.test(file)) {
    try { notices += await readFile(path.join(dir, file), 'utf8') + '\n'; } catch { /* Some packages use a directory. */ }
  }
}
await writeFile(path.join(out, 'SERVER-THIRD-PARTY-NOTICES.txt'), notices);
console.log(`Standalone engine compiled to ${out}`);
