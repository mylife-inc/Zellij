import { chromium } from '@playwright/test';
import { readFileSync, writeFileSync } from 'node:fs';
import { execSync } from 'node:child_process';

const OUT = process.argv[2];
const themes = process.argv.slice(3);
const cfgPath = 'zellij.config.yaml';
const original = readFileSync(cfgPath, 'utf8');

try {
  for (const theme of themes) {
    writeFileSync(cfgPath, original.replace(/^theme: .*$/m, `theme: ${theme}`));
    execSync('npx next build', { stdio: 'ignore' });
    const server = execSync('npx next start --port 4319 & sleep 6; echo ok', { shell: '/bin/bash' });
    const b = await chromium.launch();
    const p = await b.newPage({ viewport: { width: 1360, height: 900 } });
    await p.goto('http://127.0.0.1:4319/', { waitUntil: 'networkidle' });
    await p.waitForTimeout(900);
    await p.screenshot({ path: `${OUT}/theme-${theme}.png` });
    await b.close();
    try { execSync("lsof -ti:4319 | xargs kill"); } catch {}
    console.log('captured', theme);
  }
} finally {
  writeFileSync(cfgPath, original);
}
