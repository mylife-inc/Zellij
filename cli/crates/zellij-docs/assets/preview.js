/* Injected only by `zel view --watch`; never written into portable exports. */
(() => {
  'use strict';
  const script = new URL(document.currentScript.src);
  const version = script.searchParams.get('version');
  const endpoint = new URL('preview.json', script);
  let notice;
  async function poll() {
    try {
      const response = await fetch(endpoint, { cache: 'no-store' });
      if (response.ok) {
        const state = await response.json();
        if (state.version !== version) { location.reload(); return; }
        if (state.error && !notice) {
          notice = document.createElement('div');
          notice.setAttribute('role', 'alert');
          notice.id = '_zel-preview-error';
          notice.textContent = 'Preview build failed. Showing the last successful build. See the terminal for details.';
          notice.style.cssText = 'position:fixed;bottom:1rem;left:1rem;right:1rem;z-index:2147483647;padding:1rem;border:1px solid #b91c1c;border-radius:.5rem;background:#fff1f2;color:#881337;font:14px/1.5 system-ui;box-shadow:0 2px 12px #0003';
          document.body.append(notice);
        } else if (!state.error && notice) { notice.remove(); notice = undefined; }
      }
    } catch { /* A stopped/restarting preview is retried without a reload loop. */ }
    setTimeout(poll, 500);
  }
  poll();
})();
