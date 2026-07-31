'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { useFocusTrap } from './hooks';
import { THEME_NAME_STORAGE_KEY } from './theme-toggle';
import type { ThemeOption } from './nav-model';

/**
 * Theme picker for the nav bar.
 *
 * Switching is a single attribute write on <html>: every theme's tokens are in
 * the page (see `emitThemeCss` under `nav.themeSwitcher`), so the whole site
 * repaints from CSS with no re-render and no request.
 *
 * The swatches carry `data-zellij-theme` themselves, which means each one is
 * painted by the very rules it represents — the preview cannot drift from the
 * real thing, because it *is* the real thing.
 */
export function ThemeSwitcher({
  themes,
  open,
  onOpen,
  onToggle,
  onClose,
  onHover,
}: {
  themes: ThemeOption[];
  open: boolean;
  onOpen: () => void;
  onToggle: () => void;
  onClose: () => void;
  /** Present only on hover-capable pointers, per the bar's hover intent. */
  onHover?: () => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState<string | undefined>(undefined);

  useFocusTrap(panelRef, open, onClose);

  useEffect(() => {
    setActive(document.documentElement.getAttribute('data-zellij-theme') ?? undefined);
  }, []);

  const choose = useCallback(
    (name: string) => {
      document.documentElement.setAttribute('data-zellij-theme', name);
      setActive(name);
      try {
        localStorage.setItem(THEME_NAME_STORAGE_KEY, name);
      } catch {
        // Storage unavailable — the choice still applies for this page view.
      }
      onClose();
    },
    [onClose],
  );

  return (
    <div
      className="zellij-theme-switcher"
      {...(onHover ? { onPointerEnter: onHover } : {})}
    >
      <button
        type="button"
        className="zellij-icon-button"
        aria-label="Change theme"
        aria-expanded={open}
        aria-haspopup="true"
        aria-controls="zellij-theme-panel"
        onClick={onToggle}
        onKeyDown={(event) => {
          if (event.key === 'ArrowDown') {
            event.preventDefault();
            onOpen();
            requestAnimationFrame(() => {
              panelRef.current?.querySelector<HTMLElement>('button')?.focus();
            });
          }
        }}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <circle cx="13.5" cy="6.5" r=".5" fill="currentColor" />
          <circle cx="17.5" cy="10.5" r=".5" fill="currentColor" />
          <circle cx="8.5" cy="7.5" r=".5" fill="currentColor" />
          <circle cx="6.5" cy="12.5" r=".5" fill="currentColor" />
          <path d="M12 2a10 10 0 0 0 0 20 2.5 2.5 0 0 0 2-4 2.5 2.5 0 0 1 2-4h2a4 4 0 0 0 4-4 10 10 0 0 0-10-8z" />
        </svg>
      </button>

      {open ? (
        <div className="zellij-theme-panel" id="zellij-theme-panel" ref={panelRef}>
          <p className="zellij-theme-panel-heading">Theme</p>
          <ul className="zellij-theme-list">
            {themes.map((theme) => (
              <li key={theme.name}>
                <button
                  type="button"
                  className="zellij-theme-option"
                  data-selected={active === theme.name}
                  aria-pressed={active === theme.name}
                  onClick={() => choose(theme.name)}
                >
                  {/*
                    The swatch declares the theme it previews, so the emitted
                    rules for that theme paint it — including in dark mode.
                  */}
                  <span className="zellij-theme-swatch" data-zellij-theme={theme.name}>
                    <span className="zellij-theme-swatch-accent" />
                    <span className="zellij-theme-swatch-text">Aa</span>
                  </span>
                  <span className="zellij-theme-option-copy">
                    <span className="zellij-theme-option-name">{theme.label}</span>
                    <span className="zellij-theme-option-description">{theme.description}</span>
                  </span>
                  {active === theme.name ? (
                    <svg
                      className="zellij-theme-check"
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M20 6 9 17l-5-5" />
                    </svg>
                  ) : null}
                </button>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
