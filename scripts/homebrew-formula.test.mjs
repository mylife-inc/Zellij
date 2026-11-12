import assert from 'node:assert/strict';
import { test } from 'node:test';
import { homebrewFormula } from './homebrew-formula.mjs';
import { nativeTargets } from './release-policy.mjs';

const inventory = (tag, prefix = '') => nativeTargets.map((target, i) =>
  `${String(i + 1).repeat(64)}  ${prefix}zel-${tag}-${target}.${target.includes('windows') ? 'zip' : 'tar.gz'}`,
).join('\n') + '\n';

test('tap formula selects four verified native downloads and preserves preview identity', () => {
  const tag = 'v0.2.0-rc.4';
  const result = homebrewFormula(tag, inventory(tag));
  assert.match(result, /version "0.2.0-rc.4"/);
  assert.match(result, /development preview 0.2.0-rc.4/);
  for (const [i, target] of nativeTargets.entries()) {
    if (target.includes('windows')) { assert(!result.includes(target)); continue; }
    assert(result.includes(`releases/download/zellij-${tag}/zel-${tag}-${target}.tar.gz"\n      sha256 "${String(i+1).repeat(64)}"`));
  }
  assert.equal((result.match(/\burl "/g) || []).length, 4);
  assert.match(result, /installation-manager.*write "homebrew\\n"/);
  assert.match(result, /generate_completions_from_executable/);
  assert.match(result, /manual\/site\/index.html/);
  assert(!result.includes('latest'));
  assert.equal(result, homebrewFormula(tag, inventory(tag, './').replaceAll('\n', '\r\n')));
});

test('stable formula has no preview label and retains exact asset/image pins', () => {
  const result = homebrewFormula('v1.0.0', inventory('v1.0.0'));
  assert(!result.includes('development preview'));
  assert.match(result, /zellij-base:1.0.0/);
});

test('incomplete, ambiguous or injected release metadata cannot generate a formula', () => {
  const tag = 'v0.2.0-rc.4';
  const sums = inventory(tag);
  for (const input of ['', sums.split('\n').slice(1).join('\n'), sums + sums.split('\n')[0], sums.replace(/1{64}/, 'invalid'), sums.replace('x86_64-apple-darwin', 'wrong-target')]) {
    assert.throws(() => homebrewFormula(tag, input));
  }
  for (const badTag of ['v0.2.0-rc.4\n', 'v0.2.0-rc.4"', 'main', 'v1.0.0+extra']) {
    assert.throws(() => homebrewFormula(badTag, sums));
  }
});
