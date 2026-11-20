// Real WinGet lifecycle in a disposable native runner. No client documents.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, writeFile, rm, copyFile, realpath, cp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { wingetManifests, manifestYaml, wingetId } from './winget-manifest.mjs';
import { downloadRelease, downloadTaggedInstallers } from './download-release.mjs';

assert.equal(process.env.GITHUB_ACTIONS, 'true'); assert.equal(process.platform, 'win32');
const manager = process.env.ZEL_TEST_WINGET; assert(manager);
const target = process.env.TARGET; assert(['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'].includes(target));
const arch = target.startsWith('aarch64') ? 'arm64' : 'x64';
const version = (await readFile('cli/Cargo.toml', 'utf8')).match(/^version = "([^"]+)"/m)[1];
const tag = `v${version}`, previous = 'v0.2.0-rc.14'; assert.notEqual(tag, previous);
const root = await mkdtemp(path.join(tmpdir(), 'zel winget 日本 '));
const evidence = process.env.ZEL_WINGET_RECEIPT ? path.dirname(process.env.ZEL_WINGET_RECEIPT) : null;
if (evidence) await mkdir(evidence, { recursive: true });
const env = { ...process.env, ZELLIJ_HOME: path.join(root, 'resources') };
for (const key of Object.keys(env)) if (key.toLowerCase() === 'psmodulepath') delete env[key];
const dist = path.resolve(process.argv[2] || 'dist');
const sums = (await Promise.all((await readdir(dist)).filter(n => n.endsWith('.zip.sha256')).map(n => readFile(path.join(dist, n), 'utf8')))).join('');
const docs = wingetManifests(tag, sums);
const bytes = new Map();
for (const item of docs[`${wingetId}.installer.yaml`].Installers) {
  const name = new URL(item.InstallerUrl).pathname.split('/').pop(), data = await readFile(path.join(dist, name));
  assert.equal(createHash('sha256').update(data).digest('hex').toUpperCase(), item.InstallerSha256);
  bytes.set(name, data);
}
async function run(command, args, { expected = 0, quiet = false, extraEnv = {}, timeout = 180000 } = {}) {
  console.log(`Running ${path.basename(command)} ${args.join(' ')}`);
  return await new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: root, env: { ...env, ...extraEnv }, windowsHide: true, timeout });
    let out = '', err = '';
    child.stdout.on('data', d => { out += d; if (!quiet) process.stdout.write(d); });
    child.stderr.on('data', d => { err += d; if (!quiet) process.stderr.write(d); });
    child.once('error', reject);
    child.once('close', code => (expected === 'nonzero' ? code !== 0 && code !== null : code === expected) ? resolve({ out, err, code }) : reject(Error(`exit ${code}, expected ${expected}\n${out}\n${err}`)));
  });
}
const flags = ['--disable-interactivity', '--accept-source-agreements'];
const server = createServer((req, res) => {
  const data = bytes.get(new URL(req.url, 'http://localhost').pathname.slice(1));
  if (!data) { res.writeHead(404).end(); return; }
  res.writeHead(200, { 'Content-Length': data.length, 'Content-Type': 'application/octet-stream' }); res.end(req.method === 'HEAD' ? undefined : data);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
async function manifests(name, documents, local = true) {
  const folder = path.join(root, name); await mkdir(folder);
  for (const [filename, original] of Object.entries(documents)) {
    const document = structuredClone(original);
    if (document.Installers && local) for (const i of document.Installers) i.InstallerUrl = `${origin}/${new URL(i.InstallerUrl).pathname.split('/').pop()}`;
    await writeFile(path.join(folder, filename), manifestYaml(document));
  }
  return folder;
}
const location = path.join(root, 'custom install 日本');
const alias = path.join(process.env.LOCALAPPDATA, 'Microsoft/WinGet/Links/zel.exe');
// Local manifests have no community-source identity. WinGet records this exact
// product code in their UninstallString; --id is for catalog-correlated packages.
const productCode = `${wingetId}__DefaultSource`;
let executable;
const sentinel = path.join(env.ZELLIJ_HOME, 'cache/engines/keep.txt');
let installed = false, completed = false;
try {
  const publicManifest = await manifests('public-manifest', docs, false);
  await run(manager, ['validate', '--manifest', publicManifest]);
  const current = process.env.ZEL_WINGET_PUBLIC === 'true' ? publicManifest : await manifests('current', docs);
  // The old release is real and checked, not a relabelled current executable.
  const installers = await downloadTaggedInstallers({ version: previous, repository: 'mylife-inc/Zellij' });
  const old = await downloadRelease({ version: previous, target, repository: 'mylife-inc/Zellij', destination: path.join(root, 'previous'), installers });
  bytes.set(old.archive, old.contents[old.archive]);
  // rc.14 predates ownership protection: never generate a distributable
  // WinGet manifest for it. This local-only fixture tests a genuine upgrade.
  const oldDocs = structuredClone(docs);
  for (const doc of Object.values(oldDocs)) doc.PackageVersion = previous.slice(1);
  oldDocs[`${wingetId}.installer.yaml`].Installers = [{
    Architecture: arch, InstallerUrl: `${origin}/${old.archive}`,
    InstallerSha256: createHash('sha256').update(old.contents[old.archive]).digest('hex').toUpperCase(),
  }];
  const oldManifest = await manifests('old', oldDocs);
  await run(manager, ['install', '--manifest', oldManifest, '--architecture', arch, '--scope', 'user', '--accept-package-agreements', ...flags]); installed = true;
  executable = await realpath(alias);
  assert.equal((await run(executable, ['--version'])).out.trim(), `zel ${previous.slice(1)}`);
  await mkdir(path.dirname(sentinel), { recursive: true }); await writeFile(sentinel, 'retain cache');
  const badDocs = structuredClone(docs);
  for (const i of badDocs[`${wingetId}.installer.yaml`].Installers) i.InstallerSha256 = '0'.repeat(64);
  const badManifest = await manifests('bad-checksum', badDocs);
  const beforeUpgrade = await readFile(executable);
  const bad = await run(manager, ['upgrade', '--manifest', badManifest, '--architecture', arch, '--accept-package-agreements', ...flags], { expected: 'nonzero' });
  assert.match(bad.out + bad.err, /Installer hash does not match/i);
  assert((await readFile(executable)).equals(beforeUpgrade), 'Failed checksum changed the installed CLI');
  await run(manager, ['upgrade', '--manifest', current, '--architecture', arch, '--accept-package-agreements', ...flags]);
  assert.equal((await run(executable, ['--version'])).out.trim(), `zel ${version}`);
  assert.equal((await run(alias, ['--version'])).out.trim(), `zel ${version}`);
  const before = await readFile(executable);
  for (const args of [['upgrade', '--check'], ['upgrade', '--force', version, '--release-base-url', 'http://127.0.0.1:1']]) {
    const rejected = await run(alias, args, { expected: 1 });
    assert.match(rejected.err + rejected.out, /WinGet manages this installation/);
    assert.match(rejected.err + rejected.out, /winget upgrade --id Shebka\.Zel --exact/);
    assert((await readFile(executable)).equals(before));
  }
  const loose = path.join(root, 'standalone.exe'); await copyFile(executable, loose);
  const standalone = await run(loose, ['upgrade', '--check', version]);
  assert.doesNotMatch(standalone.out + standalone.err, /WinGet manages/);
  await mkdir(path.join(root, 'docs'));
  await writeFile(path.join(root, 'docs/index.md'), '# WinGet installed CLI\n');
  await run(executable, ['build', path.join(root, 'docs'), '--target', 'portable', '--out', path.join(root, 'site')], { extraEnv: { PATH: '' } });
  assert.match(await readFile(path.join(root, 'site/index.html'), 'utf8'), /WinGet installed CLI/);
  await run(manager, ['list', '--name', 'Zellij CLI', '--exact', ...flags]);
  await run(manager, ['uninstall', '--product-code', productCode, '--exact', ...flags]); installed = false;
  await assert.rejects(readFile(executable), { code: 'ENOENT' });
  await assert.rejects(readFile(alias), { code: 'ENOENT' });
  assert.equal(await readFile(sentinel, 'utf8'), 'retain cache');
  // A fresh custom-location user install exercises the same ownership guard.
  await run(manager, ['install', '--manifest', current, ...(process.env.ZEL_WINGET_PUBLIC === 'true' ? [] : ['--architecture', arch]), '--scope', 'user', '--location', location, '--accept-package-agreements', ...flags]); installed = true;
  assert.equal(await realpath(alias), await realpath(path.join(location, 'zel.exe')));
  assert.equal((await run(alias, ['--version'])).out.trim(), `zel ${version}`);
  assert.equal(JSON.parse((await run(alias, ['version', '--json'])).out).cli.target, target, 'WinGet must select the native architecture');
  const customGuard = await run(alias, ['upgrade', '--check'], { expected: 1 });
  assert.match(customGuard.out + customGuard.err, /WinGet manages this installation/);
  await run(manager, ['uninstall', '--product-code', productCode, '--exact', ...flags]); installed = false;
  await assert.rejects(readFile(alias), { code: 'ENOENT' });
  assert.equal(await readFile(sentinel, 'utf8'), 'retain cache');
  // Machine scope is selected by WinGet, not hard-coded in portable manifests.
  // Exercise both registry views on the corresponding native runners.
  const machine = path.join(root, 'machine install 日本');
  const machineExe = path.join(machine, 'zel.exe');
  await run(manager, ['install', '--manifest', current, '--architecture', arch, '--scope', 'machine', '--location', machine, '--accept-package-agreements', ...flags]); installed = true;
  assert.equal((await run(machineExe, ['--version'])).out.trim(), `zel ${version}`);
  const machineGuard = await run(machineExe, ['upgrade', '--check'], { expected: 1 });
  assert.match(machineGuard.out + machineGuard.err, /WinGet manages this installation/);
  await run(manager, ['uninstall', '--product-code', productCode, '--exact', '--scope', 'machine', ...flags]); installed = false;
  await assert.rejects(readFile(machineExe), { code: 'ENOENT' });
  assert.equal(await readFile(sentinel, 'utf8'), 'retain cache');
  const receipt = { version, previous, target, harnessCommit: process.env.GITHUB_SHA, binaryRun: process.env.ZEL_WINGET_BINARY_RUN, winGet: '1.29.380', manifestValidated: true, realUpgrade: true, customLocation: true, defaultLocation: true, machineScope: true, launcherVerified: true, checksumFailurePreserved: true, selfUpgradeBlocked: true, standaloneUnaffected: true, portableBuildWithoutTools: true, cacheRetained: true, uninstall: true, nativeArchitecture: true, publicUrlInstalls: process.env.ZEL_WINGET_PUBLIC === 'true', publicCatalogSubmitted: false };
  if (process.env.ZEL_WINGET_RECEIPT) await writeFile(process.env.ZEL_WINGET_RECEIPT, JSON.stringify(receipt, null, 2) + '\n');
  console.log(JSON.stringify(receipt)); completed = true;
} finally {
  if (!completed) {
    // Only this test package's public metadata is collected, never other apps.
    await run('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command',
      "$roots = @('HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall', 'HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall', 'HKLM:\\Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall'); Get-ChildItem $roots -ErrorAction SilentlyContinue | Where-Object { $_.PSChildName -like 'Shebka.Zel_*' } | ForEach-Object { Get-ItemProperty $_.PSPath } | Select-Object PSPath,DisplayName,DisplayVersion,WinGetPackageIdentifier,WinGetInstallerType,InstallLocation,TargetFullPath | ConvertTo-Json -Depth 2"
    ]).catch(error => console.error(`Registry diagnostics: ${error.message}`));
  }
  if (installed) await run(manager, ['uninstall', '--product-code', productCode, '--exact', ...flags]).catch(e => { console.error(`Cleanup failed: ${e.message}`); process.exitCode = 1; });
  if (evidence) {
    const logs = path.join(process.env.LOCALAPPDATA, 'Packages/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe/LocalState/DiagOutputDir');
    await cp(logs, path.join(evidence, 'logs'), { recursive: true }).catch(error => console.error(`WinGet diagnostics: ${error.message}`));
  }
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  await rm(root, { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
}
