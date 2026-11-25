// Private contract fixture only: it deliberately simulates a reviewed manifest.
// It grants no license, does not promote qualification archives, and must never
// be uploaded. The installed CLI's default remains the container until release.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { execFileSync, spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { createReadStream } from 'node:fs';
import { cp, mkdir, mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import { installCachedRenderer } from '../fixtures/install-cached-renderer.mjs';

const repo = process.cwd();
const zel = path.join(repo, `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const packageRoot = path.resolve(process.env.ZEL_PDF_TOOLCHAIN ?? 'test-results/pdf-toolchain');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async p => JSON.parse(await readFile(p, 'utf8'));
const version = (await json(path.join(packageRoot, 'verification.json'))).version;
const platform = `${process.platform === 'win32' ? 'win' : process.platform}-${process.arch}`;
const root = await mkdtemp(path.join(os.tmpdir(), 'zel-pdf-installed-'));
const home = path.join(root, 'home café 日本');
const site = path.join(root, 'book café 日本');
const emptyPath = path.join(root, 'empty');
const archiveRoot = path.join(root, 'archives');
const requests = [];
const evidencePath = path.join(repo, 'test-results/pdf-installed/verification.json');
await rm(evidencePath, { force: true });
let base;

function launch(args, extra = {}) {
  const env = { ...process.env, PATH: emptyPath, ZELLIJ_HOME: home,
    ZEL_PDF_BASE_URL: `${base}pdf/`, ZEL_ENGINE_BASE_URL: `${base}renderer/`,
    ZEL_PDF_BACKEND: 'cached', ...extra };
  delete env.ZEL_ENGINE_CACHE;
  const child = spawn(zel, args, { env, stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = '', stderr = '';
  child.stdout.on('data', b => { stdout += b; });
  child.stderr.on('data', b => { stderr += b; });
  const done = new Promise((resolve, reject) => {
    const timer = setTimeout(() => { child.kill(); reject(new Error(`CLI deadline: ${stderr}`)); }, 120_000);
    child.once('error', e => { clearTimeout(timer); reject(e); });
    child.once('close', code => { clearTimeout(timer); resolve({ code, stdout, stderr }); });
  });
  return { child, done, output: () => stdout + stderr };
}
async function good(args, env) {
  const result = await launch(args, env).done;
  assert.equal(result.code, 0, result.stderr);
  return result;
}
async function until(check, detail) {
  const end = Date.now() + 90_000;
  while (Date.now() < end) {
    if (await check()) return;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error(`Missing ${detail}`);
}
function archive(input, output) {
  execFileSync(process.platform === 'win32' ? 'python' : 'python3', ['-c',
    "import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]); import distribution; distribution.archive(Path(sys.argv[2]),Path(sys.argv[3]))",
    path.join(repo, 'scripts/pdf-wasm'), input, output], { timeout: 120_000 });
}
async function checksum(file) { await writeFile(file + '.sha256', `${hash(await readFile(file))}  ${path.basename(file)}\n`); }

async function fixture() {
  await mkdir(emptyPath); await mkdir(archiveRoot);
  await cp(path.join(repo, 'scripts/fixtures/pdf-book'), site, { recursive: true });
  const compiler = path.join(root, 'compiler');
  await cp(path.join(packageRoot, 'package'), compiler, { recursive: true });
  const manifest = await json(path.join(compiler, 'manifest.json'));
  Object.assign(manifest, { format: 2, kind: 'zellij-pdf-toolchain', redistribution_review_complete: true });
  manifest.source_archive.name = `zel-pdf-sources-v${version}.tar.gz`;
  for (const name of ['ZELLIJ-PDF-ADAPTER-LICENSE.txt', 'DISTRIBUTION-REVIEW.json']) {
    const relative = 'notices/' + name;
    const data = Buffer.from('TEST FIXTURE ONLY: simulated review; no license or publication permission.\n');
    await writeFile(path.join(compiler, relative), data);
    manifest.files[relative] = { bytes: data.length, sha256: hash(data) };
  }
  await writeFile(path.join(compiler, 'manifest.json'), JSON.stringify(manifest));
  const pdfArchive = path.join(archiveRoot, `zel-pdf-v${version}.tar.gz`);
  archive(compiler, pdfArchive); await checksum(pdfArchive);
  const cache = await installCachedRenderer(zel, path.join(root, 'renderer'), path.join(repo, 'scripts/fixtures/build-cached-engine.mjs'));
  const rendererArchive = path.join(archiveRoot, `zel-renderer-v${version}-${platform}.tar.gz`);
  archive(path.join(cache, `${version}-${platform}`), rendererArchive); await checksum(rendererArchive);
  return manifest;
}

test('installed CLI uses cached PDF execution and its verified private Node', { timeout: 600_000 }, async t => {
  t.after(async () => { await rm(root, { recursive: true, force: true }); });
  const manifest = await fixture();
  const server = createServer((req, res) => {
    requests.push(req.url);
    const name = req.url.split('/').pop();
    if (!/^(zel-pdf-v|zel-renderer-v)[A-Za-z0-9.-]+\.tar\.gz(?:\.sha256)?$/.test(name)) { res.writeHead(404).end(); return; }
    createReadStream(path.join(archiveRoot, name)).on('error', () => res.destroy()).pipe(res);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  base = `http://127.0.0.1:${server.address().port}/`;
  t.after(async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); });

  await t.test('native check and offline cache miss never download', async () => {
    await good(['pdf', site, '--check', '--offline']);
    const config = path.join(site, 'zel.yaml');
    const original = await readFile(config, 'utf8');
    // Diagnose an enabled PDF site without building or creating its cache.
    await writeFile(config, original + '\npdf: {enabled: true}\n');
    const diagnosis = JSON.parse((await good(['doctor', site, '--json'])).stdout);
    assert.equal(diagnosis.runtime.status, 'not_required');
    assert.equal(diagnosis.pdf_engine.status, 'download_required');
    assert.equal(diagnosis.rich_engine.status, 'download_required');
    const result = await launch(['pdf', site, '--offline']).done;
    assert.notEqual(result.code, 0); assert.match(result.stderr, /not cached/);
    assert.deepEqual(requests, []);
    await assert.rejects(readdir(home), { code: 'ENOENT' });
  });
  await t.test('cold PDF downloads both packages with an empty PATH and records the real backend', async () => {
    await good(['pdf', site]);
    assert.equal(requests.length, 4);
    assert(requests.some(p => p.includes('/renderer/')));
    const receipt = await json(path.join(site, 'pdf/pdf.json'));
    assert.equal(receipt.backend, 'luahbtex-wasm');
    assert.equal(receipt.toolchain_manifest_sha256, hash(Buffer.from(JSON.stringify(manifest))));
    assert.equal(receipt.pdf_sha256, hash(await readFile(path.join(site, 'pdf/book.pdf'))));
    assert.equal(receipt.node, process.version);
    assert.equal(receipt.engine_release, '2026-f5d94438a7c75a23');
    for (const key of ['resource_lock_sha256', 'format_sha256', 'font_metadata_sha256']) {
      assert.match(receipt[key], /^[a-f0-9]{64}$/);
    }
    assert.deepEqual(receipt.source_archive, manifest.source_archive);
    assert.equal(receipt.toolchain_image, undefined);
    const diagnosis = JSON.parse((await good(['doctor', site, '--json'])).stdout);
    assert.equal(diagnosis.pdf_engine.status, 'cached');
    assert.equal(diagnosis.rich_engine.status, 'cached');
  });
  await t.test('offline alias and PDF-enabled build reuse both caches and preserve output on corruption', async () => {
    await good(['export', 'pdf', site, '--offline']);
    await good(['build', site, '--pdf', '--offline']);
    assert.equal(requests.length, 4);
    const publication = await json(path.join(site, 'site/_zellij/book.json'));
    const receipt = await json(path.join(site, 'site/_zellij/pdf/pdf.json'));
    assert.equal(receipt.revision, publication.revision);
    const before = await readFile(path.join(site, 'site/_zellij/pdf/book.pdf'));
    const node = path.join(home, 'cache/engines', `${version}-${platform}`, process.platform === 'win32' ? 'node.exe' : 'node');
    await rename(node, node + '.saved');
    try {
      const result = await launch(['build', site, '--pdf', '--offline']).done;
      assert.notEqual(result.code, 0); assert.match(result.stderr, /Rich renderer/);
      const diagnosis = await launch(['doctor', site, '--json']).done;
      assert.notEqual(diagnosis.code, 0);
      assert.match(JSON.parse(diagnosis.stdout).error.message, /Rich renderer/);
      assert.deepEqual(await readFile(path.join(site, 'site/_zellij/pdf/book.pdf')), before);
      assert.equal(requests.length, 4);
    } finally { await rename(node + '.saved', node); }
  });
  await t.test('watched preview rebuilds cached PDFs offline and serves the matching download', async () => {
    const run = launch(['view', site, '--pdf', '--offline', '--watch', '--no-open', '--port', '0']);
    try {
      await until(() => /http:\/\/127\.0\.0\.1:\d+/.test(run.output()), 'preview URL');
      const url = run.output().match(/http:\/\/127\.0\.0\.1:\d+/)[0];
      const download = await fetch(url + '/_zellij/pdf/book.pdf');
      assert.equal(download.status, 200);
      assert(Buffer.from(await download.arrayBuffer()).subarray(0, 5).equals(Buffer.from('%PDF-')));
      const receiptPath = path.join(site, 'site/_zellij/pdf/pdf.json');
      const before = await json(receiptPath);
      const file = path.join(site, 'content/principles.md');
      await writeFile(file, await readFile(file, 'utf8') + '\nA saved revision for the offline preview.\n');
      await until(async () => { try { return (await json(receiptPath)).revision !== before.revision; } catch { return false; } }, 'new PDF revision');
      const after = await json(receiptPath);
      assert.equal(after.pdf_sha256, hash(await readFile(path.join(site, 'site/_zellij/pdf/book.pdf'))));
      const updated = await fetch(url + '/_zellij/pdf/book.pdf');
      assert.equal(updated.status, 200);
      assert.equal(hash(Buffer.from(await updated.arrayBuffer())), after.pdf_sha256);
      assert.equal(requests.length, 4);
    } finally { run.child.kill(); await run.done; }
  });
  const evidence = { schemaVersion: 1, fixtureOnly: true, releaseApproved: false,
    platform: `${process.platform}-${process.arch}`, cases: 4, emptyPath: true,
    privateNodeReused: true, coldRequests: requests.length, offlineWatchRebuilt: true };
  await mkdir(path.join(repo, 'test-results/pdf-installed'), { recursive: true });
  await writeFile(evidencePath, JSON.stringify(evidence, null, 2) + '\n');
});
