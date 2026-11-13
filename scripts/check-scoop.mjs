// Hosted-only Scoop lifecycle acceptance using actual native CLI packages.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { cp, mkdir, mkdtemp, readFile, readdir, realpath, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { scoopManifest } from './scoop-manifest.mjs';

assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Scoop acceptance requires a disposable hosted runner');
assert.equal(process.platform, 'win32', 'Scoop acceptance requires native Windows');
const pin = 'e6aa3b366bdee8ed138c1e0f7b85192ebdd35d0f'; // Scoop v0.6.0
const dist = path.resolve(process.argv[2] || 'dist');
const source = path.resolve(process.argv[3]);
const target = process.env.TARGET;
const scoopShell = process.env.ZEL_SCOOP_SHELL || 'pwsh';
assert(['powershell.exe', 'pwsh'].includes(scoopShell));
assert(['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'].includes(target));
const version = process.env.ZEL_SCOOP_PUBLIC_VERSION?.replace(/^v/, '') ||
  (await readFile('cli/Cargo.toml', 'utf8')).match(/^version = "([^"]+)"/m)?.[1];
assert(version);
const tag = `v${version}`;
const sums = (await Promise.all((await readdir(dist)).filter(n => n.endsWith('.sha256')).sort().map(n => readFile(path.join(dist, n), 'utf8')))).join('');
const manifest = scoopManifest(tag, sums);
const downloads = new Map();
for (const asset of Object.values(manifest.architecture)) {
  const name = new URL(asset.url).pathname.split('/').pop();
  const bytes = await readFile(path.join(dist, name));
  assert.equal(createHash('sha256').update(bytes).digest('hex'), asset.hash);
  downloads.set(name, bytes);
}
const temp = await mkdtemp(path.join(tmpdir(), 'zel scoop ')); // Exercise paths containing spaces.
const scoopRoot = path.join(temp, 'scoop');
const core = path.join(scoopRoot, 'apps/scoop/current');
const env = { ...process.env, SCOOP: scoopRoot, SCOOP_GLOBAL: path.join(temp, 'global'),
  SCOOP_CACHE: path.join(temp, 'cache'), XDG_CONFIG_HOME: path.join(temp, 'config'),
  PATH: `${path.join(scoopRoot, 'shims')};${process.env.PATH}` };
async function run(command, args, { code = 0, cwd = temp, log = true, ...options } = {}) {
  return await new Promise((resolve, reject) => {
    const child = spawn(command, args, { env, cwd, timeout: 180000, ...options });
    let output = '';
    for (const stream of [child.stdout, child.stderr]) stream?.on('data', bytes => {
      output += bytes; if (log) process.stdout.write(bytes);
    });
    child.on('error', reject);
    child.on('close', actual => actual === code ? resolve(output) : reject(new Error(`${command} ${args.join(' ')} exited ${actual}, expected ${code}\n${output}`)));
  });
}
// Like Scoop's normal PowerShell shim, propagate the nested command's exit code.
const psLiteral = value => `'${value.replaceAll("'", "''")}'`;
const scoop = (args, options) => run(scoopShell, ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
  '-Command', `& ${[path.join(core, 'bin/scoop.ps1'), ...args].map(psLiteral).join(' ')}; exit $LASTEXITCODE`], options);
const server = createServer((request, response) => {
  const name = new URL(request.url, 'http://localhost').pathname.slice(1);
  const bytes = downloads.get(name);
  if (!bytes) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'Content-Length': bytes.length, 'Content-Type': 'application/octet-stream' });
  response.end(request.method === 'HEAD' ? undefined : bytes);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const localManifest = structuredClone(manifest);
for (const asset of Object.values(localManifest.architecture)) asset.url = `${origin}/${new URL(asset.url).pathname.split('/').pop()}`;
const bucket = path.join(temp, 'bucket source');
const recipe = path.join(bucket, 'bucket/zel.json');
let builds = 0;
async function verify(packageVersion, manager = 'scoop update zel') {
  const current = path.join(scoopRoot, 'apps/zel/current');
  assert.equal(path.basename(await realpath(current)), packageVersion);
  const executable = path.join(current, 'bin/zel.exe');
  const bytes = await readFile(executable);
  const pe = bytes.readUInt32LE(0x3c);
  assert.equal(bytes.toString('ascii', pe, pe + 4), 'PE\0\0');
  assert.equal(bytes.readUInt16LE(pe + 4), target.startsWith('aarch64') ? 0xaa64 : 0x8664);
  assert.equal(await readFile(path.join(current, 'share/zel/installation-manager'), 'utf8'), 'scoop\n');
  const shim = path.join(scoopRoot, 'shims/zel.exe');
  assert.equal((await run(shim, ['--version'])).trim(), `zel ${version}`);
  // Both normal Windows shells must reach the installed shim through PATH.
  for (const shell of ['powershell.exe', 'pwsh']) {
    assert.equal((await run(shell, ['-NoProfile', '-NonInteractive', '-Command', 'zel --version'])).trim(), `zel ${version}`);
  }
  for (const args of [['--check'], ['--preview'], [version, '--force'], [version, '--release-base-url', origin]]) {
    const result = await run(shim, ['upgrade', ...args], { code: 1 });
    assert(result.includes(manager), result);
    assert.deepEqual(await readFile(executable), bytes, 'Self-upgrade changed a managed executable');
  }
  const docs = path.join(temp, `manual-${++builds}`);
  await run(shim, ['new', 'portable', docs, '--name', 'Scoop acceptance']);
  await run(shim, ['check', docs, '--strict']);
  await run(shim, ['build', docs]);
  assert((await readFile(path.join(docs, 'site/index.html'), 'utf8')).includes('Scoop acceptance'));
  return { executable, bytes };
}
async function commitRecipe(value) {
  await writeFile(recipe, JSON.stringify(value, null, 4) + '\n');
  await run('git', ['-C', bucket, 'add', 'bucket/zel.json'], { log: false });
  await run('git', ['-C', bucket, '-c', 'user.name=Zellij CI', '-c', 'user.email=ci@example.invalid', 'commit', '-qm', 'Acceptance manifest'], { log: false });
}
async function assertUninstalled() {
  for (const name of ['apps/zel', 'shims/zel.exe', 'shims/zel.shim', 'shims/zel.ps1', 'shims/zel.cmd']) {
    await assert.rejects(stat(path.join(scoopRoot, name)), { code: 'ENOENT' }, name);
  }
}
try {
  assert.equal((await run('git', ['-C', source, 'rev-parse', 'HEAD'], { log: false })).trim(), pin);
  await cp(source, core, { recursive: true });
  for (const directory of ['shims', 'buckets']) await mkdir(path.join(scoopRoot, directory), { recursive: true });
  // Scoop supports this portable config layout. Core is pinned; no host profile or PATH edits.
  await writeFile(path.join(scoopRoot, 'config.json'), JSON.stringify({
    last_update: new Date().toISOString(), hold_update_until: new Date(Date.now() + 86400000).toISOString(),
    aria2_enabled: false, use_external_7zip: false, use_sqlite_cache: false,
  }));
  await mkdir(path.dirname(recipe), { recursive: true });
  await run('git', ['init', '--quiet', '--initial-branch=main', bucket], { log: false });
  await commitRecipe(localManifest);
  await scoop(['bucket', 'add', 'acceptance', pathToFileURL(bucket).href]);
  assert.deepEqual(JSON.parse(await readFile(path.join(scoopRoot, 'buckets/acceptance/bucket/zel.json'), 'utf8')), localManifest);
  await scoop(['install', 'acceptance/zel']);
  const guidance = process.env.ZEL_SCOOP_PUBLIC_VERSION && version === '0.2.0-rc.6' ? 'package manager' : 'scoop update zel';
  const initial = await verify(version, guidance);

  // Only the fixture's package revision changes; the real CLI/archive version does not.
  // This tests ordinary Scoop version updates without publishing a fictitious release.
  const updated = { ...structuredClone(localManifest), version: `${version}.1` };
  for (const asset of Object.values(updated.architecture)) asset.hash = '0'.repeat(64);
  await commitRecipe(updated);
  await scoop(['update']); // Sync the actual local Git bucket; held Core remains pinned.
  const rejected = await scoop(['update', 'zel'], { code: 1 });
  assert.match(rejected, /hash check failed|hash mismatch/i);
  assert.deepEqual(await readFile(initial.executable), initial.bytes, 'Bad checksum replaced the installed binary');
  await verify(version, guidance);

  updated.architecture = structuredClone(localManifest.architecture);
  await commitRecipe(updated);
  await scoop(['update']);
  await scoop(['update', 'zel']);
  await verify(updated.version, guidance);
  await scoop(['uninstall', 'zel']);
  await assertUninstalled();
  await scoop(['bucket', 'rm', 'acceptance']);

  if (process.env.ZEL_SCOOP_PUBLIC_BUCKET === 'true') {
    await scoop(['bucket', 'add', 'mylife-inc', 'https://github.com/mylife-inc/scoop-bucket']);
    const publicManifest = JSON.parse(await readFile(path.join(scoopRoot, 'buckets/mylife-inc/bucket/zel.json'), 'utf8'));
    assert.deepEqual(publicManifest, manifest, 'Public bucket differs from verified release manifest');
    await scoop(['install', 'mylife-inc/zel']);
    await verify(version, guidance);
    await scoop(['uninstall', 'zel']);
    await assertUninstalled();
    await scoop(['bucket', 'rm', 'mylife-inc']);
  }
  console.log(`${target}/${scoopShell}: Scoop shim, native architecture, ownership guard, portable build, checksum-failure preservation, package-revision update and uninstall passed.`);
} finally {
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  // Everything above lives under this newly created disposable directory.
  await rm(temp, { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
}
