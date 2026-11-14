import assert from 'node:assert/strict';
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
