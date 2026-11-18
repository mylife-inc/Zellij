import test from 'node:test';
import assert from 'node:assert/strict';
import { verifyAudits, parseAuditResult } from './check-dependency-audit.mjs';

function report(vulnerabilities = {}) {
  const counts = { info: 0, low: 0, moderate: 0, high: 0, critical: 0, total: 0 };
  for (const v of Object.values(vulnerabilities)) { counts[v.severity]++; counts.total++; }
  return { auditReportVersion: 2, vulnerabilities, metadata: { vulnerabilities: counts } };
}
const formerFinding = {
  name: 'picomatch', nodes: ['node_modules/picomatch'], severity: 'high',
  via: [{ url: 'https://github.com/advisories/GHSA-fc5q-39xc-vjpf',
    name: 'picomatch', dependency: 'picomatch', range: '<=4.0.7', severity: 'high' }],
};

test('accepts only clean production and full audit reports', () => {
  assert.deepEqual(verifyAudits(report(), report()), { findings: 0 });
});
test('rejects production findings even when the full report is clean', () => {
  assert.throws(() => verifyAudits(report(), report({ picomatch: formerFinding })), /Production dependency audit/);
});
test('rejects the formerly deferred advisory if a registry reports it again', () => {
  assert.throws(() => verifyAudits(report({ picomatch: formerFinding }), report()), /Full dependency audit/);
});
test('rejects development findings at every severity', () => {
  for (const severity of ['info', 'low', 'moderate', 'high', 'critical']) {
    assert.throws(() => verifyAudits(report({ example: { name: 'example', severity } }), report()), /Full dependency audit/);
  }
});
test('fails closed on malformed reports and inconsistent counts', () => {
  for (const malformed of [
    {}, { ...report(), error: { code: 'E503' } },
    { ...report(), auditReportVersion: 3 }, { ...report(), vulnerabilities: [] },
    { ...report(), metadata: {} },
    { ...report(), metadata: { vulnerabilities: { ...report().metadata.vulnerabilities, total: 1 } } },
    { ...report(), metadata: { vulnerabilities: { ...report().metadata.vulnerabilities, high: 1 } } },
    report({ example: { name: 'example', severity: 'unknown' } }),
  ]) {
    assert.throws(() => verifyAudits(malformed, report()));
    assert.throws(() => verifyAudits(report(), malformed));
  }
});
test('rejects registry errors, interrupted commands and exit/report disagreement', () => {
  for (const result of [
    { status: 1, stdout: '{"error":{"code":"E503"}}' },
    { status: 0, stdout: 'not json' },
    { status: 2, stdout: JSON.stringify(report()) },
    { status: 1, stdout: JSON.stringify(report()) },
    { status: 0, stdout: JSON.stringify(report({ picomatch: formerFinding })) },
    { status: 0, stdout: JSON.stringify(report()), signal: 'SIGTERM' },
    { status: 0, stdout: JSON.stringify(report()), error: new Error('timeout') },
  ]) assert.throws(() => parseAuditResult(result));
});
test('parses successful clean and findings reports without suppressing findings', () => {
  assert.deepEqual(parseAuditResult({ status: 0, stdout: JSON.stringify(report()) }), report());
  const findings = report({ picomatch: formerFinding });
  assert.deepEqual(parseAuditResult({ status: 1, stdout: JSON.stringify(findings) }), findings);
});
