// Integration regressions for the actual acceptance command, using copies of
// the built docs. No production requests or changes to the authored build.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { cp, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const run = promisify(execFile);
const repo = fileURLToPath(new URL('../', import.meta.url));
const checker = path.join(repo, 'scripts/check-docs-preservation.mjs');

for (const regression of [
  {
    name: 'recoverable hydration errors from changed body text',
    marker: '>Skip to content</a>',
    replacement: '>Changed by the hosting fixture</a>',
    // React's reporting channel differs between browser integrations.
    kind: /console\.error|pageerror/,
    diagnostic: /Minified React error #418|Hydration failed/,
  },
  {
    name: 'uncaught browser exceptions',
    marker: '</head>',
    replacement: '<script>throw new Error("preservation-runtime-sentinel")</script></head>',
    kind: /pageerror/,
    diagnostic: /preservation-runtime-sentinel/,
  },
  {
    name: 'console-only errors without an uncaught exception',
    scriptError: 'preservation-console-sentinel',
    kind: /console\.error/,
    diagnostic: /preservation-console-sentinel/,
  },
]) {
  test(`preservation rejects ${regression.name}`, { timeout: 120_000 }, async t => {
    const temp = await mkdtemp(path.join(tmpdir(), 'zellij-preservation-regression-'));
    t.after(() => rm(temp, { recursive: true, force: true }));
    const output = path.join(temp, 'out');
    await cp(path.join(repo, 'examples/zellij-docs/out'), output, { recursive: true });
    const home = path.join(output, 'index.html');
    const html = await readFile(home, 'utf8');
    if (regression.scriptError) {
      // Change only an existing JS asset, leaving the HTML/React tree intact.
      // The old pageerror-only checker would incorrectly accept this case.
      const source = html.match(/<script\b[^>]*\bsrc="(\/_next\/[^"?]+\.js)(?:\?[^"]*)?"/);
      assert(source, 'Built docs must reference a local Next.js runtime script');
      const script = path.join(output, source[1]);
      await writeFile(script, `${await readFile(script, 'utf8')}\nconsole.error(${JSON.stringify(regression.scriptError)});\n`);
    } else {
      assert(html.includes(regression.marker), 'Built documentation must contain the injection point');
      await writeFile(home, html.replace(regression.marker, regression.replacement));
    }
    await assert.rejects(run(process.execPath, [checker, output], {
      cwd: repo, timeout: 90_000, maxBuffer: 4 * 1024 * 1024,
    }), error => {
      assert.equal(error.code, 1, 'Checker must fail normally, not hang or crash');
      assert.match(error.stderr, /Browser runtime errors at http:\/\/127\.0\.0\.1:\d+\//);
      assert.match(error.stderr, regression.kind);
      assert.match(error.stderr, regression.diagnostic);
      assert(!error.stdout.includes('Preservation passed:'), 'Bad HTML must not report success');
      return true;
    });
  });
}

// The old root layout fails these cases with React #418. Exercise the actual
// preservation command on every exported route, with no live hosting requests.
for (const [name, prefix] of [
  ['head whitespace', '\n'],
  ['hosting annotation and CRLF whitespace', '\r\n<!-- This site is hosted on Netlify. -->\r\n'],
]) {
  test(`preservation accepts ${name}`, { timeout: 120_000 }, async t => {
    const temp = await mkdtemp(path.join(tmpdir(), 'zellij-head-acceptance-'));
    t.after(() => rm(temp, { recursive: true, force: true }));
    const output = path.join(temp, 'out');
    await cp(path.join(repo, 'examples/zellij-docs/out'), output, { recursive: true });
    let changed = 0;
    const marker = '<meta name="viewport"';
    for (const file of await readdir(output, { recursive: true })) {
      if (!file.endsWith('.html')) continue;
      const target = path.join(output, file);
      const html = await readFile(target, 'utf8');
      assert.equal(html.split(marker).length, 2, `${file}: expected one viewport insertion point`);
      await writeFile(target, html.replace(marker, prefix + marker));
      changed++;
    }
    assert(changed >= 55, 'Every documentation route must receive the hosting mutation');
    const result = await run(process.execPath, [checker, output], {
      cwd: repo, timeout: 90_000, maxBuffer: 4 * 1024 * 1024,
    });
    assert.match(result.stdout, /Preservation passed:/);
    assert.equal(result.stderr, '');
  });
}
