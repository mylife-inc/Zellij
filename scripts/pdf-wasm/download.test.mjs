import assert from 'node:assert/strict';
import { test } from 'node:test';
import http from 'node:http';
import { mkdtemp, mkdir, readFile, readdir, writeFile, rm, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fetchResource, downloadBundle } from './download.mjs';
import { sha256 } from './bundle.mjs';

const content = Buffer.from('synthetic pinned resource');
const resource = url => ({ name: 'fixture', url, bytes: content.length, sha256: sha256(content) });
const options = { sleep: async () => {}, report: () => {} };

test('original resources require their prepared bytes and never fall back to HTTP', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'zel-original-download-'));
  const entry = { name: 'original', original: 'synthetic', bytes: content.length, sha256: sha256(content) };
  let requests = 0;
  const fetcher = async () => { requests++; throw new Error('unexpected network request'); };
  try {
    await assert.rejects(fetchResource(entry, { fetcher }), /Prepare the pinned original/);
    await assert.rejects(fetchResource({ ...entry, url: 'unused' }, { fetcher, originals: root }), /Prepare the pinned original/);
    await writeFile(path.join(root, entry.sha256), content);
    assert.deepEqual(await fetchResource(entry, { fetcher, originals: root }), content);
    await writeFile(path.join(root, entry.sha256), Buffer.alloc(content.length));
    await assert.rejects(fetchResource(entry, { fetcher, originals: root }), /integrity failed/);
    const link = path.join(root, 'linked');
    await symlink(root, link, process.platform === 'win32' ? 'junction' : 'dir');
    await assert.rejects(fetchResource(entry, { fetcher, originals: link }), /real directory/);
    assert.equal(requests, 0);
  } finally { await rm(root, { recursive: true, force: true }); }
});
async function server(handler, run) {
  const app = http.createServer(handler);
  await new Promise(resolve => app.listen(0, '127.0.0.1', resolve));
  try { await run(`http://127.0.0.1:${app.address().port}/resource`); }
  finally { app.closeAllConnections(); await new Promise(resolve => app.close(resolve)); }
}

test('a reset before headers is retried without weakening the pinned hash', async () => {
  let requests = 0;
  await server((req, res) => {
    if (++requests === 1) req.socket.destroy();
    else res.end(content);
  }, async url => assert.deepEqual(await fetchResource(resource(url), options), content));
  assert.equal(requests, 2);
});

test('a reset during a streamed body discards partial bytes before retry', async () => {
  let requests = 0;
  await server((req, res) => {
    if (++requests === 1) {
      res.writeHead(200, { 'Content-Length': content.length });
      res.write(content.subarray(0, 5));
      setTimeout(() => req.socket.destroy(), 20);
    } else res.end(content);
  }, async url => assert.deepEqual(await fetchResource(resource(url), options), content));
  assert.equal(requests, 2);
});

test('temporary HTTP errors use a bounded schedule and stop after three attempts', async () => {
  for (const status of [408, 429, 500, 502, 503, 504]) {
    let requests = 0;
    const waits = [];
    await server((_req, res) => { requests++; res.writeHead(status); res.end(); }, async url => {
      await assert.rejects(fetchResource(resource(url), { ...options, sleep: async n => waits.push(n) }),
        new RegExp(`after 3 attempt\\(s\\).*HTTP ${status}`));
    });
    assert.equal(requests, 3);
    assert.deepEqual(waits, [1000, 2000]);
  }
});

test('timeouts retry within the same fixed attempt limit', async () => {
  let attempts = 0;
  await assert.rejects(fetchResource(resource('unused'), { ...options,
    fetcher: async () => { attempts++; throw new DOMException('synthetic deadline', 'TimeoutError'); },
  }), /after 3 attempt/);
  assert.equal(attempts, 3);
});

test('the deadline also bounds a stalled response body', async () => {
  let requests = 0;
  await server((_req, res) => {
    requests++;
    res.writeHead(200, { 'Content-Length': content.length });
    res.write(content.subarray(0, 1));
  }, async url => {
    await assert.rejects(fetchResource(resource(url), { ...options, timeoutMs: 500 }), /after 3 attempt/);
  });
  assert.equal(requests, 3);
});

test('wrong hashes, short or oversized bodies and permanent HTTP failures never retry', async () => {
  for (const [status, body, message] of [[200, Buffer.alloc(content.length), 'Integrity failed'],
    [200, content.subarray(0, 2), 'Integrity failed'], [200, Buffer.concat([content, content]), 'Oversized resource'],
    [401, Buffer.from('private'), 'HTTP 401'], [404, Buffer.from('missing'), 'HTTP 404']]) {
    let requests = 0;
    await server((_req, res) => { requests++; res.writeHead(status); res.end(body); }, async url => {
      await assert.rejects(fetchResource(resource(url), options), new RegExp(message));
    });
    assert.equal(requests, 1);
  }
});

test('permanent connection errors and invalid limits fail without retries', async () => {
  let attempts = 0;
  const fetcher = async () => {
    attempts++;
    throw new TypeError('fetch failed', { cause: { code: 'CERT_HAS_EXPIRED' } });
  };
  await assert.rejects(fetchResource(resource('unused'), { ...options, fetcher }), /after 1 attempt/);
  assert.equal(attempts, 1);
  await assert.rejects(fetchResource({ ...resource('unused'), sha256: '../escape' }, { fetcher }), /Invalid pinned/);
  await assert.rejects(fetchResource(resource('unused'), { fetcher, timeoutMs: 60_001 }), /Invalid pinned/);
  assert.equal(attempts, 1);
  let corruptAttempts = 0;
  await assert.rejects(fetchResource(resource('unused'), { ...options, timeoutMs: 1, fetcher: async () => {
    corruptAttempts++;
    await new Promise(resolve => setTimeout(resolve, 20));
    return new Response(Buffer.alloc(content.length));
  } }), /Integrity failed/);
  assert.equal(corruptAttempts, 1, 'An expired deadline cannot turn a checksum failure into a retry');
});

test('failed transfers clean their stage and pre-existing or late-created outputs survive', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'zel-download-'));
  try {
    const out = path.join(root, 'bundle');
    const entry = resource('unused');
    let requests = 0;
    const fail = async () => { requests++; return new Response('', { status: 404 }); };
    await assert.rejects(downloadBundle(out, { ...options, fetcher: fail }, [entry]), /HTTP 404/);
    assert.deepEqual(await readdir(root), []);
    await mkdir(out); await writeFile(path.join(out, 'keep'), 'authored');
    await assert.rejects(downloadBundle(out, { ...options, fetcher: fail }, [entry]), /already exists/);
    assert.equal(requests, 1);
    assert.equal(await readFile(path.join(out, 'keep'), 'utf8'), 'authored');
    const late = path.join(root, 'late');
    await assert.rejects(downloadBundle(late, { ...options, fetcher: async () => {
      await mkdir(late); await writeFile(path.join(late, 'keep'), 'late authored');
      return new Response(content);
    } }, [entry]), /already exists/);
    assert.equal(await readFile(path.join(late, 'keep'), 'utf8'), 'late authored');
    assert.deepEqual((await readdir(root)).sort(), ['bundle', 'late']);
    const valid = path.join(root, 'valid');
    await downloadBundle(valid, { ...options, fetcher: async () => new Response(content) }, [entry, { ...entry, name: 'alias' }]);
    assert.deepEqual(await readdir(valid), [entry.sha256]);
    assert.deepEqual(await readFile(path.join(valid, entry.sha256)), content);
  } finally { await rm(root, { recursive: true, force: true }); }
});
