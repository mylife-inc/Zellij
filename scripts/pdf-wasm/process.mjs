// Process contract for the optional PDF runner. Not a published CLI backend.
// The acceptance adapter owns staging and reaps the child; a Rust caller must
// retain those responsibilities, including on forced termination.
import { spawn } from 'node:child_process';
import { lstat, mkdir, mkdtemp, readFile, writeFile, rename, rm } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import { compile } from './probe.mjs';
import { sha256 } from './bundle.mjs';

function deadline(value, maximum) {
  if (!Number.isSafeInteger(value) || value < 1 || value > maximum) throw new Error('Invalid PDF process deadline');
  return value;
}

async function absent(output) {
  try { await lstat(output); throw new Error('PDF process output already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
}

async function regular(file, maximum) {
  const info = await lstat(file);
  if (!info.isFile() || info.isSymbolicLink() || info.size > maximum) throw new Error('Invalid PDF process artifact');
  const bytes = await readFile(file);
  if (bytes.length !== info.size) throw new Error('PDF process artifact changed');
  return bytes;
}

export function runnerEnvironment() {
  // Private Node needs OS bootstrap paths on Windows. No PATH, NODE_OPTIONS,
  // NODE_PATH, coverage hooks, credentials or user-home configuration is passed.
  // Node spawn copies NODE_V8_COVERAGE even into a supplied allowlist unless
  // it is explicitly present. An empty value disables that automatic hook.
  const env = { LANG: 'C.UTF-8', TZ: 'UTC', NODE_V8_COVERAGE: '' };
  for (const name of ['SystemRoot', 'WINDIR', 'TEMP', 'TMP', 'TMPDIR']) {
    if (process.env[name] !== undefined) env[name] = process.env[name];
  }
  return env;
}

export async function compileProcess(bundle, input, output, {
  signal, timeoutMs = 600_000, passTimeoutMs = 120_000, onEvent = () => {},
} = {}) {
  deadline(timeoutMs, 600_000);
  deadline(passTimeoutMs, 120_000);
  signal?.throwIfAborted();
  const destination = path.resolve(output);
  await absent(destination);
  await mkdir(path.dirname(destination), { recursive: true });
  const stage = await mkdtemp(path.join(path.dirname(destination), '.pdf-process-'));
  let child, timer, killTimer, abort, exited;
  let closed = false;
  try {
    const result = path.join(stage, 'result');
    // V8's process flag overrides Worker resourceLimits; keep both at 128 MiB.
    // Node 24's Wasm code-GC race can abort between disposable passes (#66366).
    // Retain compiled code only until this bounded, isolated process exits.
    child = spawn(process.execPath, ['--max-old-space-size=128', '--no-wasm-code-gc', fileURLToPath(import.meta.url), '--run',
      path.resolve(bundle), path.resolve(input), result, String(timeoutMs), String(passTimeoutMs)], {
      cwd: stage, env: runnerEnvironment(), stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true,
    });
    let failure, stderr = '', pending = '', received = 0, ready = false, complete = false;
    const passes = [];
    function fail(error) {
      if (failure) return;
      failure = error;
      // EOF cancels cooperatively on every OS; kill is the bounded fallback.
      child.stdin.end();
      killTimer = setTimeout(() => child.kill('SIGKILL'), 1_000);
    }
    exited = new Promise(resolve => {
      child.once('error', error => fail(new Error(`PDF process could not start: ${error.message}`)));
      child.once('close', (code, exitSignal) => { closed = true; resolve({ code, exitSignal }); });
    });
    child.stdin.on('error', error => { if (error.code !== 'EPIPE' && error.code !== 'ERR_STREAM_DESTROYED') fail(error); });
    child.stderr.on('data', bytes => {
      if (Buffer.byteLength(stderr) + bytes.length > 64 * 1024) fail(new Error('PDF process diagnostic budget exceeded'));
      else stderr += bytes.toString();
    });
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', text => {
      received += Buffer.byteLength(text);
      if (received > 16 * 1024) { fail(new Error('PDF process protocol budget exceeded')); return; }
      pending += text;
      try {
        for (let end; (end = pending.indexOf('\n')) >= 0;) {
          const event = JSON.parse(pending.slice(0, end));
          pending = pending.slice(end + 1);
          if (event.pid !== child.pid || complete) throw new Error('Invalid PDF process event');
          if (event.event === 'ready' && !ready && passes.length === 0) ready = true;
          else if (ready && event.event === 'pass' && event.pass === passes.length + 1 && event.pass <= 3) passes.push(event.pass);
          else if (ready && event.event === 'complete' && passes.length === 3) complete = true;
          else throw new Error('Invalid PDF process event sequence');
          onEvent(event);
        }
      } catch (error) { fail(error); }
    });
    abort = () => fail(new Error('PDF process cancelled'));
    signal?.addEventListener('abort', abort, { once: true });
    if (signal?.aborted) abort();
    timer = setTimeout(() => fail(new Error('PDF process deadline exceeded')), timeoutMs);
    const status = await exited;
    clearTimeout(timer);
    clearTimeout(killTimer);
    if (failure) throw failure;
    signal?.throwIfAborted();
    if (status.code !== 0 || !complete || pending) {
      throw new Error(`PDF process failed (${status.exitSignal || status.code}): ${stderr.slice(-16 * 1024)}`);
    }
    const evidence = JSON.parse(await regular(path.join(result, 'probe.json'), 8 * 1024 * 1024));
    const pdf = await regular(path.join(result, 'book.pdf'), 128 * 1024 * 1024);
    await regular(path.join(result, 'book.log'), 8 * 1024 * 1024);
    if (evidence.pdfSha256 !== sha256(pdf) || evidence.passes?.length !== 3
        || !evidence.passes.every((p, i) => p.pass === i + 1)
        || !pdf.subarray(0, 5).equals(Buffer.from('%PDF-')) || !pdf.subarray(-1024).includes(Buffer.from('%%EOF'))) {
      throw new Error('Invalid PDF process result');
    }
    evidence.execution = { kind: 'separate-process', node: process.version };
    await writeFile(path.join(result, 'probe.json'), JSON.stringify(evidence, null, 2) + '\n');
    signal?.throwIfAborted();
    await absent(destination);
    signal?.throwIfAborted();
    await rename(result, destination);
    return evidence;
  } finally {
    clearTimeout(timer);
    clearTimeout(killTimer);
    if (abort) signal?.removeEventListener('abort', abort);
    // No live child can outlast removal of its private output directory.
    if (child && !closed) {
      child.kill('SIGKILL');
      await exited;
    }
    await rm(stage, { recursive: true, force: true });
  }
}

async function run(args) {
  if (args.length !== 6 || args[0] !== '--run') throw new Error('Expected --run BUNDLE INPUT NEW_OUTPUT TOTAL_MS PASS_MS');
  const [, bundle, input, output, total, pass] = args;
  const timeoutMs = deadline(Number(total), 600_000);
  const passTimeoutMs = deadline(Number(pass), 120_000);
  const control = new AbortController();
  const abort = () => control.abort(new Error('PDF process parent disconnected or cancelled'));
  const timer = setTimeout(() => control.abort(new Error('PDF process deadline exceeded')), timeoutMs);
  const event = (event, fields = {}) => process.stdout.write(JSON.stringify({ event, pid: process.pid, ...fields }) + '\n');
  process.stdin.on('end', abort).on('error', abort).on('data', abort).resume();
  process.on('SIGTERM', abort).on('SIGINT', abort);
  try {
    event('ready', { node: process.version });
    await compile(bundle, input, output, { signal: control.signal, passTimeoutMs, onPass: pass => event('pass', { pass }) });
    event('complete');
  } finally {
    clearTimeout(timer);
    process.off('SIGTERM', abort).off('SIGINT', abort);
    process.stdin.removeListener('end', abort).removeListener('error', abort).removeListener('data', abort);
    process.stdin.destroy();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { await run(process.argv.slice(2)); }
  catch (error) { process.stderr.write(String(error.message).slice(-16 * 1024) + '\n'); process.exitCode = 1; }
}
