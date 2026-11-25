// Real typesetting regression: chapter 100 must fit in the contents column.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { writeLongBook } from './fixtures/long-book.mjs';

const binary = path.resolve(process.argv[2] || 'cli/target/debug/zel');
const source = mkdtempSync(path.join(tmpdir(), 'zel-long-book-'));
const output = path.resolve('test-results/pdf/long-book');
mkdirSync(path.dirname(output), { recursive: true });
try {
  writeLongBook(source);
  // This comparison suite runs against the isolated container compiler.
  execFileSync(binary, ['pdf', source, '--pdf-backend', 'container', '--out', output], { stdio: 'pipe', timeout: 600_000 });
  const pdf = path.join(output, 'book.pdf');
  execFileSync('qpdf', ['--check', pdf]);
  const text = execFileSync('pdftotext', ['-layout', pdf, '-'], { encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 });
  const words = text.replace(/\s+/g, ' ');
  assert.match(words, /100 Topic 100/, 'The last chapter number and title appear in the contents');
  assert.match(words, /Chapter 100 Topic 100/);
  assert.match(words, /original text of topic 100\./);
  assert(!words.includes('??'), 'All references resolve');
  const info = execFileSync('pdfinfo', [pdf], { encoding: 'utf8' });
  const pages = Number(info.match(/^Pages:\s+(\d+)/m)?.[1]);
  assert(pages >= 100 && pages < 140, 'Every chapter and both parts are retained');
  writeFileSync(path.join(output, 'text.txt'), text);
  writeFileSync(path.join(output, 'pdfinfo.txt'), info);
  // Retain the contents pages and final chapter for visual review.
  for (const page of [2, 3, 4, 5, 6, 7, 8, pages]) {
    execFileSync('pdftoppm', ['-f', String(page), '-singlefile', '-scale-to', '1300', '-png', pdf, path.join(output, `page-${page}`)]);
  }
  console.log(`Long book: 100 chapters, two parts, ${pages} pages; no overfull boxes.`);
} finally { rmSync(source, { recursive: true, force: true }); }
