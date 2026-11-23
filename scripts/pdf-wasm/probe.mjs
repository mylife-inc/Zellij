import { Worker } from 'node:worker_threads';
import { mkdir, mkdtemp, lstat, writeFile, rename, rm } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { loadBundle, loadInputs, lock, sha256 } from './bundle.mjs';

async function runWorker(bundle, files, timeout, logBytes, formatEpoch, fontMetadata = false, signal) {
  if (!Number.isSafeInteger(timeout) || timeout < 1 || timeout > 120_000) throw new Error('Invalid PDF pass deadline');
  signal?.throwIfAborted();
  const worker = new Worker(new URL('./pass.mjs', import.meta.url), {
    workerData: { bundle, files, logBytes, formatEpoch, fontMetadata }, resourceLimits: { maxOldGenerationSizeMb: 128 },
    // Do not inherit startup hooks, inspector flags or host environment into a pass.
    env: {}, execArgv: [],
  });
  let timer, abort;
  try {
    return await new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error('PDF probe pass timed out')), timeout);
      abort = () => reject(new Error('PDF compilation cancelled'));
      signal?.addEventListener('abort', abort, { once: true });
      if (signal?.aborted) abort();
      worker.once('message', value => value.error ? reject(new Error(`${value.error}\n${value.console}`)) : resolve(value));
      worker.once('error', reject);
      worker.once('exit', code => reject(new Error(`PDF probe worker exited ${code} without a result`)));
    });
  } finally {
    clearTimeout(timer);
    if (abort) signal?.removeEventListener('abort', abort);
    await worker.terminate();
  }
}

export const runPass = (bundle, files, timeout = 120_000, logBytes, signal) => runWorker(bundle, files, timeout, logBytes, undefined, false, signal);
export const runFormatPass = (bundle, epoch, timeout = 120_000) => runWorker(bundle, new Map(), timeout, undefined, epoch);
export const runFontMetadataPass = (bundle, epoch, timeout = 120_000) => runWorker(bundle, new Map(), timeout, undefined, epoch, true);

export async function compile(bundleRoot, inputRoot, output, { format, metadata, signal, passTimeoutMs = 120_000, onPass = () => {} } = {}) {
  signal?.throwIfAborted();
  if (!Number.isSafeInteger(passTimeoutMs) || passTimeoutMs < 1 || passTimeoutMs > 120_000) throw new Error('Invalid PDF pass deadline');
  const bundle = loadBundle(bundleRoot, { format, metadata });
  const files = loadInputs(inputRoot);
  const destination = path.resolve(output);
  try { await lstat(destination); throw new Error('Probe output already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  await mkdir(path.dirname(destination), { recursive: true });
  const stage = await mkdtemp(path.join(path.dirname(destination), '.pdf-probe-'));
  const passes = [];
  try {
    let result;
    for (let pass = 1; pass <= 3; pass++) {
      signal?.throwIfAborted();
      onPass(pass);
      result = await runPass(bundle, files, passTimeoutMs, undefined, signal);
      for (const [name, bytes] of result.aux) files.set(name, bytes);
      passes.push({ pass, heapBytes: result.heapBytes, pdfBytes: result.pdf.length, misses: result.misses });
      await writeFile(path.join(stage, `pass-${pass}.log`), result.log);
    }
    const text = new TextDecoder().decode(result.log);
    for (const marker of ['Missing character:', 'Undefined control sequence', 'undefined references', 'undefined citations',
      'has been referenced but does not exist', 'Rerun to get cross-references right', 'Overfull \\hbox', 'Overfull \\vbox']) {
      if (text.includes(marker)) throw new Error(`PDF probe typesetting reported ${marker}`);
    }
    const pdf = Buffer.from(result.pdf);
    if (!pdf.subarray(0, 5).equals(Buffer.from('%PDF-')) || !pdf.subarray(-1024).includes(Buffer.from('%%EOF'))) {
      throw new Error('Invalid PDF probe artifact');
    }
    await writeFile(path.join(stage, 'book.pdf'), pdf);
    await writeFile(path.join(stage, 'book.log'), result.log);
    const evidence = { schemaVersion: 1, experimental: true, engineRelease: lock.engineRelease,
      resourceLockSha256: sha256(JSON.stringify(lock)),
      formatSha256: sha256(bundle.format),
      fontMetadataSha256: sha256(bundle.resources.get('51/luaotfload-names.lua')),
      fontMapSha256: sha256(bundle.resources.get('11/pdftex.map')),
      pdfSha256: sha256(pdf), passes };
    await writeFile(path.join(stage, 'probe.json'), JSON.stringify(evidence, null, 2) + '\n');
    signal?.throwIfAborted();
    try { await lstat(destination); throw new Error('Probe output appeared during compilation'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    signal?.throwIfAborted();
    await rename(stage, destination);
    return evidence;
  } finally { await rm(stage, { recursive: true, force: true }); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (process.argv.length !== 5) throw new Error('Usage: node scripts/pdf-wasm/probe.mjs BUNDLE INPUTS NEW_OUTPUT');
  const result = await compile(...process.argv.slice(2));
  console.log(`Experimental offline LuaHBTeX: ${result.passes.length} passes; PDF ${result.pdfSha256}`);
}
