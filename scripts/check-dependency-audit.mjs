// Keep both npm reports visible. One unpatched, development-only advisory has
// a dated disposition; this is not a general severity or package allowlist.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const disposition = {
  url: 'https://github.com/advisories/GHSA-fc5q-39xc-vjpf',
  expires: '2026-12-01T00:00:00.000Z',
  packages: {
    fdir: ['node_modules/fdir', '6.5.0', ['picomatch']],
    picomatch: ['node_modules/picomatch', '4.0.5', []],
    sucrase: ['node_modules/sucrase', '3.35.1', ['tinyglobby']],
    tinyglobby: ['node_modules/tinyglobby', '0.2.17', ['fdir', 'picomatch']],
    tsup: ['node_modules/tsup', '8.5.1', ['tinyglobby']],
    vite: ['node_modules/vite', '7.3.7', ['fdir', 'picomatch', 'tinyglobby']],
    vitest: ['engine/packages/zellij/node_modules/vitest', '4.1.12', ['picomatch', 'tinyglobby']],
  },
};

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

export function verifyAudits(all, production, lock, now = new Date()) {
  assert.equal(findings(production).length, 0, 'Production dependency audit must be clean');
  const entries = findings(all);
  if (!entries.length) return { findings: 0, deferred: false };
  assert(Number.isFinite(now.getTime()) && now >= new Date('2026-11-17T00:00:00.000Z')
    && now < new Date(disposition.expires), 'Development advisory disposition expired or clock invalid');
  assert.equal(lock?.lockfileVersion, 3, 'Expected reviewed npm lockfile');
  for (const [name, finding] of entries) {
    const reviewed = disposition.packages[name];
    assert(reviewed, `Unreviewed vulnerable package: ${name}`);
    const [node, version, via] = reviewed;
    assert.equal(finding.name, name, 'Inconsistent advisory name');
    assert.equal(finding.severity, 'high', `Changed severity for ${name}`);
    assert.deepEqual(finding.nodes, [node], `Changed dependency location for ${name}`);
    assert.equal(lock.packages?.[node]?.dev, true, `${name} must remain development-only`);
    assert.equal(lock.packages[node].version, version, `Unreviewed ${name} version`);
    if (name === 'picomatch') {
      assert.equal(finding.via?.length, 1, 'Additional picomatch advisory');
      const advisory = finding.via[0];
      assert.equal(advisory.url, disposition.url, 'Unreviewed picomatch advisory');
      assert.equal(advisory.name, 'picomatch');
      assert.equal(advisory.dependency, 'picomatch');
      assert.equal(advisory.range, '<=4.0.7', 'Advisory range changed; re-review upstream');
      assert.equal(advisory.severity, 'high');
    } else {
      assert.deepEqual([...finding.via].sort(), via, `Unreviewed advisory chain for ${name}`);
      assert(finding.via.every(parent => all.vulnerabilities[parent]), 'Missing root advisory');
    }
  }
  return { findings: entries.length, deferred: true };
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
    const result = verifyAudits(all, production, JSON.parse(readFileSync('package-lock.json', 'utf8')));
    console.log(result.deferred
      ? `WARNING: ${result.findings} development findings trace to ${disposition.url}; reviewed scope only, expires ${disposition.expires}. See specs/zellij-picomatch-disposition-2026-11-17.md. No production findings.`
      : 'Both production and development dependency audits are clean.');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
