// Exercise native math in the relocated artifact, with no tools/server/network.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect } from '@playwright/test';

export async function checkMath({ binary, root, env, engine, browser }) {
  const source = path.join(root, `Math 日本 ${engine}`);
  const output = `${source}-site`;
  const moved = path.join(root, `Moved equations café ${engine}`);
  await mkdir(path.join(source, 'guide'), { recursive: true });
  await writeFile(path.join(source, 'zel.yaml'), 'markdown: {math: true}\n');
  await writeFile(path.join(source, 'index.md'), '# Equations\n\n[Read formulas](guide/formulas.md#fractions)\n');
  await writeFile(path.join(source, 'guide/formulas.md'), String.raw`# Formulas

## Fractions

Inline $\frac{1}{2} + \sqrt{x}$ is built into the document.

$$
\begin{pmatrix}a & b \\ c & d\end{pmatrix}
$$

$$
\sum_{i=0}^{n}i^2 = \frac{n(n+1)(2n+1)}{6}
$$

$$
${Array.from({ length: 30 }, (_, i) => `x_{${i}}`).join(' + ')}
$$

## Energy $E=mc^2$

Escaped \$5 and code ${'`$\\notacommand$`'} stay literal.

$\text{</mtext><img src=x onerror=alert(1)>}$

$\operatorname{<script>alert(1)</script>}(x)$

[Home](../index.md)
`);
  const report = JSON.parse(execFileSync(binary, ['check', source, '--strict', '--json'], { env, encoding: 'utf8' }));
  assert.equal(report.ok, true);
  assert.deepEqual(report.diagnostics, []);
  execFileSync(binary, ['build', source], { env });
  await rename(output, moved);
  const font = await readFile(path.join(moved, '_zellij/math/STIXTwoMath-Regular.woff2'));
  assert.equal(createHash('sha256').update(font).digest('hex'), '094191335def3f0452c81ec0713cfc2f29bb6af8cecbf79b60881fbf2db97562');
  for (const javaScriptEnabled of [true, false]) {
    const context = await browser.newContext({ javaScriptEnabled, offline: engine === 'chromium', serviceWorkers: 'block' });
    const requests = [];
    const errors = [];
    context.on('request', request => requests.push(request.url()));
    await context.route(/^https?:/, route => route.abort());
    try {
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      page.on('dialog', dialog => { errors.push(dialog.message()); dialog.dismiss(); });
      const home = pathToFileURL(path.join(moved, 'index.html')).href;
      const nested = pathToFileURL(path.join(moved, 'guide/formulas.html')).href;
      // Start on the nested page: no warmed scripts or home-page cache.
      await page.goto(nested);
      await expect(page.locator('article mfrac').first()).toBeVisible();
      await expect(page.locator('article msqrt')).toBeVisible();
      await expect(page.locator('article mtable')).toBeVisible();
      assert.equal(await page.evaluate(async () => {
        await document.fonts.ready;
        return [...document.fonts].some(font => font.family.includes('Zellij STIX Two Math') && font.status === 'loaded');
      }), true, 'The actual equation loads the bundled math font through file://');
      assert(await page.locator('mfrac').first().evaluate(fraction => {
        const a = fraction.children[0].getBoundingClientRect();
        const b = fraction.children[1].getBoundingClientRect();
        return a.width > 0 && b.width > 0 && a.bottom <= b.top;
      }), 'Numerator renders above denominator, not as unformatted source');
      await expect(page.locator('article img, article script')).toHaveCount(0);
      await expect(page.locator('article')).toContainText('Escaped $5');
      await expect(page.locator('article code')).toHaveText('$\\notacommand$');
      if (process.env.ZELLIJ_PORTABLE_ARTIFACTS && javaScriptEnabled) {
        await page.screenshot({ path: path.join(process.env.ZELLIJ_PORTABLE_ARTIFACTS, `${engine}-math.png`), fullPage: true });
      }
      if (javaScriptEnabled) {
        await page.getByLabel('Theme', { exact: true }).selectOption('forest');
        await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
        assert.equal(await page.locator('math').first().evaluate(node => getComputedStyle(node).color), await page.locator('article').evaluate(node => getComputedStyle(node).color));
        await page.getByRole('button', { name: /Search/ }).click();
        await page.getByRole('searchbox').fill('E=mc^2');
        await page.locator('#_zel-results a').click();
        assert.equal(new URL(page.url()).hash, '#energy-emc2');
      }
      await page.setViewportSize({ width: 390, height: 844 });
      // WebKit can settle the resized layout after setViewportSize resolves.
      await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), { message: 'Equations do not overflow the mobile page' }).toBe(true);
      const wide = page.locator('.math-display').nth(2);
      assert(await wide.evaluate(node => node.scrollWidth > node.clientWidth), 'Wide formula can scroll independently');
      await wide.focus();
      await expect(wide).toBeFocused();
      await page.locator('article').getByRole('link', { name: 'Home', exact: true }).click();
      await expect(page).toHaveURL(url => url.pathname === new URL(home).pathname);
      await page.locator('article').getByRole('link', { name: 'Read formulas' }).click();
      await expect(page).toHaveURL(url => url.pathname === new URL(nested).pathname && url.hash === '#fractions');
      if (javaScriptEnabled) {
        await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
        await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'dark');
      }
      await expect(page.locator('article mfrac').first()).toBeVisible();
      assert(requests.every(url => /^(file|data):/.test(url)), 'Math, font and CSS need no network');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
  console.log(`${engine}: native math, bundled font, fractions/matrices, offline search/navigation, escaping, mobile and JavaScript-disabled rendering passed.`);
}
