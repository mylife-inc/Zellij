import { test } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const events: unknown[] = [];
    const pending = new Map<number, number>();
    const record = (event: string, detail?: unknown) => {
      if (events.length < 1500) events.push({ event, at: performance.now(), detail });
    };
    const request = window.requestAnimationFrame.bind(window);
    const cancel = window.cancelAnimationFrame.bind(window);
    window.requestAnimationFrame = callback => {
      const id = request(time => {
        pending.delete(id);
        record('frame-run', { id });
        callback(time);
      });
      pending.set(id, performance.now());
      record('frame-request', { id });
      return id;
    };
    window.cancelAnimationFrame = id => {
      pending.delete(id);
      record('frame-cancel', { id });
      cancel(id);
    };
    const Observer = window.ResizeObserver;
    window.ResizeObserver = class extends Observer {
      constructor(callback: ResizeObserverCallback) {
        super((entries, observer) => {
          record('resize-observer', entries.map(entry => ({
            class: entry.target.className, width: entry.contentRect.width, height: entry.contentRect.height,
          })));
          callback(entries, observer);
        });
      }
    };
    addEventListener('resize', () => record('window-resize'));
    document.addEventListener('visibilitychange', () => record('visibility', document.visibilityState));
    (window as unknown as { timingObservation: unknown }).timingObservation = { events, pending, request };
  });
});

test.afterEach(async ({ page }, info) => {
  if (info.status === info.expectedStatus) return;
  const observation = await page.evaluate(async () => {
    const data = (window as any).timingObservation;
    const nav = document.querySelector('.zellij-nav-desktop');
    const probes = document.querySelector('.zellij-nav-measure');
    const current = {
      at: performance.now(), width: innerWidth, height: innerHeight,
      visibility: document.visibilityState, documentFocus: document.hasFocus(), ready: document.readyState,
      navWidth: nav?.getBoundingClientRect().width, measured: nav?.getAttribute('data-measured'),
      probeWidths: probes && [...probes.children].map(el => el.getBoundingClientRect().width),
      gap: probes && getComputedStyle(probes).columnGap,
      items: [...document.querySelectorAll('.zellij-nav-desktop [data-nav-index]')].map(el => el.getAttribute('data-nav-index')),
      active: document.activeElement?.outerHTML.slice(0, 1000),
      pending: [...data.pending], events: [...data.events],
    };
    let frameAt: number | null = null;
    data.request(() => { frameAt = performance.now(); });
    await new Promise(resolve => setTimeout(resolve, 1000));
    return { ...current, frameAt, timerAt: performance.now(), eventsAfterProbe: data.events };
  });
  mkdirSync('timing-results', { recursive: true });
  writeFileSync(`timing-results/overflow-${info.repeatEachIndex}-${info.workerIndex}.json`, JSON.stringify(observation, null, 2));
});
