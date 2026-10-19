// Test the shipped archive through the real installer, using a local HTTP release mirror.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { copyFile, mkdir, mkdtemp, readFile, readdir, rename, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const windows = process.platform === 'win32';
const dist = path.resolve(process.argv[2] || 'dist');
const target = process.env.TARGET || `${process.arch === 'arm64' ? 'aarch64' : 'x86_64'}-${windows ? 'pc-windows-msvc' : process.platform === 'darwin' ? 'apple-darwin' : 'unknown-linux-gnu'}`;
const archives = (await readdir(dist)).filter(file => file.endsWith(`${target}.${windows ? 'zip' : 'tar.gz'}`));
assert.equal(archives.length, 1, `Expected one ${target} archive`);
const archive = archives[0];
const version = archive.slice(4, -(`-${target}.${windows ? 'zip' : 'tar.gz'}`).length);
const bytes = await readFile(path.join(dist, archive));
const sha = data => createHash('sha256').update(data).digest('hex');
assert.equal((await readFile(path.join(dist, `${archive}.sha256`), 'utf8')).trim(), `${sha(bytes)}  ${archive}`);
const temp = await mkdtemp(path.join(tmpdir(), 'zel-native-install-'));
const installation = path.join(temp, 'Installed tools café 日本');
const executable = path.join(installation, windows ? 'zel.exe' : 'zel');
const consoleExe = path.join(temp, 'console.exe');
const active = new Set();
let badArchive;
let nextArchive;
const requests = [];
const server = createServer((request, response) => {
  requests.push({ url: request.url, auth: !!request.headers.authorization });
  const [scenario, file] = new URL(request.url, 'http://localhost').pathname.slice(1).split('/');
  const name = ['version-mismatch', 'next'].includes(scenario) ? archive.replace(version, 'v999.0.0') : archive;
  const data = scenario === 'next' ? nextArchive : scenario === 'unsafe-archive' ? badArchive : bytes;
  if (file === name) {
    response.writeHead(200, { 'Content-Type': 'application/octet-stream' });
    if (scenario === 'slow') { response.write(data.subarray(0, 1024)); setTimeout(() => response.end(data.subarray(1024)), 2000); }
    else response.end(data);
    return;
  }
  if ((file === 'SHA256SUMS' && scenario !== 'single') || file === `${name}.sha256`) {
    if (scenario === 'missing') { response.writeHead(404); response.end(); return; }
    response.writeHead(200, { 'Content-Type': 'application/octet-stream' });
    response.end(`${scenario === 'corrupt' ? '0'.repeat(64) : sha(data)}  ${name}\n`); return;
  }
  response.writeHead(404); response.end();
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
function start(command, args, env = process.env, cwd) {
  const child = spawn(command, args, { env, cwd, stdio: ['pipe', 'pipe', 'pipe'] });
  const state = { child, stdout: '', stderr: '', exit: undefined };
  active.add(state);
  child.stdout.on('data', bytes => state.stdout += bytes);
  child.stderr.on('data', bytes => state.stderr += bytes);
  child.on('error', error => state.stderr += error.message);
  state.closed = new Promise(resolve => child.once('close', code => { state.exit = code; active.delete(state); resolve(code); }));
  return state;
}
async function run(command, args, env = process.env, succeeds = true, cwd) {
  const state = start(command, args, env, cwd);
  const timeout = setTimeout(() => state.child.kill(), 90000);
  try {
    const code = await state.closed;
    assert.equal(code === 0, succeeds, `${command}: ${state.stdout}\n${state.stderr}`);
    return state;
  } finally { clearTimeout(timeout); }
}
async function install(scenario, shell, succeeds = true, dir = installation, requested = version, mirror = `${origin}/${scenario}`) {
  const env = { ...process.env, GITHUB_TOKEN: 'fixture-not-a-credential', GH_TOKEN: 'fixture-not-a-credential', ZEL_VERSION: requested, ZEL_INSTALL_DIR: dir, ZEL_RELEASE_BASE_URL: mirror };
  delete env.ZEL_INSECURE_SKIP_CHECKSUM;
  return run(shell, windows ? ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', path.resolve('scripts/install.ps1')] : [path.resolve('scripts/install.sh')], env, succeeds);
}
const until = async (predicate, message) => {
  const end = Date.now() + 15000;
  while (!predicate()) { if (Date.now() > end) throw new Error(message); await delay(25); }
};
try {
  // Both supported Windows shells execute the same installer. No persistent PATH changes.
  const shells = windows ? ['powershell.exe', 'pwsh'] : ['sh'];
  if (windows) {
    execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleExe]);
    const unsafeZip = path.join(temp, 'unsafe.zip');
    execFileSync('pwsh', ['-NoProfile', '-File', 'scripts/package-windows.ps1', '-Binary', path.join(dist, archive), '-Archive', unsafeZip, '-EntryName', '../outside']);
    badArchive = await readFile(unsafeZip);
  } else {
    const unsafeDir = path.join(temp, 'unsafe'); await mkdir(unsafeDir);
    await symlink('../outside', path.join(unsafeDir, 'zel'));
    const unsafeTar = path.join(temp, 'unsafe.tar.gz');
    execFileSync('tar', ['-czf', unsafeTar, '-C', unsafeDir, 'zel']);
    badArchive = await readFile(unsafeTar);
  }
  await mkdir(installation);
  await writeFile(path.join(installation, 'zel.new'), 'authored file');
  for (const shell of shells) {
    await install('combined', shell);
    const installed = await readFile(executable);
    // Verify the archive's actual CPU architecture, including native ARM64 Windows.
    if (windows) {
      const pe = installed.readUInt32LE(0x3c);
      assert.equal(installed.toString('ascii', pe, pe + 4), 'PE\0\0');
      assert.equal(installed.readUInt16LE(pe + 4), target.startsWith('aarch64') ? 0xaa64 : 0x8664);
    } else if (process.platform === 'linux') {
      assert.equal(installed.toString('ascii', 1, 4), 'ELF');
      assert.equal(installed.readUInt16LE(18), target.startsWith('aarch64') ? 183 : 62);
    } else {
      assert.equal(installed.readUInt32LE(4), target.startsWith('aarch64') ? 0x100000c : 0x1000007);
    }
    await install('single', shell);
    for (const scenario of ['corrupt', 'missing', 'unsafe-archive', 'version-mismatch']) {
      await install(scenario, shell, false, installation, scenario === 'version-mismatch' ? 'v999.0.0' : version);
      assert.deepEqual(await readFile(executable), installed, `${scenario} must preserve the installed binary`);
    }
    await install('combined', shell, false, installation, 'v../invalid');
    for (const mirror of [`${origin}/combined?query=1`, origin.replace('http://', 'http://localhost:80@') + '/combined']) {
      const before = requests.length;
      const failure = await install('combined', shell, false, installation, version, mirror);
      assert.match(failure.stderr, /Release asset URLs|ReleaseBaseUrl/);
      assert.equal(requests.length, before, 'Malformed mirror URLs must fail before downloading');
    }
    assert.deepEqual(await readFile(executable), installed);
    assert.equal(await readFile(path.join(installation, 'zel.new'), 'utf8'), 'authored file');
    assert.deepEqual((await readdir(installation)).sort(), [path.basename(executable), 'zel.new'].sort());
    console.log(`${shell}: combined/per-file checksums, replacement, malformed archive/version refusal and preservation passed.`);
  }
  assert(requests.every(request => !request.auth), 'No credentials may be sent to release mirrors');
  const blocked = path.join(temp, 'Blocked installation');
  await mkdir(path.join(blocked, path.basename(executable)), { recursive: true });
  await writeFile(path.join(blocked, path.basename(executable), 'keep'), 'authored');
  await install('combined', shells[0], false, blocked);
  assert.equal(await readFile(path.join(blocked, path.basename(executable), 'keep'), 'utf8'), 'authored');
  const packageFailure = await run(process.execPath, ['scripts/package-native.mjs', 'v999.0.0', target, path.join(temp, 'bad-package')], process.env, false);
  assert.match(packageFailure.stderr, /committed workspace version/);
  const env = { ...process.env, PATH: path.join(temp, 'no-external-tools') };
  assert.equal((await run(executable, ['--version'], env)).stdout.trim(), `zel ${version.slice(1)}`);
  const details = JSON.parse((await run(executable, ['version', '--json'], env)).stdout);
  assert.equal(details.cli.version, version.slice(1)); assert.equal(details.cli.target, target);
  assert.equal(details.configuration_version, 1); assert.match(details.content_schema_sha256, /^[a-f0-9]{64}$/);
  const beforeUpgrade = await readFile(executable);
  const upgradeEnv = { ...env, GITHUB_TOKEN: 'fixture-not-a-credential', GH_TOKEN: 'fixture-not-a-credential' };
  delete upgradeEnv.ZEL_RELEASE_BASE_URL;
  const upgradeArgs = (scenario, requested = version) => ['upgrade', requested, '--force', '--release-base-url', `${origin}/${scenario}`];
  const noDownload = requests.length;
  await run(executable, ['upgrade', version, '--check'], upgradeEnv);
  await run(executable, ['upgrade', version], upgradeEnv);
  await run(executable, ['upgrade', 'v0.0.0', '--force'], upgradeEnv, false);
  await run(executable, ['upgrade', version, '--force', '--json'], upgradeEnv, false);
  assert.equal(requests.length, noDownload, 'check, current-version and rejected commands must not download');
  for (const scenario of ['corrupt', 'missing', 'unsafe-archive', 'version-mismatch']) {
    const failure = await run(executable, upgradeArgs(scenario, scenario === 'version-mismatch' ? 'v999.0.0' : version), upgradeEnv, false);
    assert.match(failure.stderr, new RegExp({corrupt:'checksum mismatch', missing:'no valid checksum', 'unsafe-archive':'archive must contain', 'version-mismatch':'binary version does not match'}[scenario]));
    assert.deepEqual(await readFile(executable), beforeUpgrade, `${scenario} upgrade must preserve installed binary`);
  }
  for (const scenario of ['combined', 'single']) {
    assert.match((await run(executable, upgradeArgs(scenario), upgradeEnv)).stdout, /Upgraded /);
    assert.deepEqual(await readFile(executable), beforeUpgrade, 'verified archive replaced the executable');
    assert.equal((await run(executable, ['--version'], env)).stdout.trim(), `zel ${version.slice(1)}`);
  }
  const beforeSlow = requests.length;
  const slowArgs = upgradeArgs('slow');
  const slow = start(windows ? consoleExe : executable, windows ? [executable, ...slowArgs] : slowArgs, upgradeEnv);
  await until(() => requests.slice(beforeSlow).some(r => r.url === `/slow/${archive}`) || slow.exit !== undefined, 'Upgrade download never started');
  assert.equal(slow.exit, undefined, slow.stderr);
  if (windows) slow.child.stdin.end('stop\n'); else slow.child.kill('SIGINT');
  await until(() => slow.exit !== undefined, 'Upgrade cancellation timed out');
  assert.notEqual(slow.exit, 0, slow.stdout);
  assert.match(slow.stderr, /cancelled/);
  assert.deepEqual(await readFile(executable), beforeUpgrade, 'cancelled upgrade preserves installed executable');
  assert(requests.every(request => !request.auth), 'Upgrades must not forward credentials to mirrors');
  for (const entry of await readdir(installation)) {
    if (entry.startsWith('.zel-upgrade-')) {
      assert(windows, 'Unix staging directories are removed after success/failure');
      assert.deepEqual(await readdir(path.join(installation, entry)), ['zel.previous.exe']);
      assert.deepEqual(await readFile(path.join(installation, entry, 'zel.previous.exe')), beforeUpgrade);
    }
  }
  console.log(`${target}: version reporting and native self-upgrade, checksum/archive/version refusal, cancellation and preservation passed.`);
  const fixtureDir = path.join(temp, 'fixture'); await mkdir(fixtureDir);
  const fixture = path.join(fixtureDir, windows ? 'zel.exe' : 'zel');
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/maintenance-tool.rs', '-o', fixture]);
  const runtime = path.join(fixtureDir, windows ? 'podman.exe' : 'podman'); await copyFile(fixture, runtime);
  const runtimeEnv = { ...env };
  const selectors = ['PATH', 'DOCKER_HOST', 'DOCKER_CONTEXT', 'CONTAINER_HOST', 'CONTAINER_CONNECTION', 'CONTAINERD_ADDRESS', 'CONTAINERD_NAMESPACE'];
  for (const key of Object.keys(runtimeEnv)) if (selectors.includes(key.toUpperCase())) delete runtimeEnv[key];
  Object.assign(runtimeEnv, { PATH: fixtureDir, CONTAINER_CONNECTION: 'isolated-fixture', ZEL_FIXTURE_LOG: path.join(temp, 'pull.log'), ZELLIJ_IMAGE: 'registry.example.test/engine:configured' });
  await run(executable, ['pull'], runtimeEnv);
  assert.equal(await readFile(runtimeEnv.ZEL_FIXTURE_LOG, 'utf8'), runtimeEnv.ZELLIJ_IMAGE);
  const digestRef = 'registry.example.test/engine@sha256:' + 'a'.repeat(64);
  await run(executable, ['pull', '--image', digestRef], runtimeEnv);
  assert.equal(await readFile(runtimeEnv.ZEL_FIXTURE_LOG, 'utf8'), digestRef);
  const pullFailure = await run(executable, ['pull'], { ...runtimeEnv, ZEL_FIXTURE_EXIT: '42' }, false);
  assert.equal(pullFailure.exit, 3);
  assert.equal((await run(executable, ['pull'], { ...runtimeEnv, ZEL_FIXTURE_UNAVAILABLE: '1' }, false)).exit, 4);
  for (const args of [['pull', '--image', '--help=true'], ['pull', '--image', 'invalid image'], ['pull', '--json']]) {
    assert.equal((await run(executable, args, runtimeEnv, false)).exit, 2);
  }
  assert.equal(await readFile(runtimeEnv.ZEL_FIXTURE_LOG, 'utf8'), runtimeEnv.ZELLIJ_IMAGE);
  const future = path.join(temp, windows ? 'next.zip' : 'next.tar.gz');
  if (windows) execFileSync('pwsh', ['-NoProfile', '-File', 'scripts/package-windows.ps1', '-Binary', fixture, '-Archive', future]);
  else execFileSync('tar', ['-czf', future, '-C', fixtureDir, 'zel'], { env: { ...process.env, COPYFILE_DISABLE: '1' } });
  nextArchive = await readFile(future);
  const copyDir = path.join(temp, 'Separate installation'); await mkdir(copyDir);
  const copyExe = path.join(copyDir, path.basename(executable)); await copyFile(executable, copyExe);
  assert.match((await run(copyExe, ['upgrade', 'v999.0.0', '--release-base-url', `${origin}/next`], upgradeEnv)).stdout, /Upgraded /);
  assert.equal((await run(copyExe, ['--version'], env)).stdout.trim(), 'zel 999.0.0');
  assert.deepEqual(await readFile(executable), beforeUpgrade, 'upgrades affect only their own executable');
  console.log(`${target}: selected-runtime pull, failure propagation and upgrade to a higher native version passed.`);
  const source = path.join(temp, 'Docs 日本');
  const output = path.join(temp, 'Moved site café');
  await run(executable, ['new', 'portable', source, '--name', 'Installed documentation'], env);
  const report = JSON.parse((await run(executable, ['check', '--strict', '--json'], env, true, source)).stdout);
  assert.equal(report.ok, true); assert.deepEqual(report.diagnostics, []);
  await run(executable, ['build'], env, true, source);
  await rename(path.join(source, 'site'), output);
  assert((await readFile(path.join(output, 'index.html'), 'utf8')).includes('guide/getting-started.html#write-your-first-page'));
  assert((await readFile(path.join(output, 'guide/getting-started.html'), 'utf8')).includes('../index.html#build-and-share'));
  assert((await readFile(path.join(output, '_zellij/search.js'), 'utf8')).includes('Installed documentation'));
  for (const args of [['serve', output, '--port', '0'], ['view', source, '--out', output, '--port', '0', '--no-open']]) {
    const state = start(windows ? consoleExe : executable, windows ? [executable, ...args] : args, env);
    await until(() => /Serving at (http:\/\/127\.0\.0\.1:\d+\/)/.test(state.stdout) || state.exit !== undefined, 'Preview startup timed out');
    const url = state.stdout.match(/Serving at (http:\/\/127\.0\.0\.1:\d+\/)/)?.[1];
    assert(url, `${state.stdout}\n${state.stderr}`);
    for (const page of ['', 'guide/getting-started.html', 'assets/workflow.svg']) assert.equal((await fetch(url + page)).status, 200);
    if (windows) state.child.stdin.end('stop\n'); else state.child.kill('SIGINT');
    await until(() => state.exit !== undefined, 'Ctrl-C shutdown timed out');
    assert.equal(state.exit, 0, state.stderr); assert.match(state.stdout, /Stopped\./);
  }
  console.log(`${target}: installed release version/architecture, native starter/check/build, moved output, HTTP serve/view and real Ctrl-C passed.`);
} finally {
  for (const state of active) { if (windows) state.child.stdin.end('stop\n'); else state.child.kill(); }
  await Promise.all([...active].map(state => state.closed));
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  await rm(temp, { recursive: true, force: true });
}
