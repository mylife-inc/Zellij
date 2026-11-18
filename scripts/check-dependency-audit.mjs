// Keep both npm reports visible and reject every finding. The withdrawn
// picomatch advisory and its retired exception remain in the dated record.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function findings(report) {
  assert.equal(report?.auditReportVersion, 2, 'Unsupported/missing npm audit report');
  assert(!report.error, 'npm audit returned an error');
  assert(report.vulnerabilities && typeof report.vulnerabilities === 'object'
    && !Array.isArray(report.vulnerabilities), 'Missing vulnerability inventory');
  const entries = Object.entries(report.vulnerabilities);
  const counts = report.metadata?.vulnerabilities;
  assert(counts && Number.isInteger(counts.total), 'Missing audit totals');
  assert.equal(counts.total, entries.length, 'Inconsistent audit totals');
  for (const severity of ['info', 'low', 'moderate', 'high', 'critical']) {
    assert.equal(counts[severity], entries.filter(([, v]) => v.severity === severity).length,
      `Inconsistent ${severity} total`);
  }
  assert(entries.every(([, v]) => ['info', 'low', 'moderate', 'high', 'critical'].includes(v.severity)),
    'Unknown advisory severity');
  return entries;
}

export function verifyAudits(all, production) {
  assert.equal(findings(production).length, 0, 'Production dependency audit must be clean');
  assert.equal(findings(all).length, 0, 'Full dependency audit must be clean');
  return { findings: 0 };
}

export function parseAuditResult(result) {
  assert(!result.error && !result.signal, 'npm audit did not complete');
  assert([0, 1].includes(result.status), `npm audit failed with exit ${result.status}`);
  const report = JSON.parse(result.stdout);
  const count = findings(report).length;
  assert.equal(result.status, count ? 1 : 0, 'npm audit exit status disagrees with report');
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const audit = (args) => {
      const npmArgs = ['audit', '--json', '--audit-level=low', ...args];
      const command = process.platform === 'win32' ? process.execPath : 'npm';
      const argv = process.platform === 'win32'
        ? [path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js'), ...npmArgs]
        : npmArgs;
      const result = spawnSync(command, argv, { encoding: 'utf8', timeout: 120000, maxBuffer: 8 * 1024 * 1024 });
      process.stdout.write(result.stdout || '');
      process.stderr.write(result.stderr || '');
      return parseAuditResult(result);
    };
    const production = audit(['--omit=dev']);
    const all = audit(['--include=dev']);
    verifyAudits(all, production);
    console.log('Both production and development dependency audits are clean.');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
