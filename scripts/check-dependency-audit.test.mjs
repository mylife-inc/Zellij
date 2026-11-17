import test from 'node:test';
import assert from 'node:assert/strict';
import { disposition, verifyAudits, parseAuditResult } from './check-dependency-audit.mjs';

const now = new Date('2026-11-17T21:00:00Z');
function report(vulnerabilities = {}) {
  const counts = { info: 0, low: 0, moderate: 0, high: 0, critical: 0, total: 0 };
  for (const v of Object.values(vulnerabilities)) { counts[v.severity]++; counts.total++; }
  return { auditReportVersion: 2, vulnerabilities, metadata: { vulnerabilities: counts } };
}
function fixture() {
  const lock = { lockfileVersion: 3, packages: {} }, vulnerabilities = {};
  for (const [name, [node, version, via]] of Object.entries(disposition.packages)) {
    lock.packages[node] = { version, dev: true };
    vulnerabilities[name] = { name, nodes: [node], severity: 'high', via: [...via] };
  }
  vulnerabilities.picomatch.via = [{ url: disposition.url, name: 'picomatch', dependency: 'picomatch', range: '<=4.0.7', severity: 'high' }];
  return { all: report(vulnerabilities), production: report(), lock };
}
const verify = (f, date = now) => verifyAudits(f.all, f.production, f.lock, date);

test('accepts clean audits and keeps the exact development disposition visible', () => {
  assert.deepEqual(verifyAudits(report(), report(), {}, now), { findings: 0, deferred: false });
  assert.deepEqual(verify(fixture()), { findings: 7, deferred: true });
});
test('never permits a production finding, including the same advisory', () => {
  const f = fixture(); f.production = f.all;
  assert.throws(() => verify(f), /Production dependency audit/);
});
test('rejects a new advisory, package, severity or changed advisory range', () => {
  for (const mutate of [
    f => f.all.vulnerabilities.picomatch.via.push({ url: 'https://github.com/advisories/GHSA-new' }),
    f => { f.all.vulnerabilities.picomatch.via[0].url += '-new'; },
    f => { f.all.vulnerabilities.picomatch.via[0].range = '*'; },
    f => { f.all.vulnerabilities.picomatch.severity = 'critical'; },
    f => { f.all.vulnerabilities.unknown = { name: 'unknown', severity: 'high' }; },
    f => { f.all.vulnerabilities.vite.via.push('unknown'); },
  ]) {
    const f = fixture(); mutate(f); f.all = report(f.all.vulnerabilities);
    assert.throws(() => verify(f));
  }
});
test('rejects production promotion, changed lock versions or additional installations', () => {
  for (const mutate of [
    f => { f.lock.packages['node_modules/picomatch'].dev = false; },
    f => { f.lock.packages['node_modules/vite'].version = '8.0.0'; },
    f => { f.all.vulnerabilities.picomatch.nodes.push('another/node_modules/picomatch'); },
    f => { delete f.all.vulnerabilities.picomatch; f.all = report(f.all.vulnerabilities); },
  ]) { const f = fixture(); mutate(f); assert.throws(() => verify(f)); }
});
test('requires re-review at expiry and rejects an invalid or earlier clock', () => {
  for (const date of [new Date(disposition.expires), new Date('2027-01-01'), new Date('invalid'), new Date('2026-11-16')]) {
    assert.throws(() => verify(fixture(), date), /expired or clock invalid/);
  }
});
test('fails closed on registry failures, malformed reports and inconsistent totals', () => {
  for (const result of [
    { status: 1, stdout: '{"error":{"code":"E503"}}' },
    { status: 0, stdout: 'not json' },
    { status: 2, stdout: JSON.stringify(report()) },
    { status: 1, stdout: JSON.stringify(report()) },
    { status: 0, stdout: JSON.stringify(report()), signal: 'SIGTERM' },
    { status: 0, stdout: JSON.stringify(report()), error: new Error('timeout') },
    { status: 0, stdout: JSON.stringify({ ...report(), auditReportVersion: 3 }) },
  ]) assert.throws(() => parseAuditResult(result));
  const f = fixture(); f.all.metadata.vulnerabilities.high = 0;
  assert.throws(() => verify(f), /Inconsistent/);
  assert.deepEqual(parseAuditResult({ status: 0, stdout: JSON.stringify(report()) }), report());
  assert.deepEqual(parseAuditResult({ status: 1, stdout: JSON.stringify(fixture().all) }), fixture().all);
});
