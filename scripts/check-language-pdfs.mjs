// This suite deliberately exercises the optional container compiler.
process.env.ZEL_PDF_BACKEND = 'container';
// Real LuaLaTeX only on hosted runners; retain PDFs/text/fonts/pages for review.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync, cpSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { languages, writeLanguageBook, normalizedPrintText, requiredPrintTerms } from './fixtures/book-languages.mjs';
const binary = path.resolve(process.argv[2] || 'cli/target/debug/zel');
const root = mkdtempSync(path.join(tmpdir(),'zel-language-'));
const artifacts = path.resolve('test-results/pdf');
try {
  for(const [language, edition, defaultFonts] of [['ar','classic'],['ar','technical'],['ar','study'],['ar','classic',true],['fr','study'],['de','classic'],['es','classic']]) {
    const key = `${language}-${edition}${defaultFonts ? '-default' : ''}`, source = path.join(root,key), text = languages[language];
    const config = writeLanguageBook(source,language,edition);
    if(defaultFonts) { delete config.pdf.style; writeFileSync(path.join(source,'zel.yaml'),JSON.stringify(config,null,2)+'\n'); }
    try { execFileSync(binary,['build',source],{stdio:'pipe',timeout:600000}); }
    catch(e) { console.error(e.stderr?.toString()); throw e; }
    const printed=path.join(source,'site/_zellij/pdf'), out=path.join(artifacts,key);
    mkdirSync(out,{recursive:true});cpSync(printed,out,{recursive:true});
    const pdf=path.join(out,'book.pdf');execFileSync('qpdf',['--check',pdf]);
    const extracted=execFileSync('pdftotext',['-layout',pdf,'-'],{encoding:'utf8'});
    const info=execFileSync('pdfinfo',[pdf],{encoding:'utf8'}), fonts=execFileSync('pdffonts',[pdf],{encoding:'utf8'});
    writeFileSync(path.join(out,'text.txt'),extracted);writeFileSync(path.join(out,'pdfinfo.txt'),info);writeFileSync(path.join(out,'fonts.txt'),fonts);
    assert(!extracted.includes('??'));
    for(const line of fonts.trim().split('\n').slice(2)) assert.match(line,/\byes\s+yes\s+yes\b/,'Fonts embedded, subset and Unicode mapped');
    const normalized=normalizedPrintText(extracted);
    for(const term of requiredPrintTerms(language)) assert(normalized.includes(normalizedPrintText(term)),`${key}: missing extracted ${term}\n${extracted}`);
    if(language==='ar') assert.match(fonts,/Amiri/);
    if(language==='fr') assert.match(fonts,/Libertinus/);
    assert.match(extracted,/Zellij/);assert.match(extracted,/Shebka/);
    for(let n=1;n<=Number(info.match(/Pages:\s+(\d+)/)[1]);n++) execFileSync('pdftoppm',['-f',String(n),'-singlefile','-scale-to','1300','-png',pdf,path.join(out,`page-${n}`)]);
    console.log(`${key}: typeset, embedded fonts and page captures verified`);
  }
} finally { rmSync(root,{recursive:true,force:true}); }
