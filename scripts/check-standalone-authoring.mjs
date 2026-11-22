/** Exercise the real cached renderer, validation loop and rich preview on each OS. */
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile, readdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

export async function checkStandaloneAuthoring(binary, environment, parent) {
  const root = path.join(parent, 'authoring 日本');
  const windows = process.platform === 'win32';
  const consoleRunner = path.join(parent, 'console.exe');
  const browser = path.join(parent, windows ? 'browser.exe' : 'browser');
  const record = path.join(parent, 'opened-url.txt');
  const env = { ...environment, ZELLIJ_PREVIEW_RECORD: record, ZELLIJ_PREVIEW_EXPECT: 'Authoring' };
  const active = new Set();
  const browsers = [];
  const errors = [];
  const put = async (name, body) => {
    const file = path.join(root, name);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, body);
  };
  const config = 'version: 1\nrenderer: hosted\npreview: {port: 0}\n';
  const settings = 'name: Authoring\nhome: guide\nbasePath: /manual\n';
  const page = 'content/guide/index.mdx';
  const original = '# Original\n\n[Next](/guide/next)\n';
  await put('zel.yaml', config);
  await put('content/site.yaml', settings);
  await put('content/guide/_sidebar.yaml', 'nav: [index, next]\n');
  await put(page, original);
  await put('content/guide/next.mdx', '# Next\n\nMore documentation.\n');
  execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-browser.rs', '-o', browser], { stdio: 'inherit' });
  if (windows) execFileSync('rustc', ['--edition=2024', 'scripts/fixtures/preview-console.rs', '-o', consoleRunner], { stdio: 'inherit' });

  function start(args, { persistent = false, overrides = {} } = {}) {
    const wrap = windows && persistent;
    const state = { output: '', stderr: '', exit: undefined };
    state.child = spawn(wrap ? consoleRunner : binary, wrap ? [binary, ...args] : args,
      { cwd: root, env: { ...env, ...overrides }, stdio: ['pipe', 'pipe', 'pipe'] });
    state.child.stdout.on('data', d => { state.output += d; });
    state.child.stderr.on('data', d => { state.stderr += d; });
    state.closed = new Promise((resolve, reject) => {
      state.child.once('error', reject);
      state.child.once('close', (code, signal) => { state.exit = { code, signal }; resolve(state.exit); });
    });
    active.add(state);
    return state;
  }
  async function until(predicate, label, state, timeout = 90000) {
    for (const deadline = Date.now() + timeout; Date.now() < deadline;) {
      if (await predicate()) return;
      assert(!state?.exit, `${label}: process exited\n${state?.output}\n${state?.stderr}`);
      await delay(100);
    }
    throw new Error(`${label}: timed out\n${state?.output}\n${state?.stderr}`);
  }
  async function run(args, code = 0, overrides = {}) {
    const state = start(args, { overrides });
    await until(() => state.exit, 'command completion', undefined);
    await state.closed;
    active.delete(state);
    assert.equal(state.exit.code, code, `${args.join(' ')}\n${state.output}\n${state.stderr}`);
    return state;
  }
  async function stop(state) {
    if (windows) state.child.stdin.end('stop\n'); else state.child.kill('SIGINT');
    await until(() => state.exit, 'Ctrl-C shutdown', undefined, 15000);
    await state.closed;
    active.delete(state);
    assert.equal(state.exit.code, 0, state.stderr);
    assert.match(state.output, /Stopped\./);
  }
  try {
    for (const target of ['file', 'static']) {
      const result = await run(['check', '--target', target, '--deep', '--strict', '--offline', '--json']);
      assert.equal(result.stderr, '');
      const report = JSON.parse(result.output);
      assert.equal(report.target, target); assert.equal(report.ok, true);
      assert(report.pages >= 2);
      assert(!(await readdir(root)).includes('site'));
      const missing = await run(['check', '--target', target, '--offline', '--json'], 2, { ZELLIJ_HOME: path.join(parent, 'uncached') });
      assert.equal(missing.stderr, ''); assert.match(JSON.parse(missing.output).diagnostics[0].message, /not cached/);
      await put(page, original + '\n{(() => { throw new Error("CHECK_MUST_NOT_EXECUTE"); })()}\n');
      const dynamic = await run(['check', '--target', target, '--deep', '--offline', '--json']);
      assert.equal(JSON.parse(dynamic.output).ok, true);
      assert.equal(dynamic.stderr, '');
      await put(page, original);
      await put('content/site.yaml', settings + 'unknownOption: true\n');
      const strict = await run(['check', '--target', target, '--strict', '--offline', '--json'], 3);
      assert(JSON.parse(strict.output).diagnostics.some(d => d.severity === 'warning'));
      await put('content/site.yaml', settings);
      await put('zel.yaml', 'version: [');
      const invalid = await run(['check', '--target', target, '--offline', '--json'], 2);
      assert.equal(invalid.stderr, ''); assert.equal(JSON.parse(invalid.output).ok, false);
      await put('zel.yaml', config);
      await put(page, '# Original\n\n[Broken](/does-not-exist)\n');
      const watch = start(['watch', '--target', target, '--deep', '--strict', '--offline'], { persistent: true });
      await until(() => watch.output.includes('Watching ') && watch.output.includes('Validation failed;'), 'invalid content remains watched', watch);
      await put(page, original);
      await until(() => watch.output.includes('Validation passed.'), 'validation recovery', watch);
      await stop(watch);
      assert(!(await readdir(root)).includes('site'));
    }

    console.log('Standalone file/static validation: JSON, offline cache, nonexecution, strict findings and watch recovery passed.');
    const preview = start(['view', '--watch', '--offline', '--browser', browser], { persistent: true });
    await until(() => preview.output.includes('Serving at '), 'rich preview startup', preview);
    const url = preview.output.match(/Serving at (http:\/\/\S+)/)[1];
    assert.match(url, /\/manual\/$/);
    await until(async () => { try { return (await readFile(record, 'utf8')) === url; } catch { return false; } }, 'browser launch', preview);
    const pageUrl = `${url}guide/`;
    assert.match(await (await fetch(pageUrl)).text(), /Original/);
    const generated = path.join(root, 'site/guide/index.html');
    assert.doesNotMatch(await readFile(generated, 'utf8'), /preview\.js/);
    const status = async () => (await fetch(`${url}_zellij/preview.json`)).json();
    let previous = await status();
    if (process.env.ZEL_STANDALONE_BROWSER === 'true') {
      const { chromium, webkit } = await import('@playwright/test');
      for (const engine of [chromium, webkit]) {
        const session = await engine.launch(engine === chromium && process.platform === 'darwin' ? { channel: 'chrome' } : {});
        const tab = await session.newPage();
        browsers.push({ session, tab });
        tab.on('pageerror', error => errors.push(error.message));
        await tab.goto(pageUrl);
        await tab.getByRole('heading', { name: 'Original', exact: true }).waitFor();
      }
    }
    await put(page, original.replace('Original', 'Rebuilt'));
    await until(async () => (await status()).version !== previous.version, 'successful rebuild', preview);
    for (const { tab } of browsers) await tab.getByRole('heading', { name: 'Rebuilt', exact: true }).waitFor();
    previous = await status();
    const preserved = await readFile(generated);
    await put(page, '---\ntitle: [\n---\n# Broken\n');
    await until(async () => (await status()).error, 'failed edit reported', preview);
    assert.equal((await status()).version, previous.version);
    assert((await readFile(generated)).equals(preserved));
    await put(page, original.replace('Original', 'Recovered'));
    await until(async () => { const s = await status(); return !s.error && s.version !== previous.version; }, 'preview recovery', preview);
    for (const { tab } of browsers) await tab.getByRole('heading', { name: 'Recovered', exact: true }).waitFor();
    await put('content/site.yaml', settings.replace('/manual', '/changed'));
    await until(async () => (await status()).error?.includes('basePath changed'), 'preview URL stays pinned', preview);
    await put('content/site.yaml', settings);
    await until(async () => !(await status()).error, 'prefix recovery', preview);
    assert.deepEqual(errors, []);
    await stop(preview);
    const beforeCancel = await readFile(generated);
    const cancelled = start(['view', '--offline', '--no-open'], { persistent: true });
    await until(() => cancelled.output.includes('cached rich renderer'), 'build started', cancelled);
    await stop(cancelled);
    assert((await readFile(generated)).equals(beforeCancel));
    assert(!(await readdir(root)).some(name => name.startsWith('.zel-build-')));
    // Legacy `.` discovers content/ once; output and repository files are not inputs.
    await rm(path.join(root, 'zel.yaml'));
    const legacyWatch = start(['watch', '.', '--offline', '--deep', '--strict'], { persistent: true });
    await until(() => legacyWatch.output.includes('Validation passed.'), 'legacy validation discovery', legacyWatch);
    await put('unrelated.md', '# Repository file\n');
    await delay(1400);
    assert.equal(legacyWatch.output.split('Checking ').length - 1, 1);
    await stop(legacyWatch);
    const legacyPreview = start(['view', '.', '--offline', '--watch', '--no-open', '--port', '0'], { persistent: true });
    await until(() => legacyPreview.output.includes('Serving at '), 'legacy preview discovery', legacyPreview);
    await put('unrelated.md', '# Unrelated change\n');
    await delay(1400);
    assert.equal(legacyPreview.output.split('cached rich renderer').length - 1, 1, 'Publishing output must not trigger another build');
    assert.match(await readFile(path.join(root, 'dist/guide/index.html'), 'utf8'), /Recovered/);
    await stop(legacyPreview);
    console.log(JSON.stringify({ standaloneAuthoring: true, check: true, watch: true, view: true, recovery: true, cancellation: true, prefixPinned: true, legacyDiscovery: true, browserEngines: browsers.length, systemTools: false }));
  } finally {
    for (const { session } of browsers) await session.close();
    for (const state of active) {
      if (!state.exit) { if (windows) state.child.stdin.end('stop\n'); else state.child.kill('SIGKILL'); }
      await state.closed;
    }
  }
}
