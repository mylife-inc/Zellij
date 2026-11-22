// Maintainer-only fixture: exercise this checkout's compiled renderer rather
// than accidentally validating with a previously published candidate. Native
// package acceptance separately tests the pinned, downloaded private runtime.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile, cp, readdir } from 'node:fs/promises';
import path from 'node:path';

assert(process.argv[2], 'Pass a new renderer-cache directory');
assert.equal(process.versions.node.split('.')[0], '24');
const repo = path.resolve(import.meta.dirname, '..');
const version = (await readFile(path.join(repo, 'cli/Cargo.toml'), 'utf8')).match(/^version = "([^"]+)"/m)[1];
const platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`;
const cache = path.resolve(process.argv[2]);
await mkdir(cache); // Refuse an existing cache; never rewrite published identities.
const root = path.join(cache, `${version}-${platform}`);
await mkdir(root);
await cp(process.execPath, path.join(root, process.platform === 'win32' ? 'node.exe' : 'node'));
await cp(path.join(repo, 'dist/standalone-engine'), path.join(root, 'engine'), { recursive: true });
const files = {};
async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) await walk(file);
    else {
      assert(entry.isFile(), 'Test renderer must contain only regular files');
      files[path.relative(root, file).split(path.sep).join('/')] = createHash('sha256').update(await readFile(file)).digest('hex');
    }
  }
}
await walk(root);
await writeFile(path.join(root, 'manifest.json'), JSON.stringify({ format: 1, version, platform, node_version: process.version, files }));
console.log(`Prepared current-source test renderer: ${cache}`);
