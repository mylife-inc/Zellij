// Runs from the tagged checkout after publication, using anonymous public URLs.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { downloadRelease } from './download-release.mjs';

const { VERSION: version, TARGET: target, RELEASE_REPOSITORY: repository, SHARED_RELEASE: shared = 'false' } = process.env;
assert(['true', 'false'].includes(shared));
execFileSync(process.execPath, ['scripts/package-native.mjs', version, '--check-version'], { stdio: 'inherit' });
const root = await mkdtemp(path.join(tmpdir(), 'zel-published-pdf-'));
const windows = process.platform === 'win32';
const installers = Object.fromEntries(await Promise.all(['install.sh', 'install.ps1'].map(async name =>
  [name, await readFile(path.join('scripts', name))])));
const sources = [{ repository, scope: '', name: 'development' }];
if (shared === 'true') sources.push({ repository: 'mylife-inc/releases', scope: 'zellij', name: 'shared' });
const env = { ...process.env };
for (const key of ['GH_TOKEN', 'GITHUB_TOKEN', 'ZEL_INSECURE_SKIP_CHECKSUM']) delete env[key];
const reports = [];
const output = path.resolve('test-results/pdf-public-packages');
await mkdir(output, { recursive: true });
await rm(path.join(output, 'verification.json'), { force: true });
try {
  for (const source of sources) {
    const destination = path.join(root, source.name);
    const result = await downloadRelease({ version, target, ...source, destination, installers });
    const installDir = path.join(root, `installed-${source.name}`);
    const installer = path.join(destination, windows ? 'install.ps1' : 'install.sh');
    execFileSync(windows ? 'powershell.exe' : 'sh', windows
      ? ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', installer] : [installer], {
      env: { ...env, ZEL_VERSION: version, ZEL_RELEASE_BASE_URL: result.base, ZEL_INSTALL_DIR: installDir },
      stdio: 'inherit', timeout: 180_000,
    });
    const binary = path.join(installDir, windows ? 'zel.exe' : 'zel');
    assert.equal(execFileSync(binary, ['--version'], { env, encoding: 'utf8', timeout: 10_000 }).trim(), `zel ${version.slice(1)}`);
    const receipt = path.join(output, `${source.name}.json`);
    execFileSync(process.execPath, ['scripts/check-cached-pdf.mjs', binary], {
      env: { ...env, ZEL_TEST_PDF_BASE_URL: result.base, ZEL_TEST_PDF_RECEIPT: receipt },
      stdio: 'inherit', timeout: 900_000,
    });
    const report = JSON.parse(await readFile(receipt, 'utf8'));
    assert.equal(report.publicDownloadVerified, true);
    if (reports.length) assert.deepEqual(report, reports[0].verification, 'Public PDF/runtime/source repositories disagree');
    reports.push({ repository: source.repository, base: result.base, verification: report });
  }
  await writeFile(path.join(output, 'verification.json'), JSON.stringify({ version, target, reports }, null, 2) + '\n');
  console.log(`${target}: public installed PDF and rich builds passed for ${reports.length} release repositories.`);
} finally {
  await rm(root, { recursive: true, force: true });
}
