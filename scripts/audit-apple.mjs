import { mkdir, writeFile } from 'node:fs/promises';
import { chromium } from '@playwright/test';
const pages = [
  ['home', 'https://www.apple.com/'],
  ['iphone', 'https://www.apple.com/iphone/'],
  ['iphone-pro', 'https://www.apple.com/iphone-18-pro/'],
  ['macbook', 'https://www.apple.com/macbook-pro/'],
];
await mkdir('apple-research', { recursive: true });
const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, colorScheme: 'light' });
const report = [];
try {
  for (const [name, url] of pages) {
    const page = await context.newPage();
    page.setDefaultTimeout(15000);
    const item = { name, url, snapshots: [] };
    report.push(item);
    try {
      const response = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 45000 });
      item.status = response?.status();
      item.final_url = page.url();
      await page.waitForTimeout(1800);
      item.title = await page.title();
      item.sections = await page.locator('main section, main [class~="section"]').evaluateAll(nodes => nodes.slice(0, 24).map(node => ({ id: node.id, classes: node.className, title: node.querySelector('h2,h3')?.textContent?.trim().slice(0, 90), y: Math.round(node.getBoundingClientRect().top + scrollY), height: Math.round(node.getBoundingClientRect().height) })));
      item.controls = await page.locator('main button, main select, main [role="tab"]').evaluateAll(nodes => nodes.slice(0, 40).map(node => ({ tag: node.tagName, role: node.getAttribute('role'), label: (node.getAttribute('aria-label') || node.textContent || '').trim().slice(0, 80), classes: node.className })));
      const positions = [0, ...item.sections.filter(s => s.height > 400 && s.y > 650).slice(0, 5).map(s => s.y)];
      for (const [index, y] of positions.entries()) {
        await page.evaluate(y => scrollTo(0, y), y);
        await page.waitForTimeout(1300);
        const screenshot = `${name}-${index}.png`;
        await page.screenshot({ path: `apple-research/${screenshot}` });
        item.snapshots.push({ screenshot, y });
      }
      if (name === 'iphone-pro') {
        await page.setViewportSize({ width: 390, height: 844 });
        await page.evaluate(() => scrollTo(0, 0));
        await page.waitForTimeout(1200);
        await page.screenshot({ path: 'apple-research/iphone-pro-mobile.png' });
      }
    } catch (error) { item.error = error.message; }
    await page.close();
    await writeFile('apple-research/report.json', JSON.stringify(report, null, 2));
  }
} finally { await browser.close(); }
console.log(JSON.stringify(report.map(({ name, status, title, error, snapshots }) => ({ name, status, title, error, screenshots: snapshots.length }))));
