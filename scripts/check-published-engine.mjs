// Shared hosted acceptance: install verified public bytes and run a tagged engine driver.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { downloadRelease } from './download-release.mjs';
import { imageTags } from './release-policy.mjs';

export async function verifyPublishedEngine({ version, repository, imageRepository,
  captures, driver, label },
{ download = downloadRelease, run = execFileSync, environment = process.env } = {}) {
  const image = imageTags(version, imageRepository)[0];
  assert(['x64', 'arm64'].includes(process.arch), `${label} acceptance requires native x64 or arm64`);
  const target = `${process.arch === 'arm64' ? 'aarch64' : 'x86_64'}-unknown-linux-gnu`;
  const architecture = process.arch === 'arm64' ? 'arm64' : 'amd64';
  await access(driver); // The checked-out release must supply the actual driver.
  const env = { ...environment, ZELLIJ_IMAGE: image };
  for (const name of ['GITHUB_TOKEN', 'GH_TOKEN', 'ZEL_INSECURE_SKIP_CHECKSUM']) delete env[name];
  const exec = (command, args, options = {}) => run(command, args, { env, stdio: 'inherit', timeout: 180000, ...options });
  exec(process.execPath, ['scripts/package-native.mjs', version, '--check-version']);
  const installers = Object.fromEntries(await Promise.all(['install.sh', 'install.ps1'].map(async name => [name, await readFile(path.join('scripts', name))])));
  const root = await mkdtemp(path.join(tmpdir(), 'zel-public-engine-'));
  try {
    const destination = path.join(root, 'download');
    const result = await download({ version, target, repository, destination, installers });
    const installed = path.join(root, 'installed');
    const binary = path.join(installed, 'zel');
    exec('sh', [path.join(destination, 'install.sh')], { env: { ...env,
      ZEL_VERSION: version, ZEL_RELEASE_BASE_URL: result.base, ZEL_INSTALL_DIR: installed } });
    assert.equal(exec(binary, ['--version'], { stdio: 'pipe', encoding: 'utf8' }).trim(), `zel ${version.slice(1)}`);
    exec('docker', ['pull', image]);
    const [engine] = JSON.parse(exec('docker', ['image', 'inspect', image], { stdio: 'pipe', encoding: 'utf8' }));
    assert.equal(engine.Architecture, architecture, 'Published engine must match the native runner');
    assert(engine.RepoDigests?.length, 'Public pull must resolve an immutable image digest');
    const receipt = { version, target, repository, archive: result.archive, archive_sha256: result.hash,
      engine: image, engine_digests: engine.RepoDigests };
    await mkdir(captures, { recursive: true });
    await writeFile(path.join(captures, 'published.json'), JSON.stringify(receipt, null, 2) + '\n');
    exec(process.execPath, [driver, binary], { timeout: 1200000 });
    console.log(`${version}: public CLI and ${image} passed ${label} acceptance on ${architecture}`);
    return receipt;
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

