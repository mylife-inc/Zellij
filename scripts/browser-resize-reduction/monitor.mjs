// Passive wrappers: record existing work, never start a frame loop or force layout.
// Their overhead can alter timing; the uninstrumented full-site control is separate.
export function installResizeMonitor() {
  const events = [];
  const totals = Object.create(null);
  const pending = new Map();
  const describe = element => element && ({
    tag: element.tagName, id: element.id, connected: element.isConnected,
    index: element.closest?.('[data-nav-index]')?.getAttribute('data-nav-index'),
    label: element.getAttribute?.('aria-label'),
  });
  const active = () => describe(document.activeElement);
  const watched = new WeakSet();
  const record = (kind, detail = {}) => {
    totals[kind] = (totals[kind] ?? 0) + 1;
    events.push({ at: performance.now(), kind, ...detail });
    if (events.length > 512) events.shift();
  };
  const request = window.requestAnimationFrame.bind(window);
  const cancel = window.cancelAnimationFrame.bind(window);
  window.requestAnimationFrame = callback => {
    const scheduledAt = performance.now();
    const handle = request(time => {
      pending.delete(handle);
      record('frame-fired', { handle, scheduledAt, elapsed: performance.now() - scheduledAt, active: active() });
      callback.call(window, time);
      record('frame-returned', { handle, active: active() });
    });
    pending.set(handle, { scheduledAt, callback: callback.name });
    record('frame-requested', { handle, callback: callback.name, active: active() });
    return handle;
  };
  window.cancelAnimationFrame = handle => {
    pending.delete(handle);
    record('frame-cancelled', { handle });
    cancel(handle);
  };
  const NativeObserver = window.ResizeObserver;
  let observerId = 0;
  window.ResizeObserver = class extends NativeObserver {
    constructor(callback) {
      const id = ++observerId;
      super((entries, observer) => {
        record('observer-delivered', { id, count: entries.length });
        callback(entries, observer);
      });
      this.probeId = id;
      record('observer-created', { id });
    }
    observe(target, options) {
      record('observer-observe', { id: this.probeId, tag: target.tagName, class: String(target.className) });
      super.observe(target, options);
      if (target.matches('.zellij-nav-desktop') && !watched.has(target)) {
        watched.add(target);
        new MutationObserver(() => record('nav-commit', {
          items: [...target.querySelectorAll('[data-nav-index]')].map(item => item.getAttribute('data-nav-index')),
          active: active(),
        })).observe(target, { childList: true, subtree: true });
      }
    }
    disconnect() {
      record('observer-disconnect', { id: this.probeId });
      super.disconnect();
    }
  };
  for (const type of ['focusin', 'focusout']) document.addEventListener(type, event => record(type, { target: describe(event.target), related: describe(event.relatedTarget), active: active() }), true);
  window.addEventListener('resize', () => record('window-resize', { width: innerWidth, height: innerHeight, active: active() }));
  window.addEventListener('keydown', event => record('keydown', { key: event.key }), true);
  document.addEventListener('visibilitychange', () => record('visibility', { value: document.visibilityState }));
  window.__resizeProbe = {
    snapshot: () => ({
      at: performance.now(), viewport: { width: innerWidth, height: innerHeight },
      totals: { ...totals }, pending: [...pending].map(([handle, data]) => ({ handle, ...data })),
      events: [...events], visibility: document.visibilityState, documentFocus: document.hasFocus(),
    }),
  };
}
