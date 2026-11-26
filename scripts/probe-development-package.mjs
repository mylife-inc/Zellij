// Maintainer-only controlled variants. Never run in release packaging.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
const repo = path.resolve(import.meta.dirname, '..');
const runner = path.join(repo, 'scripts/development-runtime/run.mjs');
const original = await readFile(runner, 'utf8');
const variant = process.argv[2];
let source = original.replaceAll('\r\n', '\n');
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
  replace('const config = {', `for (const [file, statement] of [
    ['src/app/layout.tsx', "import '@shebka/zellij/styles.css';"],
    ['node_modules/@shebka/zellij/src/themes/fonts.ts', "import './fonts/fonts.css';"],
  ]) {
    const value = await readFile(file, 'utf8');
    if (!value.includes(statement)) throw new Error('CSS diagnostic insertion not found: ' + file);
    await writeFile(file, value.replace(statement, ''));
  }
const config = {`);
} else if (variant === 'postcss') {
  replace('const config = {', "await writeFile('postcss.config.mjs', `export default { plugins: { '@tailwindcss/postcss': {} } };\nconsole.log('PROBE: PostCSS config loaded');\n`);\nconst config = {");
  // Tailwind's own timing traces contain paths and timings, never environment values.
  replace('running = true;', "process.env.DEBUG = 'tailwindcss';\nrunning = true;");
} else if (variant === 'minimal') {
  replace('const config = {', `await writeFile('src/app/layout.tsx', 'export default function Layout({children}: {children: React.ReactNode}) { return <html><body>{children}</body></html>; }');
await writeFile('src/app/[[...slug]]/page.tsx', 'export default function Page() { console.log("PROBE: minimal page rendered"); return <main>Original</main>; }');
const config = {`);
} else if (variant === 'no-stdin') {
  replace('const config = {', `const bundle = 'node_modules/@shebka/zellij/dist/cli/dev-bundle.js';
const value = await readFile(bundle, 'utf8');
const statement = '{ cwd: appRoot, stdio: "inherit" }';
if (!value.includes(statement)) throw new Error('Next stdin diagnostic insertion not found');
await writeFile(bundle, value.replace(statement, '{ cwd: appRoot, stdio: ["ignore", "inherit", "inherit"] }'));
const config = {`);
} else if (variant === 'runtime') {
  replace('const config = {', `async function instrument(file, before, after) {
    const value = await readFile(file, 'utf8');
    if (!value.includes(before)) throw new Error('Runtime diagnostic insertion not found: ' + file);
    await writeFile(file, value.replace(before, after));
  }
const prefix = 'node_modules/@shebka/zellij/src/';
await instrument(prefix + 'render/root-layout.tsx',
  'const { site, developmentRevision } = await loadZellijRuntime(options);',
  'console.log("PROBE: layout load start"); const { site, developmentRevision } = await loadZellijRuntime(options); console.log("PROBE: layout load done");');
await instrument(prefix + 'render/page.tsx',
  'async function Page({ params }: ZellijPageProps) {',
  'async function Page({ params }: ZellijPageProps) { console.log("PROBE: page render start");');
await instrument(prefix + 'config/runtime.ts',
  'return readDevelopmentRevision(appRoot).then((revision) => loadRuntime(options, revision));',
  'console.log("PROBE: revision read start"); return readDevelopmentRevision(appRoot).then((revision) => { console.log("PROBE: revision read done"); return loadRuntime(options, revision); });');
await instrument(prefix + 'config/runtime.ts',
  'const site = await loadSite(source);',
  'console.log("PROBE: loadSite start"); const site = await loadSite(source); console.log("PROBE: loadSite done");');
await instrument(prefix + 'mdx/pipeline.ts',
  'const [remarkMaths, rehypeMaths] = await loadMaths();',
  'console.log("PROBE: MDX options start"); const [remarkMaths, rehypeMaths] = await loadMaths(); console.log("PROBE: MDX options done");');
const config = {`);
} else if (variant !== 'quiet') throw new Error(`Unknown development probe: ${variant}`);
execFileSync(process.execPath, ['--input-type=module', '--check'], { input: source, stdio: ['pipe', 'inherit', 'inherit'] });
if (process.argv.includes('--check')) process.exit(0);
try {
  await writeFile(runner, source);
  execFileSync(process.execPath, [path.join(repo, 'scripts/package-development.mjs')], { cwd: repo, stdio: 'inherit' });
} finally { await writeFile(runner, original); }
