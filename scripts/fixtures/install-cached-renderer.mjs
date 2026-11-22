// Protocol fixtures only. Production package tests separately verify pinned downloads.
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile, cp } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
export async function installCachedRenderer(binary, root, entry) {
  const version = JSON.parse(execFileSync(binary, ['version', '--json'], { encoding: 'utf8' })).cli.version;
  const platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`;
  const cache = path.join(root, 'cache');
  const cached = path.join(cache, `${version}-${platform}`);
  await mkdir(path.join(cached, 'engine'), { recursive: true });
  const node = process.platform === 'win32' ? 'node.exe' : 'node';
  await cp(process.execPath, path.join(cached, node));
  await cp(entry, path.join(cached, 'engine/engine.mjs'));
  for (const name of ['runtime.js', 'styles.css']) await writeFile(path.join(cached, 'engine', name), 'fixture');
  const files = {};
  for (const name of [node, 'engine/engine.mjs', 'engine/runtime.js', 'engine/styles.css']) files[name] = createHash('sha256').update(await readFile(path.join(cached, name))).digest('hex');
  await writeFile(path.join(cached, 'manifest.json'), JSON.stringify({ format: 1, version, platform, node_version: process.version, files }));
  return cache;
}
