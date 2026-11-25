// Public release acceptance only; no credentials, archiver or whole-source buffering.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';

const transient = new Set(['ECONNRESET', 'EAI_AGAIN', 'ETIMEDOUT', 'UND_ERR_SOCKET',
  'UND_ERR_CONNECT_TIMEOUT', 'UND_ERR_HEADERS_TIMEOUT', 'UND_ERR_BODY_TIMEOUT', 'PDF_HTTP_TRANSIENT']);
async function transfer(url, limit, fetchImpl, collect = false) {
  for (let attempt = 1; ; attempt++) {
    try {
      const response = await fetchImpl(url, { signal: AbortSignal.timeout(180_000) });
      if (!response.ok) {
        await response.body?.cancel();
        const error = new Error(`PDF release download returned HTTP ${response.status}: ${url}`);
        if ([408, 429, 500, 502, 503, 504].includes(response.status)) error.code = 'PDF_HTTP_TRANSIENT';
        throw error;
      }
      assert(response.body, `PDF release download has no body: ${url}`);
      let bytes = 0;
      const hash = createHash('sha256'), chunks = [];
      for await (const chunk of response.body) {
        bytes += chunk.length;
        assert(bytes <= limit, 'PDF release asset exceeds its size limit');
        hash.update(chunk);
        if (collect) chunks.push(chunk);
      }
      return { bytes, sha256: hash.digest('hex'), ...(collect ? { text: Buffer.concat(chunks).toString('utf8') } : {}) };
    } catch (error) {
      if (attempt === 3 || !(error.name === 'TimeoutError' || transient.has(error.code) || transient.has(error.cause?.code))) throw error;
      await delay(attempt * 250);
    }
  }
}

export async function verifyPdfRelease({ base, manifest, fetchImpl = fetch }) {
  assert.equal(manifest.format, 2);
  assert.equal(manifest.kind, 'zellij-pdf-toolchain');
  assert.equal(manifest.redistribution_review_complete, true);
  assert.match(manifest.version, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
  const source = manifest.source_archive;
  assert.equal(source.name, `zel-pdf-sources-v${manifest.version}.tar.gz`);
  assert(Number.isSafeInteger(source.bytes) && source.bytes > 0 && source.bytes <= 1024 * 1024 * 1024);
  assert.match(source.sha256, /^[a-f0-9]{64}$/);
  assert.match(manifest.source_manifest_sha256, /^[a-f0-9]{64}$/);
  const runtime = `zel-pdf-v${manifest.version}.tar.gz`;
  const url = name => `${base.replace(/\/+$/, '')}/${name}`;
  const combined = (await transfer(url('SHA256SUMS'), 1024 * 1024, fetchImpl, true)).text.trim()
    .split(/\r?\n/).map(line => line.match(/^([a-f0-9]{64})  (?:\.\/)?(.+)$/));
  assert(combined.length && combined.every(Boolean), 'Malformed combined PDF checksum manifest');
  const hashes = {};
  for (const name of [runtime, source.name]) {
    const checksum = (await transfer(url(name + '.sha256'), 4096, fetchImpl, true)).text.trim();
    const matches = combined.filter(row => row[2] === name);
    assert.equal(matches.length, 1, `${name} must appear exactly once in SHA256SUMS`);
    assert.equal(checksum, `${matches[0][1]}  ${name}`, 'Per-file and combined PDF checksums differ');
    hashes[name] = matches[0][1];
  }
  assert.equal(hashes[source.name], source.sha256, 'Published source differs from runtime binding');
  const downloaded = await transfer(url(source.name), source.bytes, fetchImpl);
  assert.equal(downloaded.bytes, source.bytes, 'Incomplete PDF source archive');
  assert.equal(downloaded.sha256, source.sha256, 'PDF source checksum mismatch');
  return { runtime: { name: runtime, sha256: hashes[runtime] }, source,
    sourceManifestSha256: manifest.source_manifest_sha256, sourceBytesVerified: true };
}
