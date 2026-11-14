import { test } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const events: unknown[] = [];
    const capture = (event: string) => {
      const nav = document.querySelector('.zellij-nav-desktop');
      const probes = document.querySelector('.zellij-nav-measure');
      const active = document.activeElement;
      const entry = {
        event, time: performance.now(), width: innerWidth, height: innerHeight,
        visibility: document.visibilityState, documentFocus: document.hasFocus(),
        mobile: matchMedia('(max-width: 900px)').matches,
        active: active && { tag: active.tagName, class: active.className },
        navWidth: nav?.getBoundingClientRect().width, measured: nav?.getAttribute('data-measured'),
        probeWidths: probes && [...probes.children].map(el => el.getBoundingClientRect().width),
        count: document.querySelectorAll('.zellij-nav-desktop [data-nav-index]').length,
        drawer: !!document.querySelector('.zellij-drawer-root'), overflow: document.body?.style.overflow,
      };
      events.push(entry);
      return entry;
    };
    (window as any).__headerObservation = { events, capture };
    addEventListener('resize', () => {
      capture('resize');
      requestAnimationFrame(() => capture('resize-frame'));
    });
    matchMedia('(max-width: 900px)').addEventListener('change', () => capture('media'));
    document.addEventListener('visibilitychange', () => capture('visibility'));
    document.addEventListener('focusin', () => capture('focusin'), true);
    document.addEventListener('focusout', () => capture('focusout'), true);
    document.addEventListener('DOMContentLoaded', () => {
      capture('loaded');
      const nav = document.querySelector('.zellij-nav-desktop');
      if (nav) new ResizeObserver(() => capture('nav-resize')).observe(nav);
      document.fonts.ready.then(() => capture('fonts-ready'));
    });
  });
});

test.afterEach(async ({ page }, info) => {
  if (info.status === info.expectedStatus) return;
  const observation = await page.evaluate(() => {
    const data = (window as any).__headerObservation;
    data?.capture('failure');
    return data?.events;
  });
  mkdirSync('focus-probe-results', { recursive: true });
  writeFileSync(`focus-probe-results/header-${info.testId}-${info.repeatEachIndex}.json`, JSON.stringify({ title: info.title, observation }, null, 2));
});
