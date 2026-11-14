// Disposable hosted investigation. Every attempt is recorded; no retries.
import assert from 'node:assert/strict';
import { mkdtemp, cp, rename, mkdir, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { webkit } from '@playwright/test';
import { checkStatements as original } from './probe-original-statements.mjs';
import { checkStatements as revised } from './check-statements.mjs';
import { browserFailureState } from './browser-failure-state.mjs';
const root=await mkdtemp(path.join(tmpdir(),'zel-statement-focus-'));
const artifacts=path.resolve('test-results/statement-focus'); await mkdir(artifacts,{recursive:true});
const results=[];
try {
  const source=path.join(root,'source');await cp('scripts/fixtures/statements',source,{recursive:true});
  execFileSync(path.resolve('cli/target/debug/zel'),['build',source],{stdio:'inherit'});
  const moved=path.join(root,'Moved book 日本');await rename(path.join(source,'site'),moved);
  const browser=await webkit.launch();
  try {
    for(let round=0;round<10;round++) {
      for(const variant of round%2?['revised','original']:['original','revised']) {
        const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:1280,height:900}});
        await context.route(/^https?:/,route=>route.abort());
        const page=await context.newPage();
        const record={round,variant,passed:false};
        try {
          await (variant==='original'?original:revised)(page,pathToFileURL(moved+path.sep).href,artifacts,`${variant}-${round}`);
          record.passed=true;
        } catch(error) {
          record.error=String(error); record.state=await browserFailureState(page, {sampleFrames:false});
          try { await page.screenshot({path:path.join(artifacts,`${variant}-${round}-failure.png`),fullPage:true,timeout:3000}); } catch {}
        } finally { await context.close(); }
        results.push(record);
        console.log(JSON.stringify({round,variant,passed:record.passed,error:record.error}));
        await writeFile(path.join(artifacts,'results.json'),JSON.stringify({platform:process.platform,browser:'webkit',javaScriptEnabled:false,attempts:results},null,2));
      }
    }
  } finally { await browser.close(); }
  assert.equal(results.filter(r=>r.variant==='revised'&&!r.passed).length,0,'Revised readiness must pass every attempt');
} finally { await rm(root,{recursive:true,force:true}); }
