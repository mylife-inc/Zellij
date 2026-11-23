import assert from 'node:assert/strict';
import { test } from 'node:test';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, readdirSync, existsSync, rmSync, cpSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { compileProcess, runnerEnvironment } from './process.mjs';
import { compile } from './probe.mjs';

const bundle = process.env.ZEL_PDF_COMPACT_PACKAGE;
assert(bundle, 'Set ZEL_PDF_COMPACT_PACKAGE to the verified compact resource directory');
const absoluteBundle = path.resolve(bundle);
const runner = fileURLToPath(new URL('./process.mjs', import.meta.url));
const tex = body => `\\documentclass{article}\\begin{document}${body}\\end{document}`;

function fixture(body = 'A separately supervised PDF.') {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-pdf-process-'));
  const input = path.join(root, 'input');
  mkdirSync(input);
  writeFileSync(path.join(input, 'book.tex'), tex(body));
  return { root, input, output: path.join(root, 'output'), cleanup: () => rmSync(root, { recursive: true, force: true }) };
}
function clean(f) {
  assert(!existsSync(f.output));
  assert(!readdirSync(f.root).some(n => n.startsWith('.pdf-')));
}
function gone(pid) {
  assert(pid && pid !== process.pid);
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
}

test('separate process produces three passes with no PATH or startup hooks', { timeout: 60_000 }, async () => {
  const f = fixture();
  const hook = path.join(f.root, 'hook.cjs');
  const marker = path.join(f.root, 'injected');
  writeFileSync(hook, `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'unexpected');`);
  const old = Object.fromEntries(['NODE_OPTIONS', 'NODE_PATH', 'NODE_V8_COVERAGE', 'ZEL_TEST_SECRET'].map(k => [k, process.env[k]]));
  Object.assign(process.env, { NODE_OPTIONS: `--require=${JSON.stringify(hook)}`, NODE_PATH: f.root,
    NODE_V8_COVERAGE: path.join(f.root, 'coverage'), ZEL_TEST_SECRET: 'must-not-propagate' });
  try {
    const env = runnerEnvironment();
    for (const name of ['PATH', 'NODE_OPTIONS', 'NODE_PATH', 'ZEL_TEST_SECRET']) assert(!(name in env));
    assert.equal(env.NODE_V8_COVERAGE, '');
    const events = [];
    const result = await compileProcess(absoluteBundle, f.input, f.output, { onEvent: e => events.push(e) });
    assert.deepEqual(events.map(e => e.event), ['ready', 'pass', 'pass', 'pass', 'complete']);
    assert.deepEqual(events.filter(e => e.event === 'pass').map(e => e.pass), [1, 2, 3]);
    assert.equal(result.execution.kind, 'separate-process');
    assert.equal(result.execution.node, process.version);
    assert.equal(JSON.parse(readFileSync(path.join(f.output, 'probe.json'))).pdfSha256, result.pdfSha256);
    assert(!existsSync(marker), 'startup hook must not execute');
    assert(!existsSync(path.join(f.root, 'coverage')), 'coverage must not escape staging');
    gone(events[0].pid);
    assert(!readdirSync(f.root).some(n => n.startsWith('.pdf-')));
  } finally {
    for (const [k, v] of Object.entries(old)) { if (v === undefined) delete process.env[k]; else process.env[k] = v; }
    f.cleanup();
  }
});

for (const pass of [1, 2, 3]) {
  test(`cancellation at pass ${pass} reaps the process and removes only its stage`, { timeout: 60_000 }, async () => {
    const f = fixture();
    const control = new AbortController();
    let pid, reached = false;
    writeFileSync(path.join(f.root, 'authored.txt'), 'keep');
    try {
      await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, {
        signal: control.signal, onEvent: event => {
          pid = event.pid;
          if (event.event === 'pass' && event.pass === pass) { reached = true; control.abort(); }
        },
      }), /cancelled/);
      assert(reached);
      gone(pid);
      clean(f);
      assert.equal(readFileSync(path.join(f.root, 'authored.txt'), 'utf8'), 'keep');
    } finally { f.cleanup(); }
  });
}

test('already-cancelled work and invalid deadlines start no process or stage', async () => {
  const f = fixture();
  let events = 0;
  try {
    await assert.rejects(compileProcess('absent', f.input, f.output, {
      signal: AbortSignal.abort(), onEvent: () => events++,
    }), /abort/i);
    for (const value of [0, -1, 1.5, NaN, Infinity, 600_001]) {
      await assert.rejects(compileProcess('absent', f.input, f.output, { timeoutMs: value }), /deadline/);
    }
    await assert.rejects(compileProcess('absent', f.input, f.output, { passTimeoutMs: 120_001 }), /deadline/);
    assert.equal(events, 0);
    clean(f);
  } finally { f.cleanup(); }
});

test('whole-process and pass deadlines stop endless input and allow a later build', { timeout: 60_000 }, async () => {
  const f = fixture('\\directlua{while true do end}');
  try {
    let pid;
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, {
      timeoutMs: 2_000, onEvent: event => { pid = event.pid; },
    }), /deadline exceeded/);
    if (pid) gone(pid); // The whole-process deadline also covers slow startup.
    clean(f);
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, { passTimeoutMs: 500 }), /pass timed out/);
    clean(f);
    writeFileSync(path.join(f.input, 'book.tex'), tex('Recovered after deadlines.'));
    await compileProcess(absoluteBundle, f.input, f.output);
    assert(existsSync(path.join(f.output, 'book.pdf')));
  } finally { f.cleanup(); }
});

test('forced process death cannot publish a partial PDF or retain its temporary files', { timeout: 60_000 }, async () => {
  const f = fixture();
  let pid;
  try {
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, { onEvent: event => {
      pid = event.pid;
      if (event.event === 'pass' && event.pass === 2) process.kill(pid, 'SIGKILL');
    } }), /PDF process failed/);
    gone(pid); clean(f);
  } finally { f.cleanup(); }
});

test('an unresponsive process is forcibly reaped after cooperative cancellation', {
  timeout: 60_000, skip: process.platform === 'win32' && 'SIGSTOP is a Unix-only suspension mechanism',
}, async () => {
  const f = fixture();
  const control = new AbortController();
  let pid, stoppedAt;
  try {
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, {
      signal: control.signal, onEvent: event => {
        pid = event.pid;
        if (event.event === 'pass' && event.pass === 1) {
          process.kill(pid, 'SIGSTOP'); stoppedAt = performance.now(); control.abort();
        }
      },
    }), /cancelled/);
    assert(stoppedAt && performance.now() - stoppedAt < 5_000);
    gone(pid); clean(f);
  } finally {
    f.cleanup();
  }
});

test('parent control-pipe closure stops the standalone runner and its worker', { timeout: 60_000 }, async () => {
  const f = fixture('\\directlua{while true do end}');
  let child, closed = false;
  try {
    child = spawn(process.execPath, ['--max-old-space-size=128', runner, '--run', absoluteBundle, f.input, f.output, '30000', '30000'], {
      env: runnerEnvironment(), stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true,
    });
    const ended = once(child, 'close');
    child.once('close', () => { closed = true; });
    let text = '', reached = false, disconnectedAt;
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', data => {
      text += data;
      if (text.includes('"event":"pass"') && !reached) {
        reached = true; disconnectedAt = performance.now(); child.stdin.end();
      }
    });
    child.stderr.resume();
    const [code] = await ended;
    assert(reached && code !== 0);
    assert(performance.now() - disconnectedAt < 5_000, 'parent EOF must stop the runner before its 30-second deadline');
    gone(child.pid); clean(f);
  } finally {
    if (child && !closed) { const done = once(child, 'close'); child.kill('SIGKILL'); await done; }
    f.cleanup();
  }
});

test('invalid TeX and altered resources fail without publishing output', { timeout: 60_000 }, async () => {
  const f = fixture('Before error.\\undefinedZellijCommand');
  try {
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output), /LuaHBTeX exited/);
    clean(f);
    const corrupt = path.join(f.root, 'corrupt');
    cpSync(absoluteBundle, corrupt, { recursive: true });
    const m = JSON.parse(readFileSync(path.join(corrupt, 'manifest.json')));
    const file = path.join(corrupt, m.engine.find(x => x.name.endsWith('.wasm')).sha256);
    const bytes = readFileSync(file); bytes[100] ^= 1; writeFileSync(file, bytes);
    await assert.rejects(compileProcess(corrupt, f.input, f.output), /integrity failed/);
    clean(f);
  } finally { f.cleanup(); }
});

test('existing and late-created destinations and cancellation after completion are preserved', { timeout: 60_000 }, async () => {
  const f = fixture();
  try {
    mkdirSync(f.output); writeFileSync(path.join(f.output, 'authored.txt'), 'keep');
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output), /already exists/);
    assert.equal(readFileSync(path.join(f.output, 'authored.txt'), 'utf8'), 'keep');
    rmSync(f.output, { recursive: true });
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, { onEvent: event => {
      if (event.event === 'complete') { mkdirSync(f.output); writeFileSync(path.join(f.output, 'late.txt'), 'keep'); }
    } }), /already exists/);
    assert.equal(readFileSync(path.join(f.output, 'late.txt'), 'utf8'), 'keep');
    rmSync(f.output, { recursive: true });
    const control = new AbortController();
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, { signal: control.signal, onEvent: event => {
      if (event.event === 'complete') control.abort();
    } }), /cancelled|abort/i);
    clean(f);
  } finally { f.cleanup(); }
});

test('an observer error reaps the process; in-process compilation also preserves a late empty directory', { timeout: 60_000 }, async () => {
  const f = fixture();
  try {
    let pid;
    await assert.rejects(compileProcess(absoluteBundle, f.input, f.output, { onEvent: event => {
      pid = event.pid; if (event.event === 'pass') throw new Error('Observer failed');
    } }), /Observer failed/);
    gone(pid); clean(f);
    await assert.rejects(compile(absoluteBundle, f.input, f.output, { onPass: pass => {
      if (pass === 3) mkdirSync(f.output);
    } }), /output appeared/);
    assert.deepEqual(readdirSync(f.output), []);
    assert(!readdirSync(f.root).some(n => n.startsWith('.pdf-')));
  } finally { f.cleanup(); }
});
