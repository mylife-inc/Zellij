// Capture only after an assertion fails. No observers, frame loops or polling
// are installed on the passing path, and capture must never replace the error.

/**
 * @param {import('@playwright/test').Page} page
 * @param {{ sampleFrames?: boolean }} options Disable frame sampling for pages
 * with JavaScript disabled: native DOM evaluation works, page timers do not.
 */
export async function browserFailureState(page, { sampleFrames = true } = {}) {
  let timer;
  const partial = {};
  try {
    return await Promise.race([
      (async () => {
        // Return the immediate snapshot before depending on page scheduling.
        // A stalled timer/frame must not discard already captured DOM state.
        partial.before = await page.evaluate(readBrowserState);
        if (!sampleFrames) {
          partial.frameTimes = [];
          partial.sampledFrames = false;
        } else {
          partial.frameTimes = await page.evaluate(async () => {
            const frames = [];
            let active = true;
            let handle = requestAnimationFrame(time => {
              if (!active) return;
              frames.push(time);
              handle = requestAnimationFrame(next => { if (active) frames.push(next); });
            });
            await new Promise(resolve => setTimeout(resolve, 250));
            active = false;
            cancelAnimationFrame(handle);
            return frames;
          });
        }
        partial.after = await page.evaluate(readBrowserState);
        return { ...partial };
      })(),
      new Promise(resolve => { timer = setTimeout(() => resolve({ ...partial, captureError: 'Browser state capture exceeded 1500 ms' }), 1500); }),
    ]);
  } catch (error) {
    return { ...partial, captureError: String(error) };
  } finally {
    clearTimeout(timer);
  }
}

// Self-contained so Playwright can serialize it into the page execution context.
function readBrowserState() {
  const describe = element => element && ({
    tag: element.tagName, id: element.id, class: String(element.className),
    role: element.getAttribute('role'), label: element.getAttribute('aria-label'),
  });
  const geometry = element => {
    if (!element) return null;
    const box = element.getBoundingClientRect();
    return { x: box.x, y: box.y, width: box.width, height: box.height };
  };
  const read = () => {
    const nav = document.querySelector('.zellij-nav-desktop');
    const probes = document.querySelector('.zellij-nav-measure');
    return {
      at: performance.now(), url: location.href, readyState: document.readyState,
      visibility: document.visibilityState, documentFocus: document.hasFocus(),
      viewport: { width: innerWidth, height: innerHeight }, scroll: { x: scrollX, y: scrollY },
      fontStatus: document.fonts.status, active: describe(document.activeElement),
      header: geometry(document.querySelector('.zellij-header')),
      navigation: {
        box: geometry(nav), measured: nav?.getAttribute('data-measured'),
        items: [...document.querySelectorAll('.zellij-nav-desktop [data-nav-index]')].map(el => el.getAttribute('data-nav-index')),
        probeWidths: probes && [...probes.children].map(el => el.getBoundingClientRect().width),
        probeFonts: probes && [...probes.children].map(el => {
          const style = getComputedStyle(el);
          return { family: style.fontFamily, size: style.fontSize, weight: style.fontWeight };
        }),
        gap: probes && getComputedStyle(probes).columnGap,
        font: probes && getComputedStyle(probes).fontFamily,
      },
      themeControls: [...document.querySelectorAll('button[aria-label]')]
        .filter(el => /theme/i.test(el.getAttribute('aria-label') ?? ''))
        .map(el => ({ ...describe(el), box: geometry(el), display: getComputedStyle(el).display,
          visibility: getComputedStyle(el).visibility, hiddenBy: describe(el.closest('[aria-hidden="true"], [inert]')) })),
      drawerCount: document.querySelectorAll('.zellij-drawer-root').length,
    };
  };
  return read();
}
