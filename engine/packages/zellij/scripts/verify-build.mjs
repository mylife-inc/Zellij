/**
 * Post-build check on the published artifact (spec §13.2).
 *
 * `"use client"` marks the RSC boundary per module. If a build ever drops one,
 * the failure is silent at build time and violent at runtime — an interactive
 * component becomes a server component and throws on its first hook. This
 * asserts the invariant instead of trusting the bundler to keep it.
 *
 *     node scripts/verify-build.mjs
 */
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const SRC = path.join(root, 'src');
const DIST = path.join(root, 'dist');

const DIRECTIVE = /^\s*['"]use client['"]/;

async function* walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(full);
    else yield full;
  }
}

const problems = [];

// 1. Every client module in src must still be marked in both output formats.
const clientModules = [];
for await (const file of walk(SRC)) {
  if (!/\.tsx?$/.test(file) || /\.test\.tsx?$/.test(file)) continue;
  if (DIRECTIVE.test(await readFile(file, 'utf8'))) {
    clientModules.push(path.relative(SRC, file).replace(/\.tsx?$/, ''));
  }
}

for (const module of clientModules) {
  for (const ext of ['js', 'cjs']) {
    const built = path.join(DIST, `${module}.${ext}`);
    try {
      if (!DIRECTIVE.test(await readFile(built, 'utf8'))) {
        problems.push(`lost "use client": dist/${module}.${ext}`);
      }
    } catch {
      problems.push(`missing output: dist/${module}.${ext}`);
    }
  }
}

// 2. The package entry points named in publishConfig must exist.
const pkg = JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8'));
for (const [name, entry] of Object.entries(pkg.publishConfig?.exports ?? {})) {
  const targets = typeof entry === 'string' ? [entry] : Object.values(entry);
  for (const target of targets) {
    try {
      await stat(path.join(root, target));
    } catch {
      problems.push(`exports "${name}" points at a missing file: ${target}`);
    }
  }
}

// 3. Server-only modules must NOT be marked client — that would pull node:fs
//    into the browser.
for (const serverOnly of [
  'content/folder-source',
  'content/legacy-source',
  'content/site',
  'config/runtime',
]) {
  const built = path.join(DIST, `${serverOnly}.js`);
  try {
    if (DIRECTIVE.test(await readFile(built, 'utf8'))) {
      problems.push(`server-only module marked "use client": dist/${serverOnly}.js`);
    }
  } catch {
    problems.push(`missing output: dist/${serverOnly}.js`);
  }
}

if (problems.length > 0) {
  console.error(`\n[zellij] Build verification failed (${problems.length}):`);
  for (const problem of problems) console.error(`  · ${problem}`);
  process.exit(1);
}

console.log(
  `[zellij] Build verified: ${clientModules.length} client modules keep their directive, ` +
    `entry points resolve, server-only modules stay server-side.`,
);
