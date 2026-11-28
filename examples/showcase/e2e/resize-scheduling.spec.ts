import { test, expect, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { browserFailureState } from '../../../scripts/browser-failure-state.mjs';
const variants = [
  { name: 'showcase-control', url: '/' },
  { name: 'escape-cancel', url: '/' },
  { name: 'showcase-flat', url: '/' },
  { name: 'dom-effects', url: '/__resize-reduction/dom.html' },
  { name: 'dom-flat', url: '/__resize-reduction/dom.html?effects=off' },
].filter(variant => variant.name === (process.env.PROBE_VARIANT ?? 'showcase-control'));
const observedErrors = new WeakMap<Page, string[]>();
test.afterEach(async ({ page }, info) => {
  if (info.status === info.expectedStatus) return;
  const output = info.outputPath('resize-state.json');
  await writeFile(output, JSON.stringify({ status: info.status, title: info.title,
    worker: info.parallelIndex, capturedAtEpochMs: Date.now(), pageErrors: observedErrors.get(page) ?? [],
    state: await browserFailureState(page) }, null, 2));
  await info.attach('resize-state', { path: output, contentType: 'application/json' });
});

// Distinct cases share the normal two worker/browser processes. No repeat-each
// worker restart, active heartbeat or injected stall; assertion deadlines stay 5s.
for (let iteration = 0; iteration < 32; iteration++) {
  for (const variant of variants) {
    test(`${variant.name} / sequence ${iteration + 1}`, async ({ page }, info) => {
      const errors: string[] = [];
      observedErrors.set(page, errors);
      page.on('pageerror', error => errors.push(error.message));
      if (variant.name === 'escape-cancel') await page.addInitScript(() => {
        document.addEventListener('keydown', event => {
          if (event.key === 'Escape') event.preventDefault();
        }, true);
      });
      await page.goto(variant.url, { waitUntil: 'networkidle' });
      await page.evaluate(() => document.fonts.ready);
      const nav = page.getByRole('navigation', { name: 'Main', exact: true });
      if (variant.url === '/') {
        await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
      }
      if (variant.name === 'showcase-flat') await page.addStyleTag({ content:
        '*, *::before, *::after { animation: none !important; transition: none !important; filter: none !important; backdrop-filter: none !important; -webkit-backdrop-filter: none !important; }',
      });
      for (let cycle = 0; cycle < 5; cycle++) {
        await test.step(`narrow/open/Escape/widen ${cycle + 1}`, async () => {
          await page.setViewportSize({ width: 901, height: 900 });
          const more = nav.locator('[data-nav-index="more"] > button');
          await expect(more).toBeVisible();
          await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
          await more.focus();
          await page.keyboard.press('ArrowDown');
          await expect(nav.locator('[data-nav-index="more"] a').first()).toBeFocused();
          await expect(nav.locator('[data-nav-index="more"] a').filter({ hasText: 'Guide' }).last()).toBeVisible();
          await page.keyboard.press('Escape');
          await expect(more).toBeFocused();
          // Keep the critical Escape → resize interval free of extra evaluation.
          await page.setViewportSize({ width: 1600, height: 900 });
          await expect(more).toHaveCount(0);
          await expect(nav.getByRole('link', { name: 'Guide', exact: true })).toBeVisible();
          await expect(nav.locator('[data-zellij-navitem]:focus')).toHaveCount(1);
        });
      }
      await info.attach('page-errors', { body: JSON.stringify(errors), contentType: 'application/json' });
      expect(errors).toEqual([]);
    });
  }
}
