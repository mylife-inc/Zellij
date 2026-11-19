import { build } from 'esbuild';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const file = name => path.join(root, 'src/offline', name);
const alias = { 'next/link': file('link.tsx'), 'next/navigation': file('navigation.ts'), 'next/image': file('image.tsx') };
const common = { bundle: true, jsx: 'automatic', sourcemap: false, logLevel: 'info', target: 'es2022' };
await build({ ...common, entryPoints: [file('export.tsx')], outfile: path.join(root, 'dist/offline/export.js'),
  platform: 'node', format: 'esm', packages: 'external', alias,
  banner: { js: "import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);" }, loader: { '.css': 'empty' } });
const browser = await build({ ...common, metafile: true, entryPoints: [file('browser.tsx')], outfile: path.join(root, 'dist/offline/runtime.js'),
  platform: 'browser', format: 'iife', minify: true, alias: { ...alias, '@shebka/zellij/optional-mermaid': file('mermaid-loader.ts') },
  define: { 'process.env.NODE_ENV': '"production"', 'process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS': 'globalThis.__zellijDiagrams' } });
const diagrams = await build({ ...common, metafile: true, entryPoints: [file('mermaid-entry.ts')], outfile: path.join(root, 'dist/offline/mermaid.js'),
  platform: 'browser', format: 'iife', minify: true, define: { 'process.env.NODE_ENV': '"production"' } });

// Keep full notices beside redistributed browser code, including transitive packages.
const packages = new Map();
for (const input of [...Object.keys(browser.metafile.inputs), ...Object.keys(diagrams.metafile.inputs)]) {
  if (!input.includes('node_modules/')) continue;
  let dir = path.dirname(path.resolve(input));
  while (dir !== path.dirname(dir)) {
    try {
      const manifest = JSON.parse(await readFile(path.join(dir, 'package.json'), 'utf8'));
      if (manifest.name) { packages.set(manifest.name, { dir, manifest }); break; }
    } catch { /* Package subdirectories may have no manifest. */ }
    dir = path.dirname(dir);
  }
}
const notices = [];
for (const [name, { dir, manifest }] of [...packages].sort(([a], [b]) => a.localeCompare(b))) {
  const files = (await readdir(dir)).filter(file => /^(?:licen[cs]e|copying|notice)(?:[.-]|$)/i.test(file));
  const texts = [];
  for (const file of files) { try { texts.push(await readFile(path.join(dir, file), 'utf8')); } catch {} }
  if (!texts.length) {
    try { const readme = await readFile(path.join(dir, 'README.md'), 'utf8');
      const license = readme.match(/^#+ Licen[cs]e[^\n]*\n([\s\S]*?)(?=^#+ |$(?![\s\S]))/mi);
      if (license) texts.push(license[1]);
    } catch {}
  }
  if (!texts.length) throw new Error(`Missing redistribution notice for ${name}`);
  notices.push(`${name} ${manifest.version} (${manifest.license ?? 'see notice'})\n${texts.join('\n')}`);
}
await writeFile(path.join(root, 'dist/offline/THIRD-PARTY-NOTICES.txt'), notices.join('\n\n========================================\n\n'));
