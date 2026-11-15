// Hosted acceptance: install verified public bytes, then use the tagged PDF driver.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm, access } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { downloadRelease } from './download-release.mjs';
import { imageTags } from './release-policy.mjs';

export async function verifyPublishedNextPdf({ version, repository, imageRepository,
  captures = path.resolve('test-results/next-pdf') },
{ download = downloadRelease, run = execFileSync, environment = process.env } = {}) {
  const image = imageTags(version, imageRepository)[0];
  assert(['x64', 'arm64'].includes(process.arch), 'Next PDF acceptance requires native x64 or arm64');
  const target = `${process.arch === 'arm64' ? 'aarch64' : 'x86_64'}-unknown-linux-gnu`;
  const architecture = process.arch === 'arm64' ? 'arm64' : 'amd64';
  await access('scripts/check-next-pdf.mjs'); // Older reading-only releases cannot pass this gate.
  const env = { ...environment, ZELLIJ_IMAGE: image };
  for (const name of ['GITHUB_TOKEN', 'GH_TOKEN', 'ZEL_INSECURE_SKIP_CHECKSUM']) delete env[name];
  const exec = (command, args, options = {}) => run(command, args, { env, stdio: 'inherit', timeout: 180000, ...options });
  exec(process.execPath, ['scripts/package-native.mjs', version, '--check-version']);
  const installers = Object.fromEntries(await Promise.all(['install.sh', 'install.ps1'].map(async name => [name, await readFile(path.join('scripts', name))])));
  const root = await mkdtemp(path.join(tmpdir(), 'zel-public-next-pdf-'));
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
    exec(process.execPath, ['scripts/check-next-pdf.mjs', binary], { timeout: 1200000 });
    console.log(`${version}: public CLI and ${image} passed Next PDF acceptance on ${architecture}`);
    return receipt;
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.platform, 'linux', 'Run public Next PDF acceptance on a native Linux runner');
  await verifyPublishedNextPdf({ version: process.env.VERSION, repository: process.env.RELEASE_REPOSITORY,
    imageRepository: process.env.RELEASE_IMAGE });
}
