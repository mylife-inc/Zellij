// The pinned original remains unchanged; exercise its text and link interfaces.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { loadBundle } from './bundle.mjs';
import { runPass } from './probe.mjs';

const bundle = loadBundle(process.env.ZEL_PDF_PROBE_BUNDLE);
const tex = body => new Map([['book.tex', new TextEncoder().encode(String.raw`
\documentclass{article}
\usepackage{fontspec,unicode-math,hyperref}
\setmainfont{Latin Modern Roman}\setmonofont{Latin Modern Mono}
\begin{document}${body}\end{document}`)]]);

test('unchanged url 3.3 supports punctuation, paths, robust definitions and hyperlinks', { timeout: 60_000 }, async () => {
  const result = await runPass(bundle, tex(String.raw`
\url{https://example.test/a_b?q=one&next=two#section}
\url{https://example.test/a%20b}
\url{mailto:reader@example.test}
\path{C:\Documents\My_Book\chapter.md}
\urldef\example\url{https://example.test/a_b?q=one&next=two}
\section{A robust URL: \example}
\href{https://example.test/a_b?q=one&next=two#section}{A descriptive link}
\urlstyle{same}\url{https://example.test/same}
`));
  const log = new TextDecoder().decode(result.log);
  assert.match(log, /Package: url 2006\/04\/12.*3\.3/);
  for (const marker of ['Undefined control sequence', 'Missing character:', 'Fatal error']) assert(!log.includes(marker), marker);
  assert(Buffer.from(result.pdf).subarray(0, 5).equals(Buffer.from('%PDF-')));
});

test('URLs in math mode still fail instead of publishing a partial PDF', { timeout: 60_000 }, async () => {
  await assert.rejects(runPass(bundle, tex(String.raw`$\url{https://example.test/math}$`)), /exit|failed|Undefined control/);
});
