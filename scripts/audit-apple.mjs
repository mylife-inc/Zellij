import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { chromium, expect } from '@playwright/test';
await mkdir('apple-research', { recursive: true });
const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1440, height: 1100 }, colorScheme: 'light' });
const page = await context.newPage();
const report = { url: 'https://www.apple.com/iphone/', observations: [] };
try {
  const response = await page.goto(report.url, { waitUntil: 'domcontentloaded', timeout: 45000 });
  assert.equal(response.status(), 200);
  const section = page.locator('.section-augment');
  await section.scrollIntoViewIfNeeded();
  const mac = page.getByRole('button', { name: 'iPhone and Mac', exact: true });
  const watch = page.getByRole('button', { name: /iPhone and Apple\s+Watch/ });
  await expect(mac).toHaveAttribute('aria-expanded', 'true');
  await page.waitForTimeout(1000);
  await section.screenshot({ path: 'apple-research/media-accordion-before.png' });
  await watch.click();
  await expect(watch).toHaveAttribute('aria-expanded', 'true');
  await expect(mac).toHaveAttribute('aria-expanded', 'false');
  await page.waitForTimeout(1200);
  await section.screenshot({ path: 'apple-research/media-accordion-after.png' });
  report.observations.push('Selecting the Watch item opens it and closes the Mac item. Before/after section screenshots captured.');
  await page.getByRole('button', { name: 'Read more, Apple Trade In', exact: true }).first().click();
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'apple-research/feature-detail-open.png' });
  report.dialogs = await page.locator('[role="dialog"], dialog').evaluateAll(nodes=>nodes.map(n=>({role:n.getAttribute('role'),label:n.getAttribute('aria-label'),visible:n.getClientRects().length>0})));
  report.observations.push('Activated a feature card detail control and captured the resulting view.');
} catch (error) { report.error = error.message; process.exitCode = 1; }
finally { await writeFile('apple-research/interaction-report.json', JSON.stringify(report,null,2)); await browser.close(); }
console.log(JSON.stringify(report));
