// Hosted-only acceptance: install the real native archive through Homebrew,
// upgrade its formula revision, run its portable build test, then uninstall.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { homebrewFormula } from './homebrew-formula.mjs';

assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Run Homebrew acceptance on disposable hosted runners only');
const dist = path.resolve(process.argv[2] || 'dist');
const version = (await readFile('cli/Cargo.toml', 'utf8')).match(/^version = "([^"]+)"/m)?.[1];
assert(version, 'Missing workspace version');
const tag = `v${version}`;
const env = { ...process.env, HOMEBREW_NO_AUTO_UPDATE: '1', HOMEBREW_NO_INSTALL_CLEANUP: '1', HOMEBREW_NO_ANALYTICS: '1' };
async function run(command, args, options = {}) {
  return await new Promise((resolve, reject) => {
    const child = spawn(command, args, { env, ...options });
    let out = '', err = '';
    child.stdout?.on('data', bytes => { out += bytes; if (options.log) process.stdout.write(bytes); });
    child.stderr?.on('data', bytes => { err += bytes; if (options.log) process.stderr.write(bytes); });
    child.on('error', reject);
    child.on('close', code => code === 0 ? resolve(out) : reject(new Error(`${command} ${args.join(' ')} exited ${code}\n${out}\n${err}`)));
  });
}
const checksums = (await Promise.all((await readdir(dist)).filter(name => name.endsWith('.sha256')).sort().map(name => readFile(path.join(dist, name), 'utf8')))).join('');
const releaseFormula = homebrewFormula(tag, checksums);
const downloads = new Map();
for (const line of checksums.trim().split('\n')) {
  const [hash, name] = line.trim().split(/\s+/);
  assert.equal(path.basename(name), name);
  const bytes = await readFile(path.join(dist, name));
  assert.equal(createHash('sha256').update(bytes).digest('hex'), hash);
  downloads.set(name, bytes);
}
const server = createServer((request, response) => {
  const name = new URL(request.url, 'http://localhost').pathname.slice(1);
  const bytes = downloads.get(name);
  if (!bytes) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'Content-Length': bytes.length, 'Content-Type': 'application/octet-stream' });
  response.end(request.method === 'HEAD' ? undefined : bytes);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const temp = await mkdtemp(path.join(tmpdir(), 'zel-brew-'));
const tap = 'mylife-inc/tap';
const name = `${tap}/zel`;
let ownsTap = false;
let tapPath;
try {
  const brewRoot = (await run('brew', ['--repository'])).trim();
  tapPath = path.join(brewRoot, 'Library/Taps/mylife-inc/homebrew-tap');
  await mkdir(path.dirname(tapPath), { recursive: true });
  await mkdir(tapPath); // Refuse an existing tap rather than modify somebody else's recipe.
  ownsTap = true;
  await mkdir(path.join(tapPath, 'Formula'));
  await run('git', ['init', '--quiet', tapPath]);
  const localFormula = releaseFormula.replaceAll(`https://github.com/mylife-inc/releases/releases/download/zellij-${tag}`, origin);
  const formula = path.join(tapPath, 'Formula/zel.rb');
  await writeFile(formula, localFormula);
  await run('brew', ['install', name], { log: true });
  const prefix = (await run('brew', ['--prefix', name])).trim();
  assert.equal(await readFile(path.join(prefix, 'share/zel/installation-manager'), 'utf8'), 'homebrew\n');
  for (const file of ['etc/bash_completion.d/zel', 'share/zsh/site-functions/_zel', 'share/fish/vendor_completions.d/zel.fish']) {
    assert((await readFile(path.join(prefix, file), 'utf8')).includes('zel'), file);
  }
  await run('brew', ['test', name], { log: true, cwd: temp });
  // Exercise Homebrew's actual upgrade path without needing a retired release.
  await writeFile(formula, localFormula.replace(`  version "${version}"`, `  version "${version}"\n  revision 1`));
  await run('brew', ['upgrade', name], { log: true });
  const versions = await run('brew', ['list', '--versions', name]);
  assert(versions.includes(`${version}_1`), versions);
  await run('brew', ['test', name], { log: true, cwd: temp });
  await run('brew', ['uninstall', '--force', name], { log: true });
  assert(!(await run('brew', ['list', '--formula'])).split('\n').includes('zel'));
  console.log('Homebrew native install, completions, ownership guard, portable build, formula-revision upgrade and uninstall passed.');
} finally {
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  if (ownsTap) {
    await run('brew', ['uninstall', '--force', name]).catch(() => {});
    await rm(tapPath, { recursive: true, force: true });
  }
  await rm(temp, { recursive: true, force: true });
}
