// Optional maintainer comparison; Poppler/qpdf are not end-user requirements.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { sha256 } from './bundle.mjs';

export function compare(native, wasm) {
  const command = (tool, args, encoding) => execFileSync(tool, args, { encoding, maxBuffer: 24 * 1024 * 1024, timeout: 60_000 });
  for (const pdf of [native, wasm]) command('qpdf', ['--check', pdf]);
  const texts = [native, wasm].map(pdf => command('pdftotext', ['-layout', pdf, '-'], 'utf8').normalize('NFKC'));
  assert.equal(texts[0], texts[1], 'NFKC-normalized extracted text and layout');
  const info = [native, wasm].map(pdf => command('pdfinfo', [pdf], 'utf8'));
  const pages = info.map(text => Number(text.match(/^Pages:\s+(\d+)/m)?.[1]));
  assert(pages[0] > 0 && pages[0] <= 100);
  assert.equal(pages[0], pages[1], 'Page count');
  assert.equal(info[0].match(/^Page size:.+$/m)?.[0], info[1].match(/^Page size:.+$/m)?.[0], 'Page dimensions');
  const pageSha256 = [];
  for (let page = 1; page <= pages[0]; page++) {
    const raster = [native, wasm].map(pdf => command('pdftoppm', ['-f', String(page), '-singlefile', '-scale-to', '1300', pdf]));
    assert(raster[0].equals(raster[1]), `Page ${page}: 1300-pixel raster differs`);
    pageSha256.push(sha256(raster[0]));
  }
  return { pages: pages[0], normalizedTextSha256: sha256(texts[0]), identicalRasterSha256: pageSha256 };
}
