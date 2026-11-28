// Repeat only the two original positive integration cases, without modifying
// their checker, 15-second hydration deadline, 90-second subprocess deadline,
// 120-second test deadline or request handling. Each round retains its outcome.
import { spawn } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
const output = path.resolve('scheduling-results/docs');
await mkdir(output, { recursive: true });
const outcomes = [];
for (let round = 1; round <= 6; round++) {
  const started = Date.now();
  let log = '';
  const child = spawn(process.execPath, ['--test', '--test-reporter=tap',
    '--test-name-pattern=^preservation accepts ', 'scripts/check-docs-preservation.test.mjs'], {
    env: { ...process.env, ZELLIJ_BROWSER_DIAGNOSTICS: path.join(output, `round-${round}-diagnostics`) },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  for (const stream of [child.stdout, child.stderr]) stream.on('data', bytes => {
    log += bytes.toString(); process.stdout.write(bytes);
  });
  const result = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', (code, signal) => resolve({ code, signal }));
  });
  await writeFile(path.join(output, `round-${round}.tap`), log);
  outcomes.push({ round, ...result, elapsedMs: Date.now() - started,
    passed: Number(log.match(/^# pass (\d+)$/m)?.[1] ?? 0),
    failed: Number(log.match(/^# fail (\d+)$/m)?.[1] ?? 0) });
  await writeFile(path.join(output, 'outcomes.json'), JSON.stringify(outcomes, null, 2) + '\n');
}
if (outcomes.some(result => result.code !== 0 || result.passed !== 2 || result.failed !== 0)) process.exitCode = 1;
