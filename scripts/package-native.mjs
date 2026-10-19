// One archive contract for CI, tagged releases, and local packaging. No npm dependencies.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, rename, rm, writeFile, access, lstat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const [requestedVersion, target, destination = 'dist'] = process.argv.slice(2);
const supported = [
  'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu',
  'x86_64-apple-darwin', 'aarch64-apple-darwin',
  'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc',
];
const manifest = await readFile(path.join(root, 'cli/Cargo.toml'), 'utf8');
const version = manifest.match(/\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/)?.[1].match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!version) throw new Error('Cannot read the Rust workspace version');
const tag = requestedVersion === 'workspace' ? `v${version}` : requestedVersion;
if (tag !== `v${version}` || !/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag)) {
  throw new Error(`Requested version ${tag} must match the committed workspace version v${version}`);
}
if (target === '--check-version') { console.log(`Release version matches ${tag}`); process.exit(0); }
if (!supported.includes(target)) throw new Error(`Unsupported release target: ${target}`);
const windows = target.endsWith('-windows-msvc');
const binary = windows ? 'zel.exe' : 'zel';
const name = `zel-${tag}-${target}.${windows ? 'zip' : 'tar.gz'}`;
const out = path.resolve(destination);
for (const file of [name, `${name}.sha256`]) {
  if (await lstat(path.join(out, file)).then(() => true, error => { if (error.code === 'ENOENT') return false; throw error; })) throw new Error(`Refusing to replace ${file}`);
}
const env = { ...process.env };
// Keep the Windows CRT in the executable; do not require a separate VC runtime installer.
if (windows) env[`CARGO_TARGET_${target.toUpperCase().replaceAll('-', '_')}_RUSTFLAGS`] = '-C target-feature=+crt-static';
execFileSync('cargo', ['build', '--locked', '--release', '--bin', 'zel', '--target', target], { cwd: path.join(root, 'cli'), env, stdio: 'inherit' });
const targetDir = process.env.CARGO_TARGET_DIR ? path.resolve(root, 'cli', process.env.CARGO_TARGET_DIR) : path.join(root, 'cli/target');
const built = path.join(targetDir, target, 'release', binary);
await access(built);
await mkdir(out, { recursive: true });
const stage = await mkdtemp(path.join(out, '.package-'));
try {
  const archive = path.join(stage, name);
  if (windows) {
    execFileSync('pwsh', ['-NoProfile', '-File', path.join(root, 'scripts/package-windows.ps1'), '-Binary', built, '-Archive', archive], { stdio: 'inherit' });
  } else {
    // macOS otherwise adds a ._zel AppleDouble entry for provenance attributes.
    // Native archives contain one executable on every platform.
    execFileSync('tar', ['-czf', archive, '-C', path.dirname(built), binary], { stdio: 'inherit', env: { ...process.env, COPYFILE_DISABLE: '1' } });
  }
  const digest = createHash('sha256').update(await readFile(archive)).digest('hex');
  await writeFile(path.join(stage, `${name}.sha256`), `${digest}  ${name}\n`);
  await rename(archive, path.join(out, name));
  await rename(path.join(stage, `${name}.sha256`), path.join(out, `${name}.sha256`));
  console.log(`Packaged ${path.join(out, name)}`);
} finally { await rm(stage, { recursive: true, force: true }); }
