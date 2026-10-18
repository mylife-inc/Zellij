export const THEME_STORAGE_KEY = 'zellij:mode';
export const THEME_NAME_STORAGE_KEY = 'zellij:theme';

/**
 * Restore only preferences this site lets visitors change. The server supplies
 * enabled controls and emitted theme names as escaped script data attributes;
 * no authored value is interpolated into executable source.
 * This module is shared by the server shell and client controls.
 */
export const themeInitScript = `
(function(){try{
  var d = document.documentElement;
  var s = document.currentScript;
  if (!s) return;
  if (s.getAttribute('data-mode-toggle') === 'true') {
    var m = localStorage.getItem('${THEME_STORAGE_KEY}');
    if (m === 'light' || m === 'dark' || m === 'system') d.setAttribute('data-zellij-mode', m);
  }
  var names = s.getAttribute('data-themes');
  var t = names ? localStorage.getItem('${THEME_NAME_STORAGE_KEY}') : null;
  if (t && names.split(' ').indexOf(t) !== -1) d.setAttribute('data-zellij-theme', t);
}catch(e){}})();
`.trim();
