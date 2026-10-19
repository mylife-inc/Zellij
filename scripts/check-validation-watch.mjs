// Actual native validation loops, recovery and Ctrl-C on every supported OS.
// --hosted uses the CI-built engine image; normal mode has no tools on PATH.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile, stat, utimes } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const windows = process.platform === 'win32';
const explicitConfig = process.argv.includes('--config');
const configName = explicitConfig ? 'alternate.yaml' : 'zel.yaml';
const configRelative = `docs/zellij/${configName}`;
const hosted = process.argv.includes('--hosted');
const zel = path.resolve(`cli/target/debug/zel${windows ? '.exe' : ''}`);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-validation-watch-'));
const site = path.join(root, 'docs/zellij');
const consoleRunner = path.join(root, 'console.exe');
let child, closed, exit;
let stdout = '', stderr = '';
const count = text => stdout.split(text).length - 1;
const put = async (file, body) => { const target = path.join(root, file); await mkdir(path.dirname(target), { recursive: true }); await writeFile(target, body); };
async function until(predicate, label) {
  for (let i = 0; i < 300; i++) {
    if (predicate()) return;
    assert(!exit, `Watch exited: ${JSON.stringify(exit)}\n${stdout}\n${stderr}`);
    await delay(100);
  }
  throw new Error(`${label}\n${stdout}\n${stderr}`);
}
try {
  if (windows) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleRunner], { stdio: 'inherit' });
  const config = `version: 1\nrenderer: ${hosted ? 'hosted' : 'portable'}\n`;
  await put(configRelative, config);
  const page = hosted ? 'docs/zellij/content/manual/index.mdx' : 'docs/zellij/content/index.md';
  const valid = '# Original\n\nValid content!\n';
  const broken = '# Original\n\n[Bad](lost.md)\n';
  await put(page, broken);
  if (hosted) {
    await put('docs/zellij/content/site.yaml', 'name: Watch\nhome: manual\n');
    await put('docs/zellij/content/manual/_sidebar.yaml', 'nav: [index]\n');
  }
  const env = hosted ? process.env : { ...process.env, PATH: path.join(root, 'no-tools'), DOCKER_HOST: 'fixture-invalid' };
  if (explicitConfig) await put('docs/zellij/zel.yaml', 'invalid default: [');
  const args = ['watch', '--deep', '--strict', ...(explicitConfig ? ['--config', configRelative] : [])];
  child = spawn(windows ? consoleRunner : zel, windows ? [zel, ...args] : args, { cwd: root, env, stdio: ['pipe', 'pipe', 'pipe'] });
  child.stdout.on('data', data => stdout += data);
  child.stderr.on('data', data => stderr += data);
  closed = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => { exit = { code, signal }; resolve(exit); }); });
  await until(() => stdout.includes('Watching ') && count('Validation failed;') === 1, 'invalid initial content remains watched');
  await put(page, valid);
  await until(() => count('Validation passed.') === 1, 'initial validation recovery');
  await put('docs/specs/ignored.yaml', 'broken: [');
  await put('docs/zellij/site/ignored.yaml', 'broken: [');
  await delay(1100);
  assert.equal(count('Checking '), 2, 'specs and generated output must not trigger validation');
  const metadata = await stat(path.join(root, page));
  await put(page, broken);
  assert.equal((await stat(path.join(root, page))).size, metadata.size, 'same-size edit fixture');
  await utimes(path.join(root, page), metadata.atime, metadata.mtime);
  await until(() => count('Validation failed;') === 2, 'broken reference reported');
  await put(page, valid);
  await until(() => count('Validation passed.') === 2, 'recovery without restart');
  await put(configRelative, 'version: [');
  await until(() => stderr.includes(configName), 'broken configuration reported');
  await put(configRelative, config);
  await until(() => count('Validation passed.') === 3, 'configuration recovery');
  const priorErrors = stderr.length;
  await rm(path.join(site, configName));
  await until(() => stderr.length > priorErrors && stderr.slice(priorErrors).includes(explicitConfig ? configName : 'requires its version: 1'), 'missing config cannot widen the watched root');
  await put(configRelative, config);
  await until(() => count('Validation passed.') === 4, 'restored configuration');
  await put(configRelative, `version: 1\nrenderer: ${hosted ? 'portable' : 'hosted'}\n`);
  await until(() => stderr.includes(`operation requires ${hosted ? 'Hosted' : 'Portable'}`), 'renderer stays pinned');
  await put(configRelative, config);
  await until(() => count('Validation passed.') === 5, 'renderer recovery');
  if (windows) child.stdin.end('stop\n'); else child.kill('SIGINT');
  await until(() => exit !== undefined, 'Ctrl-C shutdown');
  assert.equal(exit.code, 0, stderr);
  assert(stdout.includes('Stopped.'));
  assert(!stdout.includes('Serving at'));
  assert.equal(await readFile(path.join(root, page), 'utf8'), valid);
  assert.equal(await readFile(path.join(site, 'site/ignored.yaml'), 'utf8'), 'broken: [');
  console.log(`Validation watch (${hosted ? 'hosted' : 'portable'}, ${configName}): initial check, scope, edits, recovery, pinned layout/renderer and Ctrl-C passed.`);
} finally {
  if (child && !exit) { if (windows) child.stdin.end('stop\n'); else child.kill('SIGTERM'); await closed; }
  await rm(root, { recursive: true, force: true, maxRetries: 5 });
}
