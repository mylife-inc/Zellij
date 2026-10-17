/* Portable enhancement only: navigation and content work without this script. */
(() => {
  'use strict';
  const root = document.documentElement;
  const rootURL = new URL(document.body.dataset.root || './', location.href);
  const theme = document.getElementById('_zel-theme');
  const mode = document.getElementById('_zel-mode');
  const validThemes = new Set([...theme.options].map(option => option.value));
  const validModes = new Set(['system', 'light', 'dark']);
  const params = new URL(location.href).searchParams;
  const storageKey = `zellij-docs:${rootURL.pathname}`;
  let saved = {};
  try { saved = JSON.parse(localStorage.getItem(storageKey) || '{}'); } catch {}
  const initialTheme = params.get('_zel_theme') || saved?.theme;
  const initialMode = params.get('_zel_mode') || saved?.mode;
  if (validThemes.has(initialTheme)) theme.value = initialTheme;
  if (validModes.has(initialMode)) mode.value = initialMode;
  const apply = () => {
    root.dataset.zellijTheme = theme.value;
    root.dataset.zellijMode = mode.value;
    try { localStorage.setItem(storageKey, JSON.stringify({ theme: theme.value, mode: mode.value })); } catch {}
  };
  apply();
  theme.addEventListener('change', apply);
  mode.addEventListener('change', apply);
  document.querySelector('.controls').hidden = false;
  root.classList.add('js');

  // file:// storage is browser-dependent. Carry preferences on internal page
  // navigation as well; authored query parameters and fragments are retained.
  document.addEventListener('click', event => {
    const anchor = event.target.closest('a[href]');
    if (!anchor) return;
    const url = new URL(anchor.href, location.href);
    if (url.protocol !== rootURL.protocol || url.host !== rootURL.host || !url.pathname.startsWith(rootURL.pathname) || !url.pathname.endsWith('.html')) return;
    url.searchParams.set('_zel_theme', theme.value);
    url.searchParams.set('_zel_mode', mode.value);
    anchor.href = url.href;
  });

  const menu = document.getElementById('_zel-menu');
  menu.addEventListener('click', () => {
    const open = menu.getAttribute('aria-expanded') !== 'true';
    menu.setAttribute('aria-expanded', String(open));
    root.dataset.menuOpen = String(open);
  });

  const dialog = document.getElementById('_zel-search');
  const query = document.getElementById('_zel-query');
  const results = document.getElementById('_zel-results');
  const status = document.getElementById('_zel-status');
  const searchButton = document.getElementById('_zel-search-open');
  let returnFocus = searchButton;
  const open = event => {
    if (!dialog.open) {
      // Safari does not focus buttons on pointer click. Remember the explicit
      // trigger instead of relying solely on the dialog's native restoration.
      returnFocus = event?.currentTarget === searchButton ? searchButton : document.activeElement;
      if (!returnFocus || returnFocus === document.body) returnFocus = searchButton;
      dialog.showModal();
    }
    query.focus();
  };
  dialog.addEventListener('close', () => { if (returnFocus?.isConnected) returnFocus.focus(); });
  searchButton.addEventListener('click', open);
  document.addEventListener('keydown', event => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') { event.preventDefault(); open(); }
    if (event.key === 'Escape') {
      if (dialog.open) { event.preventDefault(); dialog.close(); }
      else if (root.dataset.menuOpen === 'true') {
        root.dataset.menuOpen = 'false'; menu.setAttribute('aria-expanded', 'false'); menu.focus();
      }
    }
  });
  dialog.addEventListener('click', event => {
    const bounds = dialog.getBoundingClientRect();
    if (event.target === dialog && (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom)) dialog.close();
  });
  const fold = text => text.normalize('NFKC').toLocaleLowerCase();
  const docs = (window.ZELLIJ_SEARCH || []).map(page => ({ ...page, haystack: fold(`${page.title} ${page.text}`) }));
  query.addEventListener('input', () => {
    const words = fold(query.value).trim().split(/\s+/).filter(Boolean);
    results.replaceChildren();
    if (!words.length) { status.textContent = 'Type to search.'; return; }
    const matches = docs.filter(page => words.every(word => page.haystack.includes(word)))
      .map(page => ({ page, score: words.filter(word => fold(page.title).includes(word)).length }))
      .sort((a, b) => b.score - a.score);
    status.textContent = matches.length ? `${matches.length} matching ${matches.length === 1 ? 'page' : 'pages'}${matches.length > 30 ? ' (showing 30)' : ''}.` : 'No matching pages.';
    for (const { page } of matches.slice(0, 30)) {
      const heading = page.headings.find(heading => words.every(word => fold(heading.title).includes(word)));
      const li = document.createElement('li');
      const a = document.createElement('a');
      const url = new URL(page.path.split('/').map(encodeURIComponent).join('/'), rootURL);
      if (heading) url.hash = heading.id;
      a.href = url.href;
      a.textContent = heading && heading.title !== page.title ? `${page.title} — ${heading.title}` : page.title;
      const snippet = document.createElement('p');
      const pos = Math.max(0, page.haystack.indexOf(words[0]) - page.title.length - 35);
      snippet.textContent = (pos ? '…' : '') + page.text.slice(pos, pos + 180);
      li.append(a, snippet); results.append(li);
    }
  });
  query.addEventListener('keydown', event => {
    if (event.key === 'ArrowDown') { event.preventDefault(); results.querySelector('a')?.focus(); }
  });
})();
