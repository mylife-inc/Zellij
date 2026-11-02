// Runs on each native runner after publication, and can be dispatched again later.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { downloadRelease } from './download-release.mjs';

const [version = process.env.VERSION, target = process.env.TARGET,
  repository = process.env.RELEASE_REPOSITORY, shared = process.env.SHARED_RELEASE || 'false'] = process.argv.slice(2);
assert(['true', 'false'].includes(shared), 'Shared repository selection must be true or false');
execFileSync(process.execPath, ['scripts/package-native.mjs', version, '--check-version'], { stdio: 'inherit' });
const root = await mkdtemp(path.join(tmpdir(), 'zel-published-'));
const windows = process.platform === 'win32';
const installers = Object.fromEntries(await Promise.all(['install.sh', 'install.ps1'].map(async name => [name, await readFile(path.join('scripts', name))])));
const sources = [{ repository, scope: '', name: 'development' }];
if (shared === 'true') sources.push({ repository: 'mylife-inc/releases', scope: 'zellij', name: 'shared' });
const env = { ...process.env };
delete env.GITHUB_TOKEN;
delete env.GH_TOKEN;
delete env.ZEL_INSECURE_SKIP_CHECKSUM;
let previous;
try {
  for (const source of sources) {
    const destination = path.join(root, source.name);
    const result = await downloadRelease({ version, target, ...source, destination, installers });
    if (previous) {
      for (const name of Object.keys(result.contents)) assert.deepEqual(result.contents[name], previous.contents[name], `Release repositories disagree: ${name}`);
    }
    // Exercise the public download URLs through the published installer itself.
    const installDir = path.join(root, `installed-${source.name}`);
    const installer = path.join(destination, windows ? 'install.ps1' : 'install.sh');
    const args = windows ? ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', installer] : [installer];
    const binary = path.join(installDir, windows ? 'zel.exe' : 'zel');
    for (const shell of windows ? ['powershell.exe', 'pwsh'] : ['sh']) {
      execFileSync(shell, args, { env: { ...env, ZEL_VERSION: version, ZEL_RELEASE_BASE_URL: result.base, ZEL_INSTALL_DIR: installDir }, stdio: 'inherit', timeout: 180000 });
      assert.equal(execFileSync(binary, ['--version'], { env, encoding: 'utf8', timeout: 10000 }).trim(), `zel ${version.slice(1)}`);
    }
    // Reuse the full archive/installer/upgrade/build/serve/view acceptance suite
    // once; both repositories must contain byte-identical release artifacts.
    if (!previous) execFileSync(process.execPath, ['scripts/check-native-install.mjs', destination, destination], {
      env: { ...env, TARGET: target }, stdio: 'inherit', timeout: 600000,
    });
    previous = result;
    console.log(`${source.repository}: published ${version} installed and verified for ${target}`);
  }
} finally {
  await rm(root, { recursive: true, force: true });
}
