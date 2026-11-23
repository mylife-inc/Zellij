// Exercise the Rust owner, not the JavaScript acceptance adapter.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, readdirSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const exec = promisify(execFile);
const harness = path.resolve(process.env.ZEL_PDF_RUST_RUNNER || `cli/target/debug/examples/pdf_process${process.platform === 'win32' ? '.exe' : ''}`);
const script = fileURLToPath(new URL('./process.mjs', import.meta.url));
const bundle = path.resolve(process.env.ZEL_PDF_COMPACT_PACKAGE || 'test-results/pdf-compact/package');
assert(existsSync(harness), 'Build the pdf_process example first');
assert(existsSync(bundle), 'Prepare the compact PDF package first');
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
function fixture(body = 'A Rust-supervised PDF.') {
  const root = mkdtempSync(path.join(tmpdir(), 'zel-rust-pdf-'));
  mkdirSync(path.join(root, 'input'));
  writeFileSync(path.join(root, 'input/book.tex'), `\\documentclass{article}\\begin{document}${body}\\end{document}`);
  writeFileSync(path.join(root, 'authored.txt'), 'keep');
  return root;
}
function args(root, runner = script, node = process.execPath) {
  return [node, runner, bundle, path.join(root, 'input'), path.join(root, 'output')];
}
function env(root, extra = {}) {
  const environment = { ...process.env };
  for (const key of Object.keys(environment)) if (key.startsWith('ZEL_PDF_TEST_')) delete environment[key];
  return { ...environment, ZEL_PDF_TEST_PID: path.join(root, 'pid'), ...extra };
}
function alive(pid) { try { process.kill(pid, 0); return true; } catch (error) { if (error.code === 'ESRCH') return false; throw error; } }
function clean(root, output = false) {
  assert.equal(readFileSync(path.join(root, 'authored.txt'), 'utf8'), 'keep');
  assert.equal(existsSync(path.join(root, 'output')), output);
  assert(!readdirSync(root).some(name => name.startsWith('.pdf-rust-')));
  if (existsSync(path.join(root, 'pid'))) assert(!alive(Number(readFileSync(path.join(root, 'pid'), 'utf8'))), 'compiler must be reaped');
}
async function invoke(root, extra = {}, runner = script, node = process.execPath) {
  return exec(harness, args(root, runner, node), { env: env(root, extra), timeout: 60_000, maxBuffer: 1024 * 1024 });
}
function fake(root, body) {
  const file = path.join(root, 'fake.mjs');
  writeFileSync(file, `import fs from 'node:fs'; import path from 'node:path'; import {createHash} from 'node:crypto';
    const output=process.argv[5];
    const event=(event, fields={})=>process.stdout.write(JSON.stringify({event,pid:process.pid,...fields})+'\\n');
    fs.writeFileSync(${JSON.stringify(path.join(root, 'pid'))}, String(process.pid));
    ${body}`);
  return file;
}

test('real Rust supervisor keeps protocol private in every presentation mode and blocks startup hooks', { timeout: 180_000 }, async () => {
  for (const options of [{}, { ZEL_PDF_TEST_QUIET: '1' }, { ZEL_PDF_TEST_NO_COLOR: '1' }, { ZEL_PDF_TEST_QUIET: '1', ZEL_PDF_TEST_NO_COLOR: '1' }]) {
    const root = fixture();
    try {
      const marker = path.join(root, 'hook-ran');
      const hook = path.join(root, 'hook.cjs');
      writeFileSync(hook, `require('node:fs').writeFileSync(${JSON.stringify(marker)},'bad')`);
      const result = await invoke(root, { ...options, PATH: '', NODE_OPTIONS: `--require ${JSON.stringify(hook)}`, NODE_V8_COVERAGE: path.join(root, 'coverage') });
      assert.equal(result.stdout.trim(), 'verified');
      assert.equal(result.stderr, '');
      assert(!existsSync(marker)); assert(!existsSync(path.join(root, 'coverage')));
      const receipt = JSON.parse(readFileSync(path.join(root, 'output/probe.json')));
      assert.deepEqual(receipt.execution, { kind: 'rust-process', node: process.version });
      assert.equal(receipt.pdfSha256, createHash('sha256').update(readFileSync(path.join(root, 'output/book.pdf'))).digest('hex'));
      assert.equal(receipt.passes.length, 3);
      clean(root, true);
    } finally { rmSync(root, { recursive: true, force: true }); }
  }
});

for (const phase of ['ready', '1', '2', '3', 'complete']) {
  test(`Rust cancellation at ${phase} reaps the actual compiler before cleanup`, { timeout: 60_000 }, async () => {
    const root = fixture();
    try {
      await assert.rejects(invoke(root, { ZEL_PDF_TEST_CANCEL: phase }), /cancelled/);
      clean(root);
    } finally { rmSync(root, { recursive: true, force: true }); }
  });
}

test('pre-abort, invalid deadline, missing executable and observer errors preserve authored files', { timeout: 60_000 }, async () => {
  for (const scenario of [
    { options: { ZEL_PDF_TEST_PREABORT: '1' }, error: /cancelled/ },
    { options: { ZEL_PDF_TEST_TIMEOUT_MS: '0' }, error: /invalid.*deadline/ },
    { options: { ZEL_PDF_TEST_PASS_MS: '120001' }, error: /invalid.*deadline/ },
    { missing: true, error: /could not start/ },
    { options: { ZEL_PDF_TEST_OBSERVER_ERROR: '2' }, error: /observer failed/ },
  ]) {
    const root = fixture();
    try {
      await assert.rejects(invoke(root, scenario.options, script, scenario.missing ? path.join(root, 'missing-node') : process.execPath), scenario.error);
      clean(root);
    } finally { rmSync(root, { recursive: true, force: true }); }
  }
});

test('Rust total and pass deadlines stop a real endless guest; a later compilation succeeds', { timeout: 60_000 }, async () => {
  for (const options of [{ ZEL_PDF_TEST_TIMEOUT_MS: '1500' }, { ZEL_PDF_TEST_PASS_MS: '2000' }]) {
    const root = fixture('\\directlua{while true do end}');
    try {
      await assert.rejects(invoke(root, options), /deadline exceeded|timed out/);
      clean(root);
      writeFileSync(path.join(root, 'input/book.tex'), '\\documentclass{article}\\begin{document}Recovered.\\end{document}');
      await invoke(root); clean(root, true);
    } finally { rmSync(root, { recursive: true, force: true }); }
  }
});

test('one-second fallback reaps an uncooperative child on every OS', { timeout: 15_000 }, async () => {
  const root = fixture();
  try {
    const runner = fake(root, `process.stdin.resume(); process.stdin.on('end',()=>{}); setInterval(()=>{},1000); event('ready',{node:process.version});`);
    const start = Date.now();
    await assert.rejects(invoke(root, { ZEL_PDF_TEST_CANCEL: 'ready' }, runner), /cancelled/);
    assert(Date.now() - start >= 900); assert(Date.now() - start < 6000);
    clean(root);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('forced Rust parent exit closes the compiler liveness pipe', { timeout: 15_000 }, async () => {
  const root = fixture('\\directlua{while true do end}');
  try {
    await assert.rejects(invoke(root, { ZEL_PDF_TEST_EXIT_PARENT: '1' }), error => error.code === 71);
    const pid = Number(readFileSync(path.join(root, 'pid'), 'utf8'));
    const deadline = Date.now() + 5000;
    while (alive(pid) && Date.now() < deadline) await pause(20);
    assert(!alive(pid), 'parent EOF must stop the compiler before its independent 120-second pass deadline');
    assert(!existsSync(path.join(root, 'output')));
    assert.equal(readFileSync(path.join(root, 'authored.txt'), 'utf8'), 'keep');
    // A forcibly killed parent cannot run TempDir::drop; its stage is owned by
    // this harness and is removed only after its child has actually stopped.
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('external compiler termination is reaped and cannot publish partial output', { timeout: 60_000 }, async () => {
  const root = fixture('\\directlua{while true do end}');
  let child, ended, timer;
  try {
    child = spawn(harness, args(root), { env: env(root), stdio: ['ignore','pipe','pipe'] });
    let stderr = ''; child.stderr.on('data', b => stderr += b);
    ended = new Promise((resolve, reject) => { child.on('error', reject); child.on('close', code => resolve(code)); });
    timer = setTimeout(() => child.kill('SIGKILL'), 15_000);
    const deadline = Date.now() + 10_000;
    while (!existsSync(path.join(root, 'pid')) && Date.now() < deadline) await pause(10);
    const pid = Number(readFileSync(path.join(root, 'pid'), 'utf8'));
    process.kill(pid, 'SIGKILL');
    assert.notEqual(await ended, 0); assert.match(stderr, /process failed/); clean(root);
  } finally {
    clearTimeout(timer);
    if (child && child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
    if (ended) await ended;
    rmSync(root, { recursive: true, force: true });
  }
});

test('invalid protocol, oversized output and failed status are rejected without protocol leaks', { timeout: 30_000 }, async () => {
  for (const [body, pattern] of [
    [`event('ready',{node:process.version});event('pass',{pass:2});`, /event sequence/],
    [`process.stdout.write(JSON.stringify({event:'ready',node:process.version,pid:1})+'\\n');`, /invalid.*event/],
    [`event('ready',{node:process.version,extra:'unexpected'});`, /invalid.*event/],
    [`process.stdout.write('x'.repeat(16385));`, /budget/],
    [`process.stderr.write('x'.repeat(65537));`, /budget/],
    [`process.stdout.write('partial');`, /process failed/],
    [`event('ready',{node:process.version});process.stderr.write('\\u001b[31mspecific failure\\u001b[0m');process.exitCode=7;`, /specific failure/],
  ]) {
    const root = fixture();
    try {
      await assert.rejects(invoke(root, { ZEL_PDF_TEST_QUIET: '1', ZEL_PDF_TEST_NO_COLOR: '1' }, fake(root, body)), error => {
        assert.equal(error.stdout, ''); assert(!error.stderr.includes('\u001b')); assert.match(error.stderr, pattern); return true;
      }); clean(root);
    } finally { rmSync(root, { recursive: true, force: true }); }
  }
});

test('Rust validates completed artifacts and refuses a destination created during compilation', { timeout: 30_000 }, async () => {
  const valid = `fs.mkdirSync(output); const pdf=Buffer.from('%PDF-test\\n%%EOF');
    fs.writeFileSync(path.join(output,'book.pdf'),pdf); fs.writeFileSync(path.join(output,'book.log'),'log');
    for(const pass of [1,2,3]) fs.writeFileSync(path.join(output,'pass-'+pass+'.log'),'log');
    const receipt={pdfSha256:createHash('sha256').update(pdf).digest('hex'),passes:[1,2,3].map(pass=>({pass}))};
    fs.writeFileSync(path.join(output,'probe.json'),JSON.stringify(receipt));`;
  for (const [change, pattern, collision] of [
    [`fs.writeFileSync(path.join(output,'book.pdf'),'bad PDF');`, /invalid.*result/],
    [`fs.writeFileSync(path.join(output,'probe.json'),'{}');`, /invalid.*result/],
    [`fs.writeFileSync(path.join(output,'probe.json'),'broken');`, /invalid.*receipt/],
    [`fs.writeFileSync(path.join(output,'pass-3.log'),'different log');`, /final log differs/],
    [`fs.writeFileSync(path.join(output,'extra'),'unlisted');`, /unexpected.*files/],
    [`fs.mkdirSync(${JSON.stringify('DESTINATION')});`, /already exists/, true],
  ]) {
    const root = fixture();
    try {
      const alteration = change.replace(JSON.stringify('DESTINATION'), JSON.stringify(path.join(root, 'output')));
      const runner = fake(root, `event('ready',{node:process.version}); for(const pass of [1,2,3])event('pass',{pass}); ${valid} ${alteration} event('complete');`);
      await assert.rejects(invoke(root, {}, runner), pattern); clean(root, Boolean(collision));
      if (collision) assert.deepEqual(readdirSync(path.join(root, 'output')), []);
    } finally { rmSync(root, { recursive: true, force: true }); }
  }
});

test('existing output and invalid TeX preserve the previous result', { timeout: 60_000 }, async () => {
  const root = fixture('\\undefinedZellijCommand');
  try {
    mkdirSync(path.join(root, 'output'));
    writeFileSync(path.join(root, 'output/keep.txt'), 'prior');
    await assert.rejects(invoke(root), /already exists/);
    assert.equal(readFileSync(path.join(root, 'output/keep.txt'), 'utf8'), 'prior');
    clean(root, true);
    rmSync(path.join(root, 'output'), { recursive: true });
    await assert.rejects(invoke(root), /LuaHBTeX exited/); clean(root);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
