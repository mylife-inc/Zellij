// Exercise the shipped poller without starting a browser or server.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { setImmediate } from 'node:timers/promises';
import vm from 'node:vm';
import test from 'node:test';

const source = await readFile(new URL('../cli/crates/zellij-docs/assets/preview.js', import.meta.url), 'utf8');
function harness() {
  const events = new Map(), timers = new Set(), requests = [], notices = [];
  let reloads = 0;
  vm.runInNewContext(source, {
    URL, AbortController,
    document: {
      currentScript: { src: 'http://localhost:1234/manual/_zellij/preview.js?version=one' },
      body: { append: notice => notices.push(notice) },
      createElement: () => ({ style: {}, setAttribute() {}, remove() { notices.splice(notices.indexOf(this), 1); } }),
    },
    location: { reload() { reloads++; } },
    addEventListener(name, listener) { events.set(name, listener); },
    setTimeout(callback) { timers.add(callback); return callback; },
    clearTimeout(callback) { timers.delete(callback); },
    fetch(url, options) {
      return new Promise((resolve, reject) => {
        requests.push({ url: url.href, options, resolve, reject });
        options.signal?.addEventListener('abort', () => reject(options.signal.reason), { once: true });
      });
    },
  });
  return {
    requests, timers, notices, get reloads() { return reloads; },
    emit(name, value = {}) { events.get(name)?.(value); },
    tick() { const next = timers.values().next().value; assert(next); timers.delete(next); next(); },
    reply(request, state, ok = true) { request.resolve({ ok, json: async () => state }); },
  };
}

test('leaving the page aborts the pending poll and restoration starts exactly one new poll', async () => {
  const h = harness();
  assert.equal(h.requests[0].url, 'http://localhost:1234/manual/_zellij/preview.json');
  h.emit('pagehide');
  assert.equal(h.requests[0].options.signal?.aborted, true);
  h.emit('pageshow', { persisted: true });
  h.emit('pageshow', { persisted: true });
  await setImmediate();
  assert.equal(h.requests.length, 2);
  assert.equal(h.timers.size, 0, 'aborted poll must not start another loop');
  h.reply(h.requests[1], { version: 'one', error: null });
  await setImmediate();
  assert.equal(h.timers.size, 1);
  h.emit('pagehide');
  assert.equal(h.timers.size, 0);
});

test('a stale response body cannot reload or update a restored document', async () => {
  const h = harness();
  let finishBody;
  h.requests[0].resolve({ ok: true, json: () => new Promise(resolve => { finishBody = resolve; }) });
  await setImmediate();
  h.emit('pagehide');
  h.emit('pageshow', { persisted: true });
  finishBody({ version: 'old', error: 'stale failure' });
  await setImmediate();
  assert.equal(h.reloads, 0);
  assert.equal(h.notices.length, 0);
  assert.equal(h.timers.size, 0);
  h.reply(h.requests[1], { version: 'new' });
  await setImmediate();
  assert.equal(h.reloads, 1);
  assert.equal(h.timers.size, 0);
});

test('network and HTTP failures retry, build errors recover, and an ordinary pageshow never duplicates polling', async () => {
  const h = harness();
  h.emit('pageshow', { persisted: false });
  assert.equal(h.requests.length, 1);
  h.requests[0].reject(new Error('connection stopped'));
  await setImmediate(); h.tick();
  h.reply(h.requests[1], null, false);
  await setImmediate(); h.tick();
  h.reply(h.requests[2], { version: 'one', error: 'build failed' });
  await setImmediate();
  assert.equal(h.notices.length, 1);
  h.tick(); h.reply(h.requests[3], { version: 'one', error: null });
  await setImmediate();
  assert.equal(h.notices.length, 0);
  assert.equal(h.reloads, 0);
  assert.equal(h.timers.size, 1);
});
