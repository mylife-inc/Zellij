import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { browserResultSummary } from './report-browser-results.mjs';

test('clean outcomes stay distinct from skips and unexpected failures', () => {
  const result = browserResultSummary({ stats: { expected: 552, flaky: 0, unexpected: 1, skipped: 169 } });
  assert.match(result.markdown, /\| 552 \| 0 \| 1 \| 169 \|/);
  assert.doesNotMatch(result.markdown, /passing retry/);
});

test('missing, malformed and negative counts cannot produce a clean summary', () => {
  const stats = { expected: 552, flaky: 0, unexpected: 0, skipped: 169 };
  assert.throws(() => browserResultSummary({}), /Invalid Playwright result count/);
  for (const key of Object.keys(stats)) {
    for (const value of [undefined, '0', -1, 0.5, NaN, Infinity]) {
      assert.throws(() => browserResultSummary({ stats: { ...stats, [key]: value } }), /Invalid Playwright result count/);
    }
  }
});

test('a passing retry is retained in the Actions summary and warning without changing success', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'zellij-browser-report-'));
  try {
    const report = path.join(directory, 'results.json');
    const summary = path.join(directory, 'summary.md');
    await writeFile(report, JSON.stringify({ stats: { expected: 551, flaky: 1, unexpected: 0, skipped: 169 } }));
    const result = spawnSync(process.execPath, ['scripts/report-browser-results.mjs', report], {
      encoding: 'utf8', env: { ...process.env, GITHUB_STEP_SUMMARY: summary },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /::warning title=Browser tests required retries::1 flaky test/);
    const contents = await readFile(summary, 'utf8');
    assert.match(contents, /\| 551 \| 1 \| 0 \| 169 \|/);
    assert.match(contents, /passing retry does not resolve the first failure/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
