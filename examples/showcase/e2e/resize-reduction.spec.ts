import { test, expect, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { browserFailureState } from '../../../scripts/browser-failure-state.mjs';
import { installResizeMonitor } from '../../../scripts/browser-resize-reduction/monitor.mjs';

const variants = [
  { name: 'dom-flat', url: '/__resize-reduction/dom.html?effects=off', monitor: true },
  { name: 'dom-effects', url: '/__resize-reduction/dom.html', monitor: true },
  { name: 'actual-hooks', url: '/__resize-reduction/react.html', monitor: true },
  { name: 'showcase-observed', url: '/', monitor: true },
  { name: 'showcase-control', url: '/', monitor: false },
];

const observedErrors = new WeakMap<Page, string[]>();

test.afterEach(async ({ page }, info) => {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const diagnostic: Record<string, unknown> = { status: info.status, title: info.title, worker: info.parallelIndex, pageErrors: observedErrors.get(page) ?? [] };
  if (info.status !== info.expectedStatus) diagnostic.failure = await browserFailureState(page);
  try {
    diagnostic.scheduling = await Promise.race([
      page.evaluate(() => (window as any).__resizeProbe?.snapshot() ?? { instrumentation: false }),
      new Promise(resolve => { timer = setTimeout(() => resolve({ captureError: 'Monitor snapshot exceeded 1500 ms' }), 1500); }),
    ]);
  } catch (error) { diagnostic.captureError = String(error); }
  finally { clearTimeout(timer); }
  const output = info.outputPath('resize-state.json');
  await writeFile(output, JSON.stringify(diagnostic, null, 2));
  await info.attach('resize-state', { path: output, contentType: 'application/json' });
});

// Distinct cases share the normal two worker/browser processes. No repeat-each
// worker restart, active heartbeat or injected stall; assertion deadlines stay 5s.
for (let iteration = 0; iteration < 16; iteration++) {
  for (const variant of variants) {
    test(`${variant.name} / sequence ${iteration + 1}`, async ({ page }, info) => {
      const errors: string[] = [];
      observedErrors.set(page, errors);
      page.on('pageerror', error => errors.push(error.message));
      if (variant.monitor) await page.addInitScript(installResizeMonitor);
      await page.goto(variant.url, { waitUntil: 'networkidle' });
      await page.evaluate(() => document.fonts.ready);
      const nav = page.getByRole('navigation', { name: 'Main', exact: true });
      if (variant.url === '/') {
        await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
      }
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
