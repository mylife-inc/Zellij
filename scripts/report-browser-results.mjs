import { appendFile, readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

// Report Playwright's outcomes without changing its assertions or retry policy.
export function browserResultSummary(report) {
  const counts = {};
  for (const key of ['expected', 'unexpected', 'flaky', 'skipped']) {
    const value = report.stats?.[key];
    if (!Number.isSafeInteger(value) || value < 0) {
      throw new Error(`Invalid Playwright result count: ${key}`);
    }
    counts[key] = value;
  }
  if (!Array.isArray(report.errors)) throw new Error('Missing Playwright runner error inventory');
  counts.runnerErrors = report.errors.length;
  const lines = [
    '### Browser test outcomes', '',
    '| Expected | Flaky (retry needed) | Unexpected | Skipped | Runner errors |',
    '| ---: | ---: | ---: | ---: | ---: |',
    `| ${counts.expected} | ${counts.flaky} | ${counts.unexpected} | ${counts.skipped} | ${counts.runnerErrors} |`, '',
  ];
  if (counts.flaky) {
    lines.push('A passing retry does not resolve the first failure. Inspect the retained report, traces and browser-state snapshots.', '');
  }
  return { counts, markdown: `${lines.join('\n')}\n` };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (!process.argv[2]) throw new Error('Usage: node scripts/report-browser-results.mjs <playwright-report.json>');
  const { counts, markdown } = browserResultSummary(JSON.parse(await readFile(process.argv[2], 'utf8')));
  process.stdout.write(markdown);
  if (process.env.GITHUB_STEP_SUMMARY) await appendFile(process.env.GITHUB_STEP_SUMMARY, markdown);
  if (counts.flaky) {
    console.log(`::warning title=Browser tests required retries::${counts.flaky} flaky test(s). Investigate the first failures in playwright-failure-details.`);
  }
}
