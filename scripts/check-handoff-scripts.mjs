// Public-fixture coverage only. Never upload client documentation to this runner.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, copyFile, readdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { setTimeout as delay } from 'node:timers/promises';
import { downloadRelease, downloadTaggedInstallers } from './download-release.mjs';

assert.equal(process.platform, 'win32', 'Run this harness on native Windows');
const version = process.env.VERSION || 'v0.2.0-rc.14';
const target = process.env.TARGET;
assert(['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'].includes(target));
const shellName = process.env.ZEL_HANDOFF_SHELL || 'pwsh';
assert(['powershell.exe', 'pwsh'].includes(shellName));
const shell = execFileSync('where.exe', [shellName], { encoding: 'utf8' }).trim().split(/\r?\n/)[0];
const root = await mkdtemp(path.join(tmpdir(), 'zel handoff 日本 '));
const home = path.join(root, 'resources'), content = path.join(root, 'zellij/content');
const script = path.join(root, 'generate.ps1');
const env = { ...process.env, ZELLIJ_HOME: home, PATH: '' };
for (const key of Object.keys(env)) {
  if (['psmodulepath', 'path'].includes(key.toLowerCase())) delete env[key];
  if (/^(GH_TOKEN|GITHUB_TOKEN|ZEL_ENGINE_CACHE|ZEL_ENGINE_URL|ZEL_INSECURE_SKIP_CHECKSUM)$/.test(key)) delete env[key];
}
env.PATH = '';
const active = new Set();
const consoleRunner = path.join(root, 'console.exe');
const source = path.join(content, 'guide/index.mdx');
const original = '# Original\n\n[Next](/guide/next)\n\n```mermaid\ngraph LR\n A --> B\n```\n';
const siteConfig = 'name: Handoff fixture\nhome: guide\n';
function start(args, persistent = false) {
  const argv = ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', script, ...args];
  const state = { out: '', err: '', exit: null };
  state.child = spawn(persistent ? consoleRunner : shell, persistent ? [shell, ...argv] : argv,
    { env, cwd: tmpdir(), stdio: ['pipe', 'pipe', 'pipe'] });
  state.child.stdout.on('data', d => { state.out += d; });
  state.child.stderr.on('data', d => { state.err += d; });
  state.closed = new Promise((resolve, reject) => {
    state.child.once('error', reject);
    state.child.once('close', (code, signal) => { state.exit = { code, signal }; resolve(state.exit); });
  });
  active.add(state);
  return state;
}
async function until(fn, label, state, timeout = 120000) {
  for (const end = Date.now() + timeout; Date.now() < end;) {
    if (await fn()) return;
    assert(!state?.exit, `${label}: process exited\n${state?.out}\n${state?.err}`);
    await delay(100);
  }
  throw new Error(`${label}: timed out\n${state?.out}\n${state?.err}`);
}
async function run(args, expected = 0) {
  const state = start(args);
  await until(() => state.exit, 'command completion');
  await state.closed; active.delete(state);
  assert.equal(state.exit.code, expected, `${args.join(' ')}\n${state.out}\n${state.err}`);
  return state;
}
async function stop(state) {
  state.child.stdin.end('stop\n');
  await until(() => state.exit, 'Ctrl-C shutdown', null, 15000);
  await state.closed; active.delete(state);
  // PowerShell itself may return its interrupted-pipeline status. The native
  // child must report graceful termination and release its server below.
  assert.match(state.out, /Stopped\./, state.err);
}
async function snapshot() {
  const site = path.join(root, 'site');
  return Object.fromEntries(await Promise.all((await readdir(site, { recursive: true, withFileTypes: true }))
    .filter(e => e.isFile()).map(async e => {
      const filename = path.join(e.parentPath, e.name);
      return [path.relative(site, filename).replaceAll(path.sep, '/'), (await readFile(filename)).toString('base64')];
    })));
}
try {
  const installers = await downloadTaggedInstallers({ version, repository: 'mylife-inc/Zellij' });
  const downloaded = await downloadRelease({ version, target, repository: 'mylife-inc/Zellij', destination: path.join(root, 'download'), installers });
  const unpacked = path.join(root, 'unpacked'); await mkdir(unpacked);
  execFileSync('tar.exe', ['-xf', path.join(root, 'download', downloaded.archive), '-C', unpacked]);
  const binaryName = target.startsWith('aarch64') ? 'zel-windows-arm64.exe' : 'zel-windows-x64.exe';
  await copyFile(path.join(unpacked, 'zel.exe'), path.join(root, binaryName));
  assert.equal(execFileSync(path.join(root, binaryName), ['--version'], { env, encoding: 'utf8' }).trim(), `zel ${version.slice(1)}`);
  await copyFile('scripts/fixtures/handoff/generate.ps1', script);
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleRunner]);
  await mkdir(path.join(content, 'guide'), { recursive: true });
  await writeFile(path.join(root, 'zellij/zel.yaml'), 'version: 1\nrenderer: hosted\nbuild: {out: ../site}\n');
  await writeFile(path.join(content, 'site.yaml'), siteConfig);
  await writeFile(path.join(content, 'guide/_sidebar.yaml'), 'nav: [index, next]\n');
  await writeFile(source, original);
  await writeFile(path.join(content, 'guide/next.mdx'), '# Next\n\nFixture text.\n');
  assert.match((await run(['--help'])).out, /Usage:/);
  assert.doesNotMatch((await run(['build', '--help'])).out, /Ready:/);
  await run(['unknown-operation'], 2);
  assert.match((await run([])).out, /Ready:/); // Cold cache and default operation.
  const before = await snapshot();
  assert(before['guide/index.html']);
  assert.match(Buffer.from(before['guide/index.html'], 'base64').toString(), /zellij-mermaid/);
  assert.match((await run(['build', '--offline'])).out, /Ready:/);
  assert.deepEqual(await snapshot(), before, 'Offline regeneration must preserve every generated byte');
  let checked = await run(['check', '--deep', '--strict', '--offline', '--json']);
  assert.equal(checked.err, ''); assert.equal(JSON.parse(checked.out).ok, true); assert.equal(JSON.parse(checked.out).pages, 2);
  await writeFile(path.join(content, 'site.yaml'), siteConfig + 'unknownOption: true\n');
  checked = await run(['check', '--strict', '--offline', '--json'], 3);
  assert(JSON.parse(checked.out).diagnostics.some(d => d.severity === 'warning'));
  await writeFile(path.join(content, 'site.yaml'), siteConfig);
  await writeFile(source, '---\ntitle: [\n---\n# Broken\n');
  checked = await run(['check', '--offline', '--json'], 2); assert.equal(JSON.parse(checked.out).ok, false);
  const failed = start(['build', '--offline']); await until(() => failed.exit, 'failed build'); await failed.closed; active.delete(failed);
  assert.notEqual(failed.exit.code, 0); assert.doesNotMatch(failed.out, /Ready:/); assert.deepEqual(await snapshot(), before);
  const watch = start(['watch', '--offline'], true);
  await until(() => watch.out.includes('Validation failed;'), 'invalid edit watched', watch);
  await writeFile(source, original);
  await until(() => watch.out.includes('Validation passed.'), 'watch recovered', watch); await stop(watch);
  const preview = start(['view', '--watch', '--offline', '--no-open', '--port', '0'], true);
  await until(() => preview.out.includes('Serving at '), 'preview starts', preview);
  const url = preview.out.match(/Serving at (http:\/\/\S+)/)[1];
  const status = async () => (await fetch(`${url}_zellij/preview.json`)).json();
  assert.equal((await fetch(url)).status, 200); const previous = await status();
  await writeFile(source, original.replace('Original', 'Changed'));
  await until(async () => (await status()).version !== previous.version, 'live rebuild', preview);
  assert.match(await (await fetch(`${url}guide/`)).text(), /Changed/);
  await stop(preview);
  await assert.rejects(fetch(url), 'Ctrl-C must release the server');
  const serve = start(['serve', '--port', '0'], true);
  await until(() => serve.out.includes('Serving at '), 'existing site served', serve);
  const served = serve.out.match(/Serving at (http:\/\/\S+)/)[1]; assert.equal((await fetch(served)).status, 200); await stop(serve);
  console.log(JSON.stringify({ version, target, shell: shellName, publicFixtureOnly: true, emptyPath: true, coldBuild: true, offlineIdentical: true, json: true, preservedExitCodes: [2, 3], failedBuildPreservesSite: true, watchRecovery: true, livePreview: true, ctrlC: true, serve: true }));
} finally {
  for (const state of active) { if (!state.exit) state.child.stdin.end('stop\n'); await state.closed; }
  await rm(root, { recursive: true, force: true });
}
