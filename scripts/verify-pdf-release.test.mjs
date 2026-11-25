import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { verifyPdfRelease } from './verify-pdf-release.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture() {
  const source = Buffer.from('synthetic complete source archive');
  const manifest = { format: 2, kind: 'zellij-pdf-toolchain', version: '1.2.3-rc.1', redistribution_review_complete: true,
    source_archive: { name: 'zel-pdf-sources-v1.2.3-rc.1.tar.gz', bytes: source.length, sha256: sha(source) },
    source_manifest_sha256: 'b'.repeat(64) };
  const runtime = 'zel-pdf-v1.2.3-rc.1.tar.gz', name = manifest.source_archive.name;
  const files = { [name]: source, [name + '.sha256']: `${sha(source)}  ${name}\n`,
    [runtime + '.sha256']: `${'a'.repeat(64)}  ${runtime}\n`,
    SHA256SUMS: `${'a'.repeat(64)}  ./${runtime}\n${sha(source)}  ./${name}\n` };
  const calls = [];
  const fetchImpl = async (url, init) => {
    calls.push(url);
    assert(!init.headers && init.signal, 'Never send credentials; always bound transfers');
    const value = files[new URL(url).pathname.split('/').at(-1)];
    return new Response(value ?? 'missing', { status: value === undefined ? 404 : 200 });
  };
  return { manifest, source, runtime, files, calls, options: { base: 'https://example.test/release', manifest, fetchImpl } };
}
test('checks the runtime/source pair and streams the complete matching source', async () => {
  const f = fixture(), result = await verifyPdfRelease(f.options);
  assert.deepEqual(result.source, f.manifest.source_archive);
  assert.equal(result.sourceBytesVerified, true);
  assert.equal(f.calls.length, 4);
});
test('rejects incomplete review, unsafe source names and unbounded source sizes before requests', async () => {
  for (const change of [m => m.redistribution_review_complete = false, m => m.format = 1,
    m => m.source_archive.name = '../elsewhere', m => m.source_archive.bytes = 2 ** 32]) {
    const f = fixture(); change(f.manifest);
    await assert.rejects(verifyPdfRelease(f.options)); assert.equal(f.calls.length, 0);
  }
});
test('rejects missing, duplicated and inconsistent public checksum records', async () => {
  for (const change of [f => f.files.SHA256SUMS += f.files.SHA256SUMS,
    f => delete f.files[f.manifest.source_archive.name + '.sha256'],
    f => f.files[f.runtime + '.sha256'] = 'wrong',
    f => f.manifest.source_archive.sha256 = 'c'.repeat(64)]) {
    const f = fixture(); change(f); await assert.rejects(verifyPdfRelease(f.options));
  }
});
test('rejects truncated, changed and oversized source bodies', async () => {
  for (const body of [Buffer.from('short'), Buffer.alloc(33, 'x'), Buffer.alloc(100)]) {
    const f = fixture(); f.files[f.manifest.source_archive.name] = body;
    await assert.rejects(verifyPdfRelease(f.options));
  }
});
test('restarts a broken stream instead of mixing attempts', async () => {
  const f = fixture(); let attempts = 0;
  const result = await verifyPdfRelease({ ...f.options, fetchImpl: async (url, init) => {
    if (url.endsWith('.tar.gz') && ++attempts === 1) return { ok: true, body: (async function* () {
      yield Buffer.from('discard'); throw Object.assign(new Error('reset'), { code: 'ECONNRESET' });
    })() };
    return f.options.fetchImpl(url, init);
  } });
  assert.equal(attempts, 2); assert.equal(result.source.sha256, sha(f.source));
});
test('bounds temporary HTTP retries and does not retry permanent HTTP failures', async () => {
  for (const [status, expected] of [[503, 3], [404, 1], [403, 1]]) {
    const f = fixture(); let attempts = 0;
    await assert.rejects(verifyPdfRelease({ ...f.options, fetchImpl: async () => {
      attempts++; return new Response('unavailable', { status });
    } }), new RegExp(`HTTP ${status}`));
    assert.equal(attempts, expected);
  }
});
