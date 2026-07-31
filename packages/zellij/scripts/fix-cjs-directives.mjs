/**
 * Restores `"use client"` in the CommonJS output (spec §13.2).
 *
 * esbuild's CJS transform emits its own `"use strict"` prologue and drops the
 * original directive, so client modules come out of the build looking like
 * server modules. The ESM output keeps them, which makes this easy to miss —
 * scripts/verify-build.mjs is what caught it.
 *
 * The directive is re-inserted at the very top of the prologue, ahead of
 * `"use strict"`: both are directive prologue entries, and bundlers look for
 * `"use client"` at the start of the module.
 *
 *     node scripts/fix-cjs-directives.mjs
 */
import { readdir, readFile, writeFile } from 'node:fs/promises';
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

let patched = 0;

for await (const file of walk(SRC)) {
  if (!/\.tsx?$/.test(file) || /\.test\.tsx?$/.test(file)) continue;
  if (!DIRECTIVE.test(await readFile(file, 'utf8'))) continue;

  const relative = path.relative(SRC, file).replace(/\.tsx?$/, '');
  const target = path.join(DIST, `${relative}.cjs`);

  let contents;
  try {
    contents = await readFile(target, 'utf8');
  } catch {
    continue;
  }
  if (DIRECTIVE.test(contents)) continue;

  await writeFile(target, `"use client";\n${contents}`);
  patched += 1;
}

console.log(`[zellij] Restored "use client" in ${patched} CommonJS module(s).`);
