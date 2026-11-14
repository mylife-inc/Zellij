import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';

test('captures original rapid focus ordering', async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    const entries: unknown[] = [];
    (window as any).__focusProbe = entries;
    const identify = (el: Element | null) => el && ({ tag: el.tagName, class: el.className, label: el.getAttribute('aria-label'), text: el.textContent?.slice(0, 50), connected: el.isConnected, rects: el.getClientRects().length });
    const record = (event: string, target?: Element | null, stack?: string) => entries.push({
      at: performance.now(), event, active: identify(document.activeElement), target: identify(target ?? null), stack,
      width: innerWidth, mobile: matchMedia('(max-width: 900px)').matches, drawer: !!document.querySelector('.zellij-drawer-root'),
    });
    window.addEventListener('resize', () => record('resize'));
    const focus = HTMLElement.prototype.focus;
    HTMLElement.prototype.focus = function (...args) {
      record('focus-call-before', this, new Error().stack);
      focus.apply(this, args);
      record('focus-call-after', this);
    };
    document.addEventListener('focusin', event => record('focusin', event.target as Element), true);
    document.addEventListener('focusout', event => record('focusout', event.target as Element), true);
    matchMedia('(max-width: 900px)').addEventListener('change', () => {
      record('media-before-react');
      requestAnimationFrame(() => {
        record('media-frame-1');
        requestAnimationFrame(() => record('media-frame-2'));
      });
    });
  });

  let assertionsPassed = false;
  try {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    const drawer = page.getByRole('dialog', { name: 'Site navigation' });
    await expect(drawer).toBeVisible();
    for (const width of [1280, 390, 1280, 390]) await page.setViewportSize({ width, height: 900 });
    await expect(drawer).toHaveCount(0);
    await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
    await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused();
    assertionsPassed = true;
  } finally {
    const events = await page.evaluate(() => (window as any).__focusProbe);
    const result = { repetition: testInfo.repeatEachIndex, assertionsPassed, events };
    mkdirSync('focus-probe-results', { recursive: true });
    writeFileSync(`focus-probe-results/raw-${testInfo.repeatEachIndex}-${testInfo.workerIndex}.json`, JSON.stringify(result, null, 2));
    await testInfo.attach('focus-events', { body: JSON.stringify(result, null, 2), contentType: 'application/json' });
  }
});
