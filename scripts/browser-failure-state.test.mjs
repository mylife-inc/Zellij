import assert from 'node:assert/strict';
import vm from 'node:vm';
import { test } from 'node:test';
import { browserFailureState } from './browser-failure-state.mjs';

test('a closed or disconnected page produces evidence without throwing', async () => {
  const state = await browserFailureState({ evaluate: async () => { throw new Error('Page has been closed'); } });
  assert.match(state.captureError, /Page has been closed/);
});

test('an unresponsive page cannot hold the failure hook indefinitely', { timeout: 5_000 }, async () => {
  const state = await browserFailureState({ evaluate: () => new Promise(() => {}) });
  assert.match(state.captureError, /exceeded 1500 ms/);
});

test('a synchronous capture failure preserves an error result', async () => {
  const state = await browserFailureState({ evaluate: () => { throw new Error('Disconnected'); } });
  assert.match(state.captureError, /Disconnected/);
});

test('no-script capture returns native focus state without needing page timers or frames', async () => {
  const state = await browserFailureState({ evaluate: async (fn, args) => vm.runInNewContext(`(${fn.toString()})(args)`, {
    args, performance: { now: () => 1 }, location: { href: 'file:///book/chapter.html#theorem' },
    innerWidth: 1280, innerHeight: 900, scrollX: 0, scrollY: 600,
    document: { readyState: 'complete', visibilityState: 'visible', hasFocus: () => true,
      fonts: { status: 'loaded' }, activeElement: { tagName: 'DIV', id: 'theorem', className: '', getAttribute: () => null },
      querySelector: () => null, querySelectorAll: () => [],
    },
    setTimeout: () => { throw new Error('Page timers are disabled'); },
    requestAnimationFrame: () => { throw new Error('Page frames are disabled'); },
  }) }, { sampleFrames: false });
  assert.equal(state.before.active.id, 'theorem');
  assert.equal(state.before.documentFocus, true);
  assert.equal(state.after.readyState, 'complete');
  assert.equal(state.frameTimes.length, 0);
  assert.equal(state.sampledFrames, false);
});
