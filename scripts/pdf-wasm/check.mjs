// Repository-owned manuscripts only. Output is evidence, never a CLI release.
import { execFileSync } from 'node:child_process';
import { cpSync, mkdtempSync, rmSync, writeFileSync, readFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { compile } from './probe.mjs';
import { compileProcess } from './process.mjs';
import { writeLanguageBook } from '../fixtures/book-languages.mjs';
import { printStyles, withPrintStyle } from '../fixtures/print-styles.mjs';
import { compare } from './compare.mjs';
import { writeLongBook } from '../fixtures/long-book.mjs';
import { recipe } from './format.mjs';
import { sha256 } from './bundle.mjs';
import { loadFontMetadata } from './fonts.mjs';

const bundle = process.env.ZEL_PDF_PROBE_BUNDLE;
if (!bundle) throw new Error('Set ZEL_PDF_PROBE_BUNDLE');
const format = process.env.ZEL_PDF_REBUILT_FORMAT ? new Uint8Array(readFileSync(process.env.ZEL_PDF_REBUILT_FORMAT)) : undefined;
if (format && (format.length !== recipe.output.bytes || sha256(format) !== recipe.output.sha256)) {
  throw new Error('Rebuilt format does not match the reviewed recipe output');
}
const metadata = process.env.ZEL_PDF_FONT_METADATA ? loadFontMetadata(process.env.ZEL_PDF_FONT_METADATA) : undefined;
const extension = process.platform === 'win32' ? '.exe' : '';
const exporter = path.resolve(`cli/target/debug/examples/pdf_probe_inputs${extension}`);
const zel = path.resolve(`cli/target/debug/zel${extension}`);
const root = mkdtempSync(path.join(tmpdir(), 'zel-pdf-wasm-'));
const artifacts = path.resolve('test-results/pdf-wasm');
mkdirSync(artifacts, { recursive: true });
const results = [];
async function check(name, manuscript) {
  const input = path.join(root, name + '-input');
  execFileSync(exporter, [manuscript, input], { stdio: 'pipe', timeout: 60_000 });
  // Keep optional legacy in-memory substitutions for maintainer comparisons.
  // The default and compact-package manuscript matrix use a separate process.
  const run = format || metadata ? compile : compileProcess;
  const result = await run(bundle, input, path.join(artifacts, name), { format, metadata });
  if (process.env.ZEL_PDF_PROBE_BASELINE === '1') {
    const baseline = path.join(artifacts, name + '-native');
    execFileSync(zel, ['pdf', manuscript, '--out', baseline], { stdio: 'pipe', timeout: 600_000 });
    result.comparison = compare(path.join(baseline, 'book.pdf'), path.join(artifacts, name, 'book.pdf'));
  }
  results.push({ name, ...result });
  writeFileSync(path.join(artifacts, 'acceptance.json'), JSON.stringify({
    experimental: true, platform: process.platform, arch: process.arch, node: process.version, results,
  }, null, 2) + '\n');
  console.log(`${name}: three offline passes, verified PDF/log`);
}
try {
  for (const name of ['pdf-book', 'equations', 'statements', 'bibliography', 'terms']) {
    const manuscript = path.join(root, name);
    cpSync(`scripts/fixtures/${name}`, manuscript, { recursive: true });
    await check(name, manuscript);
  }
  for (const [size, paper] of [[10, 'a4'], [12, 'letter']]) {
    const name = `book-${size}pt-${paper}`;
    const manuscript = path.join(root, name);
    cpSync('scripts/fixtures/pdf-book', manuscript, { recursive: true });
    const config = path.join(manuscript, 'zel.yaml');
    writeFileSync(config, readFileSync(config, 'utf8') + `\npdf: {enabled: true, font-size: ${size}, paper: ${paper}}\n`);
    await check(name, manuscript);
  }
  for (const edition of ['technical', 'study']) {
    const manuscript = path.join(root, 'book-' + edition);
    execFileSync(zel, ['create', manuscript, '-t', 'book-' + edition, '--offline'], { stdio: 'pipe', timeout: 60_000 });
    await check('book-' + edition, manuscript);
    const config = path.join(manuscript, 'zel.yaml');
    writeFileSync(config, withPrintStyle(readFileSync(config, 'utf8'), printStyles[edition]));
    await check('book-' + edition + '-custom', manuscript);
  }
  for (const [language, edition] of [['ar', 'classic'], ['ar', 'technical'], ['ar', 'study'], ['fr', 'study'], ['de', 'classic'], ['es', 'classic']]) {
    const name = language + '-' + edition;
    const manuscript = path.join(root, name);
    writeLanguageBook(manuscript, language, edition);
    await check(name, manuscript);
  }
  const defaultFonts = path.join(root, 'ar-default-fonts');
  const config = writeLanguageBook(defaultFonts, 'ar', 'classic');
  delete config.pdf.style;
  writeFileSync(path.join(defaultFonts, 'zel.yaml'), JSON.stringify(config, null, 2));
  await check('ar-default-fonts', defaultFonts);
  const longBook = path.join(root, 'long-book');
  writeLongBook(longBook);
  await check('long-book', longBook);
} finally { rmSync(root, { recursive: true, force: true }); }
