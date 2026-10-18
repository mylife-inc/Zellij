import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, rename, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect } from '@playwright/test';

export async function checkMermaid({ binary, root, env, engine, browser }) {
  const source = path.join(root, `Mermaid 日本 ${engine}`);
  const moved = path.join(root, `Moved diagrams café ${engine}`);
  await mkdir(path.join(source, 'guide'), { recursive: true });
  await writeFile(path.join(source, 'zel.yaml'), 'markdown: {mermaid: true}\n');
  await writeFile(path.join(source, 'index.md'), '# Home\n[Diagrams](guide/diagrams.md#graphs)');
  const graphs = [
    'flowchart LR\nA[Portable nebula] --> B[日本 café]',
    'sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Ready',
    'stateDiagram-v2\n[*] --> Working\nWorking --> [*]',
    'pie title Progress\n"Ready": 75\n"Remaining": 25',
    'mindmap\n  root((Zellij))\n    Files\n    Browser',
    '%%{init: {"forceLegacyMathML":true}}%%\nflowchart LR\nA["$$\\frac{1}{2}$$"] --> B["$$x^2$$"]',
    'invalid diagram syntax',
    // An author directive cannot lower security or turn on legacy KaTeX.
    '%%{init: {"securityLevel":"loose", "forceLegacyMathML":true}}%%\nflowchart LR\nA["&lt;img src=x onerror=alert(1)&gt;"] --> B[Safe]\nclick B "javascript:alert(2)"',
    '%%{init: {"maxEdges":99999}}%%\nflowchart LR\n' + 'A-->B\n'.repeat(501),
    'flowchart LR\nAfter[After invalid diagram] --> Ready',
  ];
  const body = '# Diagrams\n\n## Graphs\n\n' + graphs.map(g => '```mermaid\n' + g + '\n```\n').join('\n') + '\n```mindmap\nNative: [Alongside, Mermaid]\n```\n\n[Home](../index.md)';
  await writeFile(path.join(source, 'guide/diagrams.md'), body);
  const report = JSON.parse(execFileSync(binary, ['check', source, '--strict', '--json'], { env, encoding: 'utf8' }));
  assert.equal(report.ok, true, 'Native check validates bounds, not Mermaid grammar');
  execFileSync(binary, ['build', source], { env, stdio: 'pipe' });
  await rename(`${source}-site`, moved);
  const nested = pathToFileURL(path.join(moved, 'guide/diagrams.html')).href;
  for (const javaScriptEnabled of [true, false]) {
    const context = await browser.newContext({ javaScriptEnabled, offline: engine === 'chromium', serviceWorkers: 'block' });
    const requests = [];
    const errors = [];
    context.on('request', request => requests.push(request.url()));
    await context.route(/^https?:/, route => route.abort());
    try {
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      page.on('dialog', dialog => { errors.push(dialog.message()); void dialog.dismiss(); });
      await page.goto(nested);
      const figures = page.locator('.mermaid-diagram');
      await expect(figures).toHaveCount(graphs.length);
      if (javaScriptEnabled) {
        await expect(figures.last()).toHaveAttribute('data-status', 'drawn', { timeout: 30000 });
        await expect(page.locator('.mermaid-diagram[data-status=drawn]')).toHaveCount(8);
        await expect(figures.nth(6)).toHaveAttribute('data-status', 'failed');
        await expect(figures.nth(6).locator('[role=status]')).toContainText('Diagram could not be drawn');
        await expect(figures.nth(6).locator('code')).toBeVisible();
        await expect(figures.nth(5).locator('mfrac')).toBeVisible();
        await expect(figures.nth(5).locator('.katex-html')).toHaveCount(0);
        await expect(figures.nth(8)).toHaveAttribute('data-status', 'failed');
        await expect(figures.nth(8).locator('[role=status]')).toContainText('500');
        await expect(figures.first().locator('svg')).toBeVisible();
        await expect(figures.first().locator('svg')).toContainText('Portable nebula');
        // Mermaid may keep an inert <a>, but removes its unsafe destination.
        await expect(figures.nth(7).locator('img, a[href], [onclick], [onerror]')).toHaveCount(0);
        await expect(page.locator('.mermaid-stage')).toHaveCount(0);
        assert.equal(await page.locator('[id]').evaluateAll(nodes => {
          const ids = nodes.map(node => node.id); return new Set(ids).size === ids.length;
        }), true, 'SVG IDs do not collide across diagrams');
        const before = await figures.first().locator('svg').getAttribute('id');
        await page.getByLabel('Theme', { exact: true }).selectOption('forest');
        await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
        await expect(figures.last()).toHaveAttribute('data-theme', 'forest');
        await expect(figures.last()).toHaveAttribute('data-mode', 'dark');
        assert.notEqual(await figures.first().locator('svg').getAttribute('id'), before);
        await page.getByLabel('Appearance', { exact: true }).selectOption('system');
        await page.emulateMedia({ colorScheme: 'light' });
        await expect(figures.last()).toHaveAttribute('data-mode', 'light');
        await page.emulateMedia({ colorScheme: 'dark' });
        await expect(figures.last()).toHaveAttribute('data-mode', 'dark');
        // Several mutations during rendering must converge on the final theme.
        await page.evaluate(() => {
          document.documentElement.dataset.zellijTheme = 'cupertino';
          document.documentElement.dataset.zellijMode = 'light';
          setTimeout(() => { document.documentElement.dataset.zellijTheme = 'forest'; }, 0);
        });
        await expect(figures.last()).toHaveAttribute('data-theme', 'forest');
        await expect(figures.last()).toHaveAttribute('data-mode', 'light');
        await figures.first().getByText('Diagram 1 source (Mermaid)').click();
        await expect(figures.first().locator('code')).toBeVisible();
        await page.getByRole('button', { name: /Search/ }).click();
        await page.getByRole('searchbox').fill('Portable nebula');
        await page.locator('#_zel-results a').click();
        await expect(figures.last()).toHaveAttribute('data-status', 'drawn', { timeout: 30000 });
      } else {
        await expect(figures.locator('svg')).toHaveCount(0);
        for (let i = 0; i < graphs.length; i++) await expect(figures.nth(i).locator('code')).toBeVisible();
      }
      await page.setViewportSize({ width: 390, height: 844 });
      await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      if (process.env.ZELLIJ_PORTABLE_ARTIFACTS && javaScriptEnabled) {
        await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-mermaid.png`), fullPage: false });
      }
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      await expect(page.locator('script[src*=mermaid]')).toHaveCount(0);
      await page.locator('article').getByRole('link', { name: 'Diagrams', exact: true }).click();
      await expect(page).toHaveURL(url => url.pathname === new URL(nested).pathname && url.hash === '#graphs');
      if (javaScriptEnabled) await expect(page.locator('.mermaid-diagram').last()).toHaveAttribute('data-status', 'drawn', { timeout: 30000 });
      assert(requests.every(url => /^(file|data):/.test(url)), 'All diagrams and math work without HTTP requests');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
  const script = path.join(moved, '_zellij/mermaid/mermaid.js');
  await rename(script, `${script}.unavailable`);
  const fallback = await browser.newContext();
  try {
    const page = await fallback.newPage();
    await page.goto(nested);
    await expect(page.locator('.mermaid-diagram svg')).toHaveCount(0);
    await expect(page.locator('.mermaid-diagram').first().locator('code')).toBeVisible();
    await page.locator('.mermaid-diagram').first().locator('summary').focus();
    await page.keyboard.press('Space');
    await expect(page.locator('.mermaid-diagram').first().locator('code')).not.toBeVisible();
  } finally {
    await fallback.close();
    await rename(`${script}.unavailable`, script);
  }
  console.log(`${engine}: portable Mermaid, invalid-source fallback, strict labels, math, themes, relocated file navigation, mobile and JavaScript-disabled source passed.`);
}
