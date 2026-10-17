// Exercise the installed CLI without workspace dependencies, including process
// ownership. The real Next/browser/container acceptance lives in check-dev-site.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

const repo = fileURLToPath(new URL('../', import.meta.url));
const root = await mkdtemp(path.join(os.tmpdir(), 'zellij-dev-package-'));
const app = path.join(root, 'app');
const content = path.join(root, 'external café');
const installed = path.join(app, 'node_modules/@shebka/zellij');
const cli = path.join(installed, 'bin/zellij.mjs');
const children = new Set();
const put = async (name, text) => { await mkdir(path.dirname(name), { recursive: true }); await writeFile(name, text); };
async function until(check, label) {
  let error;
  for (let attempt = 0; attempt < 150; attempt++) {
    try { if (await check()) return; } catch (caught) { error = caught; }
    await delay(100);
  }
  throw new Error(`Timed out: ${label}`, { cause: error });
}
const json = async (name) => JSON.parse(await readFile(name, 'utf8'));
function start(args) {
  const child = spawn(process.execPath, [cli, ...args], { cwd: app, stdio: ['ignore', 'pipe', 'pipe'] });
  let log = '';
  child.stdout.on('data', (data) => { log += data; });
  child.stderr.on('data', (data) => { log += data; });
  const done = new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', (code, signal) => resolve({ code, signal })); });
  children.add(child);
  done.finally(() => children.delete(child));
  return { child, done, log: () => log };
}
async function stop(run, signal, expected) {
  run.child.kill(signal);
  let result;
  await until(async () => {
    if (run.child.exitCode === null && run.child.signalCode === null) return false;
    result = await run.done;
    return true;
  }, `shutdown: ${run.log()}`);
  assert.equal(result.code, expected, run.log());
}
try {
  await put(path.join(installed, 'package.json'), '{"type":"module"}');
  for (const name of ['bin/zellij.mjs', 'dist/cli/dev-bundle.js']) {
    await mkdir(path.dirname(path.join(installed, name)), { recursive: true });
    await copyFile(path.join(repo, 'engine/packages/zellij', name), path.join(installed, name));
  }
  await put(path.join(content, 'site.yaml'), 'name: Watch fixture\nhome: 01-guide\n');
  await put(path.join(content, '01-guide/_sidebar.yaml'), 'route: /manual\nnav: [index]\n');
  await put(path.join(content, '01-guide/index.mdx'), '# Welcome\n\n## Original\n\nBefore edits.\n');
  await put(path.join(content, 'assets/café.svg'), '<svg>original</svg>');
  const index = path.join(app, 'public/_zellij/search-index.json');
  const run = start(['prepare', '--watch', '--content', content]);
  await until(async () => (await json(index)).some((entry) => entry.route === '/manual#original'), 'initial preparation');
  await put(path.join(content, '01-guide/index.mdx'), '# Welcome\n\n## Changed\n\nNew passage.\n');
  await put(path.join(content, 'assets/café.svg'), '<svg>changed</svg>');
  await until(async () => (await json(index)).some((entry) => entry.route === '/manual#changed'), 'watched search update');
  await until(async () => (await readFile(path.join(app, 'public/_zellij/assets/café.svg'), 'utf8')).includes('changed'), 'watched asset update');
  await stop(run, 'SIGINT', 130);

  // A tiny stand-in for Next verifies argument boundaries, exit propagation and
  // signal forwarding. Real Next is tested separately, not inferred from this.
  const marker = path.join(app, 'child.json');
  await put(path.join(app, 'node_modules/next/package.json'), '{"name":"next"}');
  await put(path.join(app, 'node_modules/next/dist/bin/next'), `
    const fs = require('node:fs');
    fs.writeFileSync(${JSON.stringify(marker)}, JSON.stringify({ pid: process.pid, args: process.argv.slice(2) }));
    if (process.argv.includes('--fail')) process.exit(17);
    process.on('SIGTERM', () => process.exit(0));
    process.on('SIGINT', () => process.exit(0));
    setInterval(() => {}, 1000);
  `);
  const dev = start(['dev', '--content', content, '--', '--hostname', '127.0.0.1', '--port', '4567']);
  await until(async () => Boolean((await json(marker)).pid), 'Next child start');
  const info = await json(marker);
  assert.deepEqual(info.args, ['dev', '--hostname', '127.0.0.1', '--port', '4567']);
  await stop(dev, 'SIGTERM', 143);
  assert.throws(() => process.kill(info.pid, 0), { code: 'ESRCH' }, 'Next child must not be orphaned');
  const failure = start(['dev', '--content', content, '--', '--fail']);
  await until(() => failure.child.exitCode !== null, 'failed Next exits without a lingering watcher');
  assert.equal((await failure.done).code, 17, failure.log());
  await put(path.join(content, 'site.yaml'), 'name: [');
  await rm(marker);
  const invalid = start(['dev', '--content', content]);
  await until(() => invalid.child.exitCode !== null, 'initial invalid content fails');
  assert.equal((await invalid.done).code, 1);
  await assert.rejects(readFile(marker), { code: 'ENOENT' }, 'Next must not start before preparation succeeds');
  for (const args of [['prepare', '--watch=false'], ['prepare', '--watch', '--content'], ['dev', '--port', '1234']]) {
    assert.notEqual(spawnSync(process.execPath, [cli, ...args], { cwd: app, timeout: 5000 }).status, 0);
  }
  console.log('Installed dev preparation: watching, custom paths, argument forwarding, invalid startup, exit codes and child shutdown passed.');
} finally {
  for (const child of children) child.kill('SIGTERM');
  await rm(root, { recursive: true, force: true });
}
