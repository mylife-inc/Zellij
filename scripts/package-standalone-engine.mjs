// Run on release/CI builders only; end users need no archiver or package manager.
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, writeFile, cp, rm, readdir, chmod } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
const repo = path.resolve(import.meta.dirname, '..');
const version = (await readFile(path.join(repo, 'cli/Cargo.toml'), 'utf8')).match(/^version = "([^"]+)"/m)[1];
const [platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`, directory = 'dist/renderer-packages'] = process.argv.slice(2);
const pinned = JSON.parse(await readFile(path.join(repo, 'scripts/standalone-runtime.json'), 'utf8'));
const target = pinned.archives[platform];
assert(target, `Unsupported renderer platform ${platform}`);
const out = path.resolve(directory); await mkdir(out, { recursive: true });
const stage = await mkdtemp(path.join(os.tmpdir(), 'zel-renderer-package-'));
const tar = process.platform === 'win32' ? path.join(process.env.SystemRoot, 'System32/tar.exe') : 'tar';
const hash = data => createHash('sha256').update(data).digest('hex');
try {
  const name = `node-${pinned.version}-${platform}`;
  const archive = `${name}.${target.extension}`;
  const response = await fetch(`https://nodejs.org/dist/${pinned.version}/${archive}`, { signal: AbortSignal.timeout(180_000) });
  assert(response.ok, `Node archive download returned ${response.status}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.equal(hash(bytes), target.sha256, 'Pinned Node archive checksum mismatch');
  await writeFile(path.join(stage, archive), bytes);
  // bsdtar on Windows accepts ZIP, as does unzip on Unix. Only these two files
  // from the verified official archive are included in the renderer package.
  const binary = platform.startsWith('win-') ? 'node.exe' : 'bin/node';
  if (target.extension === 'zip' && process.platform !== 'win32') execFileSync('unzip', ['-q', archive, `${name}/${binary}`, `${name}/LICENSE`], { cwd: stage });
  else execFileSync(tar, ['-xf', archive, `${name}/${binary}`, `${name}/LICENSE`], { cwd: stage });
  const root = path.join(stage, 'package'); await mkdir(root);
  await cp(path.join(stage, name, binary), path.join(root, path.basename(binary)));
  if (!platform.startsWith('win-')) await chmod(path.join(root, 'node'), 0o755);
  await cp(path.join(stage, name, 'LICENSE'), path.join(root, 'NODE-LICENSE.txt'));
  await cp(path.join(repo, 'dist/standalone-engine'), path.join(root, 'engine'), { recursive: true });
  const files = {};
  async function walk(dir) {
    for (const entry of (await readdir(dir, { withFileTypes: true })).sort((a,b) => a.name.localeCompare(b.name))) {
      const file = path.join(dir, entry.name);
      if (entry.isDirectory()) await walk(file);
      else { assert(entry.isFile()); files[path.relative(root, file).split(path.sep).join('/')] = hash(await readFile(file)); }
    }
  }
  await walk(root);
  await writeFile(path.join(root, 'manifest.json'), JSON.stringify({ format: 1, version, platform, node_version: pinned.version, files }, null, 2) + '\n');
  const filename = `zel-renderer-v${version}-${platform}.tar.gz`;
  // File-only entries: the native extractor deliberately rejects links and
  // ambiguous archive paths. A response file avoids command-line length limits.
  await writeFile(path.join(stage, 'files.txt'), [...Object.keys(files), 'manifest.json'].join('\n') + '\n');
  execFileSync(tar, ['-czf', path.join(out, filename), '-C', root, '-T', path.join(stage, 'files.txt')], { env: { ...process.env, COPYFILE_DISABLE: '1' } });
  const packed = await readFile(path.join(out, filename));
  await writeFile(path.join(out, `${filename}.sha256`), `${hash(packed)}  ${filename}\n`);
  console.log(JSON.stringify({ archive: filename, bytes: packed.length, runtime: pinned.version, files: Object.keys(files).length }));
} finally { await rm(stage, { recursive: true, force: true }); }
