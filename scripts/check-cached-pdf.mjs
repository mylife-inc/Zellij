// Real compiler + real renderer acceptance; child CLI has no system tools.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { createServer } from 'node:http';
import { createReadStream } from 'node:fs';
import { mkdtemp, mkdir, readFile, writeFile, cp, rm } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import { verifyPdfRelease } from './verify-pdf-release.mjs';

const repo = path.resolve(import.meta.dirname, '..');
const binary = path.resolve(process.argv[2] ?? `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const rendererDir = path.resolve(process.argv[3] ?? 'dist/renderer-packages');
const pdfDir = path.resolve(process.argv[4] ?? 'test-results/pdf-release/artifacts');
const version = (await readFile(path.join(repo, 'cli/Cargo.toml'), 'utf8')).match(/^version = "([^"]+)"/m)[1];
const target = JSON.parse(execFileSync(binary, ['version', '--json'], { encoding: 'utf8', timeout: 10_000 })).cli.target;
const platform = `${target.includes('windows') ? 'win' : target.includes('darwin') ? 'darwin' : 'linux'}-${target.startsWith('aarch64') ? 'arm64' : 'x64'}`;
const publicBase = process.env.ZEL_TEST_PDF_BASE_URL;
if (publicBase) {
  const url = new URL(publicBase);
  assert.equal(url.protocol, 'https:');
  assert.equal(url.host, 'github.com');
  assert(!url.username && !url.password && !url.search && !url.hash);
  assert.match(url.pathname, /^\/[\w.-]+\/[\w.-]+\/releases\/download\/(?:zellij-)?v[\w.-]+\/?$/);
}
const root = await mkdtemp(path.join(os.tmpdir(), 'zel-real-pdf café '));
const home = path.join(root, 'home 日本'), book = path.join(root, 'book'), rich = path.join(root, 'product');
const output = path.resolve(process.env.ZEL_TEST_PDF_RECEIPT ?? 'test-results/pdf-real-packages/verification.json');
const sha = b => createHash('sha256').update(b).digest('hex');
const json = async p => JSON.parse(await readFile(p, 'utf8'));
const requests = [];
const assets = new Map();
let server;
await rm(output, { force: true });
try {
  let base = publicBase;
  if (!base) {
    let sums = '';
    for (const [dir, names] of [[rendererDir, [`zel-renderer-v${version}-${platform}.tar.gz`]],
      [pdfDir, [`zel-pdf-v${version}.tar.gz`, `zel-pdf-sources-v${version}.tar.gz`]]]) {
      for (const name of names) {
        assets.set(name, path.join(dir, name)); assets.set(name + '.sha256', path.join(dir, name + '.sha256'));
        sums += await readFile(path.join(dir, name + '.sha256'), 'utf8');
      }
    }
    server = createServer((req, res) => {
      requests.push(req.url);
      const name = req.url.slice(1);
      if (name === 'SHA256SUMS') { res.end(sums); return; }
      const file = assets.get(name);
      if (!file) { res.writeHead(404).end(); return; }
      createReadStream(file).on('error', () => res.destroy()).pipe(res);
    });
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    base = `http://127.0.0.1:${server.address().port}`;
  }
  const env = { ...process.env, PATH: path.join(root, 'empty'), ZELLIJ_HOME: home,
    ZEL_PDF_BACKEND: 'cached', ZEL_PDF_BASE_URL: base, ZEL_ENGINE_BASE_URL: base };
  for (const name of ['GH_TOKEN', 'GITHUB_TOKEN', 'ZEL_ENGINE_CACHE', 'NODE_OPTIONS', 'NODE_PATH', 'ZEL_INSECURE_SKIP_CHECKSUM']) delete env[name];
  await mkdir(env.PATH);
  async function run(args, offline = false) {
    return new Promise((resolve, reject) => {
      // Unreachable overrides ensure offline commands cannot silently fetch a package.
      const child = spawn(binary, args, { env: offline ? { ...env,
        ZEL_PDF_BASE_URL: 'http://127.0.0.1:1', ZEL_ENGINE_BASE_URL: 'http://127.0.0.1:1' } : env,
        windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
      let text = '', timedOut = false;
      const timer = setTimeout(() => { timedOut = true; child.kill('SIGKILL'); }, 180_000);
      const capture = b => { if (text.length < 2 * 1024 * 1024) text += b; };
      child.stdout.on('data', capture); child.stderr.on('data', capture);
      child.once('error', error => { clearTimeout(timer); reject(error); });
      child.once('close', code => { clearTimeout(timer); code === 0 && !timedOut ? resolve(text)
        : reject(new Error(`${args.join(' ')} ${timedOut ? 'exceeded deadline' : `exited ${code}`}\n${text}`)); });
    });
  }
  await cp(path.join(repo, 'scripts/fixtures/pdf-book'), book, { recursive: true });
  await run(['pdf', book, '--check', '--offline'], true);
  assert.equal(requests.length, 0);
  await run(['pdf', book]);
  if (!publicBase) assert.equal(requests.length, 4, 'Cold run downloads one PDF and one renderer pair');
  const pdfCache = path.join(home, 'cache/pdf', version), rendererCache = path.join(home, 'cache/engines', `${version}-${platform}`);
  const manifestBytes = await readFile(path.join(pdfCache, 'manifest.json'));
  const manifest = JSON.parse(manifestBytes), renderer = await json(path.join(rendererCache, 'manifest.json'));
  const receipt = await json(path.join(book, 'pdf/pdf.json'));
  assert.equal(receipt.backend, 'luahbtex-wasm');
  assert.equal(receipt.toolchain_manifest_sha256, sha(manifestBytes));
  assert.equal(receipt.node, renderer.node_version);
  assert.deepEqual(receipt.source_archive, manifest.source_archive);
  assert.equal(receipt.pdf_sha256, sha(await readFile(path.join(book, 'pdf/book.pdf'))));
  assert.deepEqual(await readFile(path.join(pdfCache, 'notices/DISTRIBUTION-REVIEW.json')),
    await readFile(path.join(repo, 'scripts/pdf-wasm/distribution-review.json')));
  for (const name of ['bundle.mjs', 'font-inputs.json', 'format-inputs.json', 'lock.json', 'pass.mjs', 'probe.mjs', 'process.mjs']) {
    assert.deepEqual(await readFile(path.join(pdfCache, 'runner', name)), await readFile(path.join(repo, 'scripts/pdf-wasm', name)));
  }
  const pair = await verifyPdfRelease({ base, manifest });
  const beforeOffline = requests.length;
  await run(['build', book, '--pdf', '--offline'], true);
  const printed = path.join(book, 'site/_zellij/pdf');
  const built = await json(path.join(printed, 'pdf.json'));
  assert.equal(built.revision, (await json(path.join(book, 'site/_zellij/book.json'))).revision);
  assert.equal(built.pdf_sha256, sha(await readFile(path.join(printed, 'book.pdf'))));
  assert.match(await readFile(path.join(book, 'site/index.html'), 'utf8'), /_zellij\/pdf\/book\.pdf/);
  await run(['create', rich, '--template', 'product-studio', '--offline'], true);
  await run(['build', rich, '--offline'], true);
  assert.match(await readFile(path.join(rich, 'site/index.html'), 'utf8'), /_zellij\/file\/styles\.css/);
  assert((await json(path.join(rich, 'site/_zellij/file/manifest.json'))).pages > 0);
  assert.deepEqual(await json(path.join(rendererCache, 'manifest.json')), renderer, 'PDF and rich web reuse the same renderer');
  assert.equal(requests.length, beforeOffline);
  const evidence = { schemaVersion: 1, version, platform, publicDownloadVerified: Boolean(publicBase),
    realRenderer: true, emptyPath: true, offlinePdfAndRichBuilds: true, privateNodeReused: true,
    manifestSha256: sha(manifestBytes), ...pair };
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, JSON.stringify(evidence, null, 2) + '\n');
  console.log(`Real PDF/renderer pair passed on ${platform}; full source verified; offline PDF and rich builds passed.`);
} finally {
  if (server) { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
  await rm(root, { recursive: true, force: true });
}
