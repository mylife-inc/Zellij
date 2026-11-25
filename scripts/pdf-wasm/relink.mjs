// Maintainer verification of the deliberately modified compiler, never a CLI backend.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Worker } from 'node:worker_threads';
import { loadBundle, sha256 } from './bundle.mjs';

assert.equal(process.argv.length, 5, 'Usage: node relink.mjs ORIGINAL_RESOURCES MODIFIED_COMPILER NEW_OUTPUT');
const [resources, compiler, output] = process.argv.slice(2).map(p => path.resolve(p));
const evidence = JSON.parse(await readFile(new URL('./relink-evidence.json', import.meta.url)));
const bundle = loadBundle(resources);
assert.equal(sha256(bundle.code), evidence.execution.originalEngine.js);
assert.equal(sha256(bundle.wasm), evidence.execution.originalEngine.wasm);
const code = await readFile(path.join(compiler, 'wasmtex-luatex.js'), 'utf8');
const wasm = await readFile(path.join(compiler, 'wasmtex-luatex.wasm'));
assert.equal(sha256(code), evidence.execution.modifiedEngine.js.sha256);
assert.equal(sha256(wasm), evidence.execution.modifiedEngine.wasm.sha256);
function imports(text) {
  const matches = [...text.matchAll(/var wasmImports=\{[^}]+\}/g)];
  assert.equal(matches.length, 1);
  return matches[0][0];
}
assert.equal(imports(code), imports(bundle.code), 'Changed imports require a new boundary review');
bundle.code = code;
bundle.wasm = new Uint8Array(wasm);
const adapter = await readFile(new URL('./pass.mjs', import.meta.url), 'utf8');
assert.equal(adapter.split('  await initialized;').length, 2);
assert.equal(adapter.split('parentPort.postMessage({ pdf:').length, 2);
const workerCode = adapter.replace('  await initialized;', `  await initialized;
  const relinkProof = context.Module._zellijRelinkProof();
  if (relinkProof !== 7) throw new Error('Modified library proof failed: ' + relinkProof);`)
  .replace('parentPort.postMessage({ pdf:', 'parentPort.postMessage({ relinkProof, pdf:');
await mkdir(output); // Deliberately reject existing outputs and retain failed evidence.
const workerPath = path.join(output, 'verify-pass.mjs');
await writeFile(workerPath, workerCode, { flag: 'wx' });
const tex = String.raw`\documentclass{book}
\usepackage{fontspec}
\setmainfont{Latin Modern Roman}
\usepackage{unicode-math}
\setmathfont{latinmodern-math.otf}
\usepackage{hyperref}
\begin{document}
\tableofcontents
\chapter{Modified-library relinking}
This repository-owned manuscript tests a compiler rebuilt with modified kpathsea,
Graphite2 and zziplib sources. Equation \(x^2+y^2=z^2\) and a
\href{https://example.com}{working link} exercise the print path.
\section{Preserved navigation}\label{sec:proof}
The libraries keep their public interfaces. See section~\ref{sec:proof}.
\end{document}
`;
assert.equal(sha256(tex), evidence.execution.manuscriptSha256);
await writeFile(path.join(output, 'book.tex'), tex);
const files = new Map([['book.tex', new TextEncoder().encode(tex)]]);
let result;
const passes = [];
for (let pass = 1; pass <= 3; pass++) {
  const worker = new Worker(pathToFileURL(workerPath), {
    workerData: { bundle, files }, resourceLimits: { maxOldGenerationSizeMb: 128 }, env: {}, execArgv: [],
  });
  let timer;
  try {
    result = await new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error('Relink verification pass timed out')), 120_000);
      worker.once('message', r => r.error ? reject(new Error(`${r.error}\n${r.console}`)) : resolve(r));
      worker.once('error', reject);
      worker.once('exit', c => reject(new Error(`Relink worker exited ${c} without a result`)));
    });
  } finally { clearTimeout(timer); await worker.terminate(); }
  assert.equal(result.relinkProof, 7);
  for (const [name, bytes] of result.aux) files.set(name, bytes);
  await writeFile(path.join(output, `pass-${pass}.log`), result.log);
  passes.push({ pass, relinkProof: 7, pdfBytes: result.pdf.length, heapBytes: result.heapBytes });
}
const log = Buffer.from(result.log).toString();
for (const marker of ['Missing character:', 'Undefined control sequence', 'undefined references',
  'undefined citations', 'has been referenced but does not exist', 'Rerun to get cross-references right',
  'Overfull \\hbox', 'Overfull \\vbox']) assert(!log.includes(marker), marker);
const pdf = Buffer.from(result.pdf);
assert(pdf.subarray(0, 5).equals(Buffer.from('%PDF-')) && pdf.subarray(-1024).includes(Buffer.from('%%EOF')));
await writeFile(path.join(output, 'book.pdf'), pdf);
await writeFile(path.join(output, 'verification.json'), JSON.stringify({ schemaVersion: 1,
  qualificationOnly: true, importMapUnchanged: true, pdfSha256: sha256(pdf), passes,
  engine: evidence.execution.modifiedEngine, adapterSha256: sha256(adapter), nodeVersion: process.version,
  verifierSha256: sha256(await readFile(new URL(import.meta.url))) }, null, 2) + '\n');
console.log('All three modified library APIs executed on three successful book typesetting passes.');
