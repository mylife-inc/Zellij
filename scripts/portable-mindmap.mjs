// Native mindmaps must survive relocation, offline browsing and disabled JS.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect } from '@playwright/test';

export async function checkMindmaps({ binary, root, env, engine, browser }) {
  const source = path.join(root, `Mindmaps 日本 ${engine}`);
  const moved = `${source}-moved café`;
  await mkdir(path.join(source, 'guide'), { recursive: true });
  await writeFile(path.join(source, 'index.md'), '# Diagrams\n\n[Read maps](guide/maps.md#hierarchy)\n');
  await writeFile(path.join(source, 'guide/maps.md'), `# Maps

## Hierarchy

\`\`\`mindmap
Handbook:
  Install:
    - Linux
    - macOS
    - Windows
  Rendering:
    Native: [Nebula, 日本 café 👩‍💻]
    Browser: [Search, Themes]
  Publish: Static HTML
\`\`\`

\`\`\`mindmap
Solo:
\`\`\`

\`\`\`mindmap
'<script>alert(1)</script>':
  - '</text><image href="https://example.com/tracker" onload="alert(1)"/>'
\`\`\`

\`\`\`mermaid
flowchart LR
 A --> B
\`\`\`

[Home](../index.md)
`);
  const report = JSON.parse(execFileSync(binary, ['check', source, '--strict', '--json'], { env, encoding: 'utf8' }));
  assert.equal(report.ok, true);
  execFileSync(binary, ['build', source], { env });
  await rename(`${source}-site`, moved);
  assert(!(await readFile(path.join(moved, 'index.html'), 'utf8')).includes('mindmap.css'));
  for (const javaScriptEnabled of [true, false]) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, javaScriptEnabled, offline: engine === 'chromium', serviceWorkers: 'block' });
    const requests = [];
    const errors = [];
    context.on('request', request => requests.push(request.url()));
    await context.route(/^https?:/, route => route.abort());
    try {
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      page.on('dialog', dialog => { errors.push(dialog.message()); dialog.dismiss(); });
      const nested = pathToFileURL(path.join(moved, 'guide/maps.html')).href;
      await page.goto(nested);
      const diagram = page.getByRole('img', { name: 'Mindmap: Handbook', exact: true });
      await expect(diagram).toBeVisible();
      await expect(page.getByRole('img', { name: 'Mindmap: Solo', exact: true })).toBeVisible();
      await expect(page.locator('.mindmap-svg')).toHaveCount(3);
      await expect(page.locator('.code-block[data-language="mermaid"]')).toContainText('flowchart LR');
      await expect(page.locator('article image, article script, article style')).toHaveCount(0);
      await page.evaluate(() => document.fonts.ready);
      // Actual glyph and node bounds catch clipping, overlap and broken SVG layout.
      assert.deepEqual(await page.locator('.mindmap-svg').evaluateAll(svgs => svgs.flatMap(svg => {
        const faults = [];
        const boxes = [];
        for (const node of svg.querySelectorAll('.mindmap-node')) {
          const rect = node.querySelector('rect').getBBox();
          const text = node.querySelector('text').getBBox();
          if (text.width <= 0 || text.x < rect.x || text.x + text.width > rect.x + rect.width || text.y < rect.y || text.y + text.height > rect.y + rect.height) faults.push(`clipped ${node.textContent}`);
          const view = svg.viewBox.baseVal;
          if (rect.x < 0 || rect.y < 0 || rect.x + rect.width > view.width || rect.y + rect.height > view.height) faults.push('outside viewBox');
          if (boxes.some(b => rect.x < b.x + b.width && rect.x + rect.width > b.x && rect.y < b.y + b.height && rect.y + rect.height > b.y)) faults.push('overlapping nodes');
          boxes.push(rect);
        }
        if (svg.querySelectorAll('path').length !== boxes.length - 1) faults.push('missing edges');
        return faults;
      })), []);
      await page.getByText('Text outline: Handbook', { exact: true }).focus();
      await page.keyboard.press('Enter');
      const outline = page.locator('.mindmap-outline').first();
      await expect(outline.locator('li').filter({ hasText: /^Native/ }).first()).toContainText('日本 café 👩‍💻');
      await expect(outline.locator('li').filter({ hasText: /^Install/ }).first().locator('ul > li')).toHaveText(['Linux', 'macOS', 'Windows']);
      if (javaScriptEnabled) {
        await page.getByLabel('Theme', { exact: true }).selectOption('forest');
        await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
        assert.equal(await diagram.locator('.mindmap-node[data-depth="1"] text').first().evaluate(n => getComputedStyle(n).fill), await page.locator('article').evaluate(n => getComputedStyle(n).color));
        assert.equal(await diagram.locator('.mindmap-node[data-depth="0"] text').evaluate(n => getComputedStyle(n).fill), await page.locator('body').evaluate(n => getComputedStyle(n).backgroundColor));
        await page.getByRole('button', { name: /Search/ }).click();
        await page.getByRole('searchbox').fill('Nebula');
        await page.locator('#_zel-results a').click();
        await expect(page).toHaveURL(url => url.pathname === new URL(nested).pathname);
      }
      if (process.env.ZELLIJ_PORTABLE_ARTIFACTS && javaScriptEnabled) {
        await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-mindmap.png`), fullPage: true });
      }
      await page.setViewportSize({ width: 390, height: 844 });
      await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      const scroll = page.locator('.mindmap-scroll').first();
      assert(await scroll.evaluate(n => n.scrollWidth > n.clientWidth));
      await scroll.focus();
      await expect(scroll).toBeFocused();
      // WebKit lacks native arrow-key scrolling here even for a plain overflow
      // div. The local enhancement supplies it; the no-JS text outline above
      // exposes the complete hierarchy through native keyboard disclosure.
      if (javaScriptEnabled || engine !== 'webkit') {
        await page.keyboard.press('ArrowRight');
        await expect.poll(() => scroll.evaluate(n => n.scrollLeft)).toBeGreaterThan(0);
      }
      if (javaScriptEnabled) {
        await page.keyboard.press('End');
        await expect.poll(() => scroll.evaluate(n => n.scrollLeft + n.clientWidth >= n.scrollWidth - 1)).toBe(true);
        await page.keyboard.press('Home');
        await expect.poll(() => scroll.evaluate(n => n.scrollLeft)).toBe(0);
      }
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      await page.locator('article').getByRole('link', { name: 'Read maps' }).click();
      await expect(page).toHaveURL(url => url.hash === '#hierarchy');
      await expect(diagram).toBeVisible();
      assert(requests.every(url => /^(file|data):/.test(url)), 'Diagrams load without an HTTP server or network');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
  console.log(`${engine}: native SVG mindmaps, text outlines, layout, escaping, search/themes, offline navigation, keyboard scrolling and JavaScript-disabled rendering passed.`);
}
