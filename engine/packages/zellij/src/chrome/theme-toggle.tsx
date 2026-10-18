'use client';

import { useEffect, useState } from 'react';
import { THEME_STORAGE_KEY } from './theme-preferences';

type Mode = 'light' | 'dark' | 'system';

function resolveNext(current: Mode): Mode {
  if (current === 'system') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    return prefersDark ? 'light' : 'dark';
  }
  return current === 'dark' ? 'light' : 'dark';
}

export function ThemeToggle() {
  const [mode, setMode] = useState<Mode>('system');
  const [systemDark, setSystemDark] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    const current = document.documentElement.getAttribute('data-zellij-mode');
    if (current === 'light' || current === 'dark' || current === 'system') setMode(current);
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const update = () => setSystemDark(media.matches);
    update();
    media.addEventListener('change', update);
    setMounted(true);
    return () => media.removeEventListener('change', update);
  }, []);

  const toggle = (): void => {
    const next = resolveNext(mode);
    document.documentElement.setAttribute('data-zellij-mode', next);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch {
      // Private browsing or storage disabled — the toggle still works per-page.
    }
    setMode(next);
  };

  // Until mounted we cannot know the effective mode, so keep the label neutral
  // rather than rendering a claim that may be wrong.
  const label = mounted
    ? mode === 'dark' || (mode === 'system' && systemDark)
      ? 'Switch to light theme'
      : 'Switch to dark theme'
    : 'Toggle theme';

  return (
    <button
      type="button"
      className="zellij-icon-button"
      onClick={toggle}
      aria-label={label}
      title={label}
    >
      <svg
        className="zellij-theme-icon zellij-theme-icon-sun"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
      </svg>
      <svg
        className="zellij-theme-icon zellij-theme-icon-moon"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" />
      </svg>
    </button>
  );
}
