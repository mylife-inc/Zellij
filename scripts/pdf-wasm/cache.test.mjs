// Exercises the real versioned archive through Rust acquisition and supervision.
// Only receipts/synthetic PDFs may be uploaded; the compiler package stays local.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { readFile, writeFile, mkdir, mkdtemp, rm, readdir, cp, symlink } from 'node:fs/promises';
import { createReadStream } from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const root = process.cwd();
const ext = process.platform === 'win32' ? '.exe' : '';
const runner = path.join(root, `cli/target/debug/examples/pdf_cache${ext}`);
const generator = path.join(root, `cli/target/debug/examples/pdf_probe_inputs${ext}`);
const distribution = path.resolve(process.env.ZEL_PDF_TOOLCHAIN ?? 'test-results/pdf-toolchain');
const report = JSON.parse(await readFile(path.join(distribution, 'verification.json')));
const evidenceRoot = path.join(root, 'test-results/pdf-cache');
await mkdir(evidenceRoot, { recursive: true });
await rm(path.join(evidenceRoot, 'verification.json'), { force: true });
const version = report.version;
const archive = report.archive.name;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function comparablePdf(bytes) {
  // LuaHBTeX stamps each invocation. Preserve every other byte, including stream
  // contents, object offsets and fonts; require exactly the observed metadata.
  let dates = 0, ids = 0;
  const normalized = bytes.toString('latin1')
    .replace(/\/(CreationDate|ModDate) \(D:\d{14}Z\)/g, (_, field) => {
      dates++; return `/${field} (D:00000000000000Z)`;
    }).replace(/\/ID \[ <[A-F0-9]{32}> <[A-F0-9]{32}> \]/g, () => {
      ids++; return `/ID [ <${'0'.repeat(32)}> <${'0'.repeat(32)}> ]`;
    });
  assert.equal(dates, 2);
  assert.equal(ids, 1);
  return hash(Buffer.from(normalized, 'latin1'));
}
const temp = await mkdtemp(path.join(os.tmpdir(), 'zel-pdf-cache-'));
const input = path.join(temp, 'input café 日本');
const emptyPath = path.join(temp, 'empty-path');
await mkdir(emptyPath);

function execute(executable, args, env = {}, timeout = 45_000) {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, { env: { ...process.env, ...env }, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => { child.kill(); reject(new Error(`Acceptance deadline: ${stderr}`)); }, timeout);
    child.stdout.on('data', b => { stdout += b; });
    child.stderr.on('data', b => { stderr += b; });
    child.once('error', e => { clearTimeout(timer); reject(e); });
    child.once('close', (code, signal) => { clearTimeout(timer); resolve({ code, signal, stdout, stderr }); });
  });
}
async function endpoint(handle) {
  const requests = [];
  const server = createServer((req, res) => { requests.push(req.url); handle(req, res); });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return { url: `http://127.0.0.1:${server.address().port}/`, requests,
    async close() { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); } };
}
function cached(home) { return path.join(home, 'cache', 'pdf-qualification', version); }
async function noStage(home) {
  const entries = await readdir(path.dirname(cached(home))).catch(e => e.code === 'ENOENT' ? [] : Promise.reject(e));
  assert.ok(entries.every(n => !n.startsWith('.download-')), entries.join(','));
}
function invoke(home, url, mode, output, env = {}) {
  return execute(runner, [url, mode, process.execPath, input, output],
    { PATH: emptyPath, ZELLIJ_HOME: home, ...env });
}
const goodHome = path.join(temp, 'Zellij home café 日本');
let assertions = 0;

test('versioned PDF package cache acceptance', { timeout: 180_000 }, async t => {
  t.after(async () => { await rm(temp, { recursive: true, force: true }); });
  const generated = await execute(generator, [path.join(root, 'scripts/fixtures/pdf-book'), input]);
  assert.equal(generated.code, 0, generated.stderr);

  await t.test('cold acquisition and two compilations without tools on PATH; offline makes no requests', async () => {
    const server = await endpoint((req, res) => {
      if (req.url !== `/${archive}` && req.url !== `/${archive}.sha256`) { res.writeHead(404).end(); return; }
      createReadStream(path.join(distribution, req.url.slice(1))).pipe(res);
    });
    try {
      const first = await invoke(goodHome, server.url, 'online', path.join(temp, 'online'));
      assert.equal(first.code, 0, first.stderr);
      const evidence = JSON.parse(first.stdout);
      assert.equal(evidence.manifestSha256, report.manifestSha256);
      assert.deepEqual(evidence.sourceArchive, report.sourceArchive);
      assert.deepEqual(server.requests, [`/${archive}.sha256`, `/${archive}`]);
      const second = await invoke(goodHome, server.url, 'offline', path.join(temp, 'offline'));
      assert.equal(second.code, 0, second.stderr);
      assert.equal(server.requests.length, 2);
      await cp(path.join(temp, 'online'), path.join(evidenceRoot, 'online'), { recursive: true });
      await cp(path.join(temp, 'offline'), path.join(evidenceRoot, 'offline'), { recursive: true });
      assert.equal(comparablePdf(await readFile(path.join(temp, 'online/book.pdf'))),
        comparablePdf(await readFile(path.join(temp, 'offline/book.pdf'))));
      const proof = JSON.parse(await readFile(path.join(temp, 'offline/probe.json')));
      assert.deepEqual(proof.execution, { kind: 'rust-process', node: process.version });
      await noStage(goodHome);
      assertions++;
    } finally { await server.close(); }
  });

  await t.test('offline cache miss creates no cache and never contacts endpoint', async () => {
    const server = await endpoint((_req, res) => res.writeHead(500).end());
    try {
      const home = path.join(temp, 'offline-miss');
      const result = await invoke(home, server.url, 'offline', path.join(temp, 'missing'));
      assert.notEqual(result.code, 0);
      assert.match(result.stderr, /not cached/);
      assert.equal(server.requests.length, 0);
      await assert.rejects(readdir(home), { code: 'ENOENT' });
      assertions++;
    } finally { await server.close(); }
  });

  await t.test('cached corruption, extra files/directories, links and wrong version fail without repair', async () => {
    const mutations = [
      async dir => writeFile(path.join(dir, 'runner/process.mjs'), 'corrupted'),
      async dir => writeFile(path.join(dir, 'extra'), 'keep'),
      async dir => mkdir(path.join(dir, 'extra')),
      async dir => rm(path.join(dir, 'runner/pass.mjs')),
      async dir => { const file = path.join(dir, 'manifest.json'); const m = JSON.parse(await readFile(file)); m.version = '0.0.0'; await writeFile(file, JSON.stringify(m)); },
      async dir => symlink(path.join(temp, 'online'), path.join(dir, 'linked'), process.platform === 'win32' ? 'junction' : 'dir'),
    ];
    for (const [index, mutate] of mutations.entries()) {
      const home = path.join(temp, `corruption-${index}`);
      await cp(goodHome, home, { recursive: true });
      await mutate(cached(home));
      const result = await invoke(home, 'http://127.0.0.1:1/', 'inspect', path.join(temp, 'unused'));
      assert.notEqual(result.code, 0, String(index));
      assert.match(result.stderr, /PDF package/);
      assert.ok((await readdir(cached(home))).length > 0);
      await noStage(home);
      assertions++;
    }
  });

  await t.test('invalid checksum, truncated archive and stalled acquisition preserve outputs and clean stages', async () => {
    for (const mode of ['checksum', 'archive', 'headers', 'checksum-body', 'archive-body']) {
      const home = path.join(temp, mode);
      const output = path.join(temp, `preserve-${mode}`);
      await mkdir(output);
      await writeFile(path.join(output, 'keep'), 'authored');
      const server = await endpoint((req, res) => {
        if (mode === 'headers') return;
        if (req.url.endsWith('.sha256')) {
          if (mode === 'checksum-body') { res.writeHead(200); res.write('a'); return; }
          res.end(mode === 'checksum' ? `invalid ${archive}\n` : `${report.archive.sha256}  ${archive}\n`);
        } else if (mode === 'archive-body') { res.writeHead(200); res.write('a'); }
        else res.end('truncated');
      });
      try {
        const env = ['headers', 'checksum-body', 'archive-body'].includes(mode) ? { ZEL_PDF_CACHE_TEST_CANCEL_MS: '750' } : {};
        const result = await invoke(home, server.url, 'online', output, env);
        assert.notEqual(result.code, 0, mode);
        assert.match(result.stderr, env.ZEL_PDF_CACHE_TEST_CANCEL_MS ? /cancelled/ : /checksum/);
        assert.ok(server.requests.length > 0);
        assert.equal(await readFile(path.join(output, 'keep'), 'utf8'), 'authored');
        await assert.rejects(readdir(cached(home)), { code: 'ENOENT' });
        await noStage(home);
        assertions++;
      } finally { await server.close(); }
    }
  });
  assert.equal(assertions, 13, 'All cache scenarios must pass before writing a success receipt');
  await writeFile(path.join(evidenceRoot, 'verification.json'), JSON.stringify({ schemaVersion: 1,
    experimental: true, redistributionReviewComplete: false, platform: `${process.platform}-${process.arch}`,
    assertions, archive: report.archive, manifestSha256: report.manifestSha256,
    sourceArchive: report.sourceArchive, emptyPath: true, coldAndOfflineCompiled: true,
    pdfComparison: 'all bytes except exactly two PDF timestamps and one document-ID pair' }, null, 2) + '\n');
});
