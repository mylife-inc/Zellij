// Format disposable copies of real authored sites, never the source corpus.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { cp, mkdtemp, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
const zel = path.resolve(`cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const engine = path.resolve('engine/packages/zellij/bin/zellij.mjs');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-format-corpus-'));
const preserved = async content => {
  const result = {};
  for (const file of await readdir(content, { recursive: true })) {
    if (!/\.(mdx?|svg|png|webp|woff2|mp4)$/.test(file)) continue;
    result[file] = (await readFile(path.join(content, file))).toString('base64');
  }
  return result;
};
try {
  for (const name of ['showcase', 'zellij-docs', 'codeseal-docs']) {
    const site = path.join(root, name);
    const content = path.join(site, 'content');
    await cp(path.resolve('examples', name, 'content'), content, { recursive: true });
    await writeFile(path.join(site, 'zel.yaml'), 'version: 1\nrenderer: hosted\n');
    const before = await preserved(content);
    const initial = spawnSync(zel, ['fmt', site, '--check'], { encoding: 'utf8' });
    assert([0, 1].includes(initial.status), initial.stderr);
    execFileSync(zel, ['fmt', site], { stdio: 'inherit' });
    execFileSync(zel, ['fmt', site, '--check'], { stdio: 'inherit' });
    assert.deepEqual(await preserved(content), before);
    execFileSync(process.execPath, [engine, 'check', content, '--deep', '--strict'], { stdio: 'inherit' });
  }
  console.log('Formatter: actual authored sites remain valid, idempotent, and preserve Markdown/media.');
} finally { await rm(root, { recursive: true, force: true, maxRetries: 5 }); }
