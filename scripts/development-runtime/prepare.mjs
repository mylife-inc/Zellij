// Maintainer-only dependency manifest generation; installed zel never runs npm.
import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import path from 'node:path';
const repo = path.resolve(import.meta.dirname, '../..');
const engine = JSON.parse(await readFile(path.join(repo, 'engine/packages/zellij/package.json'), 'utf8'));
const lock = JSON.parse(await readFile(path.join(repo, 'package-lock.json'), 'utf8'));
const names = new Set([...Object.keys(engine.dependencies), ...Object.keys(engine.peerDependencies),
  '@tailwindcss/postcss', 'tailwindcss', '@types/node', '@types/react', '@types/react-dom', 'typescript']);
const dependencies = Object.fromEntries([...names].sort().map(name => {
  const version = lock.packages[`node_modules/${name}`]?.version;
  if (!version) throw new Error(`Missing locked dependency: ${name}`);
  return [name, version];
}));
const generated = JSON.stringify({
  name: 'zellij-development-dependencies', version: '1.0.0', private: true,
  description: 'Optional, separately downloaded dependencies for zel dev.',
  license: 'UNLICENSED', dependencies, overrides: engine.overrides,
}, null, 2) + '\n';
const file = path.join(import.meta.dirname, 'package.json');
if (process.argv.includes('--check')) assert.equal((await readFile(file, 'utf8')).replace(/\r\n/g, '\n'), generated, 'Regenerate and review the development dependency manifest/lock');
else await writeFile(file, generated);
const notices = await readFile(path.join(import.meta.dirname, 'THIRD-PARTY.md'), 'utf8');
for (const name of ['lightningcss', 'next', 'next-mdx-remote']) {
  const pinned = JSON.parse(await readFile(path.join(import.meta.dirname, 'package-lock.json'), 'utf8')).packages[`node_modules/${name}`].version;
  assert(notices.includes(`v${pinned}`), `Update the ${name} source notice for ${pinned}`);
}
