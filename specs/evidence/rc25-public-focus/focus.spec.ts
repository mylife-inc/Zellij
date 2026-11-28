import { test, expect, type Page } from '@playwright/test';
const network = new WeakMap<Page, string[]>();
const errors = new WeakMap<Page, string[]>();
test.beforeEach(async ({ page }) => {
  const requests: string[] = []; const failures: string[] = [];
  network.set(page, requests); errors.set(page, failures);
  page.on('pageerror', e => failures.push(e.message));
  await page.route(/^https?:/, async route => { requests.push(route.request().url()); await route.abort(); });
});
test.afterEach(async ({ page }, info) => {
  await info.attach('offline-network', { body: JSON.stringify(network.get(page)), contentType: 'application/json' });
  await info.attach('page-errors', { body: JSON.stringify(errors.get(page)), contentType: 'application/json' });
  expect(network.get(page)).toEqual([]); expect(errors.get(page)).toEqual([]);
});
// Queue the real application callbacks while native frames still deliver resize
// observations. This models the observed order: a new measurement and an older
// focus callback execute before React commits the new list of visible links.
async function installOverflowCommitFrames(page: Page) {
  await page.addInitScript(() => {
    const request = window.requestAnimationFrame.bind(window);
    const cancel = window.cancelAnimationFrame.bind(window);
    const Observer = window.ResizeObserver;
    const navigationObservers = new WeakSet<ResizeObserver>();
    const callbacks = new Map<number, { callback: FrameRequestCallback; measurement: boolean }>();
    let measuring = false;
    let held = false;
    let id = 0;
    window.ResizeObserver = class extends Observer {
      constructor(callback: ResizeObserverCallback) {
        super((entries, observer) => {
          const previous = measuring;
          measuring = navigationObservers.has(observer);
          try { callback(entries, observer); } finally { measuring = previous; }
        });
      }
      override observe(target: Element, options?: ResizeObserverOptions) {
        if (target.matches('.zellij-nav-desktop')) navigationObservers.add(this);
        super.observe(target, options);
      }
    };
    window.requestAnimationFrame = callback => {
      if (!held) return request(callback);
      callbacks.set(--id, { callback, measurement: measuring });
      return id;
    };
    window.cancelAnimationFrame = handle => {
      if (handle < 0) callbacks.delete(handle);
      else cancel(handle);
    };
    const drive = () => { if (held) request(drive); };
    const run = (handle: number) => {
      const entry = callbacks.get(handle);
      if (!entry) throw new Error('Expected queued callback was cancelled');
      callbacks.delete(handle);
      entry.callback(performance.now());
    };
    const state = {
      hold() { held = true; request(drive); },
      pending() {
        const entries = [...callbacks.values()];
        return { measurement: entries.filter(entry => entry.measurement).length, focus: entries.filter(entry => !entry.measurement).length };
      },
      measure: () => new Promise<void>((resolve, reject) => request(() => {
        try {
          const entry = [...callbacks].find(([, value]) => value.measurement);
          if (!entry) throw new Error('No navigation measurement queued');
          run(entry[0]);
          request(() => request(() => resolve()));
        } catch (error) { reject(error); }
      })),
      resizeAndRun(width: number) {
        const entries = [...callbacks];
        const measurement = entries.filter(([, value]) => value.measurement);
        const focus = entries.filter(([, value]) => !value.measurement);
        if (measurement.length !== 1 || focus.length !== 1) throw new Error('Expected one measurement and one older focus callback');
        const bar = document.querySelector<HTMLElement>('.zellij-nav-inner')!;
        bar.style.width = `${width}px`;
        // Run in one task, before ResizeObserver delivery or React's commit.
        run(measurement[0]![0]);
        run(focus[0]![0]);
      },
      release: () => new Promise<void>(resolve => {
        held = false;
        request(() => {
          for (const handle of [...callbacks.keys()]) if (callbacks.has(handle)) run(handle);
          request(() => request(() => resolve()));
        });
      }),
    };
    (window as unknown as { overflowCommitFrames: typeof state }).overflowCommitFrames = state;
  });
}

type OverflowCommitFrames = {
  hold(): void;
  pending(): { measurement: number; focus: number };
  measure(): Promise<void>;
  resizeAndRun(width: number): void;
  release(): Promise<void>;
};

for (const finalWidth of [901, 1600]) {
  test(`preserves the latest overflow request until its ${finalWidth}px menu commits`, async ({ page, isMobile }) => {
    test.skip(Boolean(isMobile), 'desktop overflow callback ordering');
    await installOverflowCommitFrames(page);
    await page.setViewportSize({ width: 1600, height: 900 });
    await page.goto(process.env.ZEL_FOCUS_URL!);
    await page.evaluate(() => document.fonts.ready);
    await page.addStyleTag({ content: '.zellij-nav-link, .zellij-nav-probe-label { font-size: 1.25rem; }' });
    const nav = page.getByRole('navigation', { name: 'Main', exact: true });
    const more = nav.locator('[data-nav-index="more"] > button');
    const guide = nav.getByRole('link', { name: 'Guide', exact: true });
    await page.locator('.zellij-nav-inner').evaluate((bar, width) => { (bar as HTMLElement).style.width = `${width}px`; }, finalWidth);
    await expect(more).toHaveCount(finalWidth === 901 ? 1 : 0);
    if (finalWidth === 901) await expect(nav.locator('[data-nav-index]:not([data-nav-index="more"])')).toHaveCount(0);
    await (finalWidth === 901 ? more : guide).focus();
    await page.evaluate(() => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.hold());
    await page.locator('.zellij-nav-inner').evaluate((bar, width) => { (bar as HTMLElement).style.width = `${width}px`; }, finalWidth === 901 ? 1600 : 901);
    await expect.poll(() => page.evaluate(() => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.pending().measurement)).toBe(1);
    await page.evaluate(() => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.measure());
    await expect(more).toHaveCount(finalWidth === 901 ? 0 : 1);
    const current = finalWidth === 901 ? guide : more;
    await current.focus();
    await expect(current).toBeFocused();
    // Queue a real observer measurement even if this browser does not resize
    // the measured navigation element when the visible link list changes.
    await page.locator('.zellij-nav-inner').evaluate((bar, width) => { (bar as HTMLElement).style.width = `${width}px`; }, finalWidth === 901 ? 1601 : 902);
    await expect.poll(() => page.evaluate(() => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.pending())).toEqual({ measurement: 1, focus: 1 });
    await page.evaluate(width => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.resizeAndRun(width), finalWidth);
    await expect(more).toHaveCount(finalWidth === 901 ? 1 : 0);
    await page.evaluate(() => (window as unknown as { overflowCommitFrames: OverflowCommitFrames }).overflowCommitFrames.release());
    await expect(nav.locator('[data-zellij-navitem]:focus')).toHaveCount(1);
    if (finalWidth === 901) await expect(more).toBeFocused();
    else await expect(nav.locator('[data-zellij-navitem]').first()).toBeFocused();
  });
}
