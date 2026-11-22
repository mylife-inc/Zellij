import { Worker } from 'node:worker_threads';
import { mkdir, mkdtemp, lstat, writeFile, rename, rm } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { loadBundle, loadInputs, lock, sha256 } from './bundle.mjs';

export async function runPass(bundle, files, timeout = 120_000, logBytes) {
  const worker = new Worker(new URL('./pass.mjs', import.meta.url), {
    workerData: { bundle, files, logBytes }, resourceLimits: { maxOldGenerationSizeMb: 128 },
  });
  let timer;
  try {
    return await new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error('PDF probe pass timed out')), timeout);
      worker.once('message', value => value.error ? reject(new Error(`${value.error}\n${value.console}`)) : resolve(value));
      worker.once('error', reject);
      worker.once('exit', code => reject(new Error(`PDF probe worker exited ${code} without a result`)));
    });
  } finally { clearTimeout(timer); await worker.terminate(); }
}

export async function compile(bundleRoot, inputRoot, output) {
  const bundle = loadBundle(bundleRoot);
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
      result = await runPass(bundle, files);
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
      pdfSha256: sha256(pdf), passes };
    await writeFile(path.join(stage, 'probe.json'), JSON.stringify(evidence, null, 2) + '\n');
    await rename(stage, destination);
    return evidence;
  } finally { await rm(stage, { recursive: true, force: true }); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (process.argv.length !== 5) throw new Error('Usage: node scripts/pdf-wasm/probe.mjs BUNDLE INPUTS NEW_OUTPUT');
  const result = await compile(...process.argv.slice(2));
  console.log(`Experimental offline LuaHBTeX: ${result.passes.length} passes; PDF ${result.pdfSha256}`);
}
