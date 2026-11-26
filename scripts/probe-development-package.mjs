// Maintainer-only controlled variants. Never run in release packaging.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
const repo = path.resolve(import.meta.dirname, '..');
const runner = path.join(repo, 'scripts/development-runtime/run.mjs');
const original = await readFile(runner, 'utf8');
const variant = process.argv[2];
let source = original;
function replace(from, to) {
  assert(source.includes(from), `Missing diagnostic insertion: ${from}`);
  source = source.replace(from, to);
}
if (variant === 'copy') {
  replace('readFile, readdir, symlink, writeFile', 'cp, readFile, readdir, symlink, writeFile');
  replace("await symlink(path.join(request.modules, entry.name), path.join(process.cwd(), 'node_modules', entry.name),\n    process.platform === 'win32' ? 'junction' : 'dir');",
    "await cp(path.join(request.modules, entry.name), path.join(process.cwd(), 'node_modules', entry.name), { recursive: true });");
} else if (variant === 'root') {
  replace('experimental: { turbopackAdditionalRoots: { zellijDependencies: { path: path.dirname(request.modules) } } },',
    "turbopack: { root: path.resolve(request.modules, '../../..') },");
} else if (variant === 'no-css') {
  replace('const config = {', "const layout = path.join(process.cwd(), 'src/app/layout.tsx');\nconst layoutSource = await readFile(layout, 'utf8');\nif (!layoutSource.includes(\"import '@shebka/zellij/styles.css';\")) throw new Error('CSS diagnostic insertion not found');\nawait writeFile(layout, layoutSource.replace(\"import '@shebka/zellij/styles.css';\", ''));\nconst config = {");
} else if (variant === 'postcss') {
  replace('const config = {', "await writeFile('postcss.config.mjs', `import tailwind from '@tailwindcss/postcss';\nexport default { plugins: { '@tailwindcss/postcss': {} } };\nconsole.log('PROBE: PostCSS config loaded');\n`);\nconst config = {");
  // Tailwind's own timing traces contain paths and timings, never environment values.
  replace('running = true;', "process.env.DEBUG = 'tailwindcss';\nrunning = true;");
} else throw new Error(`Unknown development probe: ${variant}`);
execFileSync(process.execPath, ['--input-type=module', '--check'], { input: source, stdio: ['pipe', 'inherit', 'inherit'] });
if (process.argv.includes('--check')) process.exit(0);
try {
  await writeFile(runner, source);
  execFileSync(process.execPath, [path.join(repo, 'scripts/package-development.mjs')], { cwd: repo, stdio: 'inherit' });
} finally { await writeFile(runner, original); }
