// Maintainer-only packaging. Consumers use native zel extraction, never npm/tar.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { lstat, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
const repo = path.resolve(import.meta.dirname, '..');
const version = (await readFile(path.join(repo, 'cli/Cargo.toml'), 'utf8')).match(/^version = "([^"]+)"/m)[1];
const platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`;
const output = path.resolve(process.argv[2] ?? 'dist/development-packages');
const deps = path.join(repo, 'scripts/development-runtime');
const stage = await mkdtemp(path.join(os.tmpdir(), 'zel-development-package-'));
const sha = b => createHash('sha256').update(b).digest('hex');
try {
  const root = path.join(stage, 'package'); await mkdir(root); await mkdir(output, { recursive: true });
  const modules = path.join(deps, 'node_modules');
  assert((await lstat(modules)).isDirectory(), 'Run npm ci --ignore-scripts in scripts/development-runtime first');
  const files = {}, inventory = [];
  async function copy(dir) {
    for (const e of (await readdir(dir, { withFileTypes: true })).sort((a,b) => a.name.localeCompare(b.name))) {
      if (e.name === '.bin' || e.name === '.package-lock.json') continue;
      const source = path.join(dir, e.name), relative = path.relative(deps, source).split(path.sep).join('/');
      // The cached wrapper serves original images. Next's optional native image
      // optimizer is never used; keep it out of this separate development pack.
      if (/^node_modules\/(sharp(?:\/|$)|@img\/sharp[^/]*(?:\/|$))/.test(relative)) continue;
      // Keep Next's diagnostic source maps and all covered-library sources.
      // Large generated Mermaid/Lucide maps and demo media are not runtime inputs.
      if (/^node_modules\/(mermaid|lucide-react)\/.*\.map$/.test(relative)
        || /^node_modules\/cytoscape-fcose\/demo(?:\/|$)/.test(relative)) continue;
      if (e.isDirectory()) await copy(source);
      else {
        const resolved = await realpath(source);
        assert(resolved.startsWith(modules + path.sep), `Dependency link escapes package: ${relative}`);
        assert((await lstat(resolved)).isFile(), `Not a regular dependency: ${relative}`);
        const b = await readFile(resolved), target = path.join(root, relative);
        await mkdir(path.dirname(target), { recursive: true }); await writeFile(target, b);
        files[relative] = sha(b);
        if (e.name === 'package.json') {
          const p = JSON.parse(b); if (p.name && p.version) inventory.push({ path: relative, name: p.name, version: p.version, license: p.license ?? null, repository: p.repository ?? null });
        }
      }
    }
  }
  await copy(modules);
  for (const name of ['package.json','package-lock.json', 'THIRD-PARTY.md']) {
    const b = await readFile(path.join(deps, name)); await writeFile(path.join(root, name), b); files[name] = sha(b);
  }
  const notices = Buffer.from(JSON.stringify({ schema: 1, packages: inventory }, null, 2)+'\n');
  await writeFile(path.join(root, 'DEPENDENCIES.json'), notices); files['DEPENDENCIES.json'] = sha(notices);
  const runner = await readFile(path.join(deps, 'run.mjs')); await writeFile(path.join(root, 'run.mjs'), runner); files['run.mjs'] = sha(runner);
  await writeFile(path.join(root, 'manifest.json'), JSON.stringify({ format: 1, kind: 'next-development', version, platform, node_major: 24, files }, null, 2)+'\n');
  const archive = `zel-development-v${version}-${platform}.tar.gz`;
  await writeFile(path.join(stage, 'files.txt'), [...Object.keys(files), 'manifest.json'].join('\n')+'\n');
  execFileSync('tar', ['-czf', path.join(output, archive), '-C', root, '-T', path.join(stage, 'files.txt')], { env: { ...process.env, COPYFILE_DISABLE: '1' } });
  const bytes = await readFile(path.join(output, archive));
  await writeFile(path.join(output, archive+'.sha256'), `${sha(bytes)}  ${archive}\n`);
  console.log(JSON.stringify({ archive, bytes: bytes.length, files: Object.keys(files).length, platform, nodeIncluded: false }));
} finally { await rm(stage, { recursive: true, force: true }); }
