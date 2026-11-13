import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';

test('records focus while an open mobile drawer changes to desktop', async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    const entries: unknown[] = [];
    (window as any).__focusProbe = entries;
    const identify = (el: Element | null) => el && ({ tag: el.tagName, class: el.className, label: el.getAttribute('aria-label'), text: el.textContent?.slice(0, 50) });
    const record = (event: string, target?: Element | null, stack?: string) => entries.push({
      at: performance.now(), event, active: identify(document.activeElement), target: identify(target ?? null), stack,
      width: innerWidth, drawer: !!document.querySelector('.zellij-drawer-root'),
    });
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
  try {
    await page.setViewportSize({ width: 390, height: 900 });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open navigation' }).click();
    await expect(page.getByRole('dialog', { name: 'Site navigation' })).toBeVisible();
    await page.setViewportSize({ width: 1280, height: 900 });
    await expect(page.getByRole('dialog', { name: 'Site navigation' })).toHaveCount(0);
    await expect(page.locator('body')).not.toHaveCSS('overflow', 'hidden');
    await expect(page.locator('.zellij-nav-list [data-zellij-navitem]').first()).toBeFocused();
  } finally {
    const entries = await page.evaluate(() => (window as any).__focusProbe);
    mkdirSync('focus-probe-results', { recursive: true });
    writeFileSync(`focus-probe-results/${testInfo.repeatEachIndex}-${testInfo.workerIndex}.json`, JSON.stringify(entries, null, 2));
    await testInfo.attach('focus-events', { body: JSON.stringify(entries, null, 2), contentType: 'application/json' });
  }
});
