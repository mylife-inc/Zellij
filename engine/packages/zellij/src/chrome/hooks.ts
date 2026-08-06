'use client';

import { useEffect, useRef, useState, type RefObject } from 'react';

/** True once the visitor has scrolled past `threshold` pixels. */
export function useScrolledPast(threshold: number): boolean {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = (): void => setScrolled(window.scrollY > threshold);
    onScroll();
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, [threshold]);

  return scrolled;
}

/**
 * Hides the bar while scrolling down and reveals it on scroll up
 * (`nav.hideOnScrollDown`, spec §5.5). Always false when disabled.
 */
export function useHideOnScrollDown(enabled: boolean, threshold: number): boolean {
  const [hidden, setHidden] = useState(false);
  const lastY = useRef(0);

  useEffect(() => {
    if (!enabled) {
      setHidden(false);
      return;
    }

    lastY.current = window.scrollY;

    const onScroll = (): void => {
      const y = window.scrollY;
      const delta = y - lastY.current;

      // Ignore jitter, and never hide the bar near the top of the page.
      if (Math.abs(delta) > 4) {
        setHidden(delta > 0 && y > threshold);
        lastY.current = y;
      }
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, [enabled, threshold]);

  return hidden;
}

/**
 * True only on devices that genuinely hover (mouse/trackpad).
 *
 * Hover-to-open must not be bound on touch devices: there a tap fires
 * pointerenter *and* click, so the menu would open and immediately toggle shut
 * — the classic hover-menu touch bug. Spec §5.5 requires hover-or-click on
 * desktop and tap-to-expand on mobile, which is exactly this split.
 */
export function useHoverCapable(): boolean {
  const [hoverCapable, setHoverCapable] = useState(false);

  useEffect(() => {
    const query = window.matchMedia('(hover: hover) and (pointer: fine)');
    setHoverCapable(query.matches);

    const onChange = (event: MediaQueryListEvent): void => setHoverCapable(event.matches);
    query.addEventListener('change', onChange);
    return () => query.removeEventListener('change', onChange);
  }, []);

  return hoverCapable;
}

/** Locks body scroll while the mobile drawer is open, without layout shift. */
export function useBodyScrollLock(locked: boolean): void {
  useEffect(() => {
    if (!locked) return;

    const { body, documentElement } = document;
    const previousOverflow = body.style.overflow;
    const previousPadding = body.style.paddingRight;
    const scrollbarWidth = window.innerWidth - documentElement.clientWidth;

    body.style.overflow = 'hidden';
    if (scrollbarWidth > 0) body.style.paddingRight = `${scrollbarWidth}px`;

    return () => {
      body.style.overflow = previousOverflow;
      body.style.paddingRight = previousPadding;
    };
  }, [locked]);
}

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])';

/**
 * Makes `container` a modal dialog while `active` (spec §5.5 accessibility).
 *
 * Moves focus in when it opens, keeps Tab inside it, closes on Escape, and puts
 * focus back where it came from on close. `container` must be focusable —
 * `tabIndex={-1}` on the panel is enough.
 */
export function useFocusTrap(
  container: RefObject<HTMLElement | null>,
  active: boolean,
  onEscape?: () => void,
): void {
  useEffect(() => {
    if (!active) return;
    const node = container.current;
    if (!node) return;

    const previouslyFocused = document.activeElement as HTMLElement | null;

    /*
     * Move focus into the dialog.
     *
     * Both overlays declare `aria-modal="true"`, which tells assistive
     * technology that everything outside them is inert — and nothing was making
     * that true. Opening the drawer left focus on the hamburger, so the first
     * Tab went to the next thing in the header and a keyboard user carried on
     * through the page behind the dialog, with the trap below never engaging
     * because it only acts once focus is already inside.
     *
     * The panel takes focus rather than its first link, so a screen reader
     * announces the dialog and its label before reading out a menu item; Tab
     * still reaches the first link immediately after.
     *
     * `preventScroll` because the panel is fixed and full-height: without it a
     * browser may scroll the page underneath to bring it into view, which is
     * the position the page would be restored to on close.
     */
    if (!node.contains(document.activeElement)) node.focus({ preventScroll: true });

    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        onEscape?.();
        return;
      }
      if (event.key !== 'Tab') return;
      // Tab is only meaningful once focus is inside; the document-wide listener
      // would otherwise fight the page's own tab order.
      if (!node.contains(document.activeElement)) return;

      const focusable = [...node.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
        (el) => el.offsetParent !== null,
      );
      if (focusable.length === 0) return;

      const first = focusable[0]!;
      const last = focusable[focusable.length - 1]!;

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    /*
     * Listened for on the document, not on the panel.
     *
     * Bound to the panel, Escape only closed the dialog when focus was already
     * inside it — and nothing puts it there. Opening the drawer leaves focus on
     * the hamburger, which is outside; the keydown then bubbles from the button
     * to the document and never passes the panel at all.
     *
     * It looked like it worked because the test that covers it clicks an
     * accordion inside the drawer first, and in Chromium clicking a button
     * focuses it. WebKit does not focus a button on tap, so the same sequence
     * left focus on the body and the dialog stayed open — but pressing Escape
     * straight after opening never closed it in either engine.
     */
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      previouslyFocused?.focus?.();
    };
  }, [container, active, onEscape]);
}

/** Closes a menu when a click or focus lands outside `container`. */
export function useDismissOnOutside(
  container: RefObject<HTMLElement | null>,
  active: boolean,
  onDismiss: () => void,
): void {
  useEffect(() => {
    if (!active) return;

    const handle = (event: Event): void => {
      const node = container.current;
      if (node && !node.contains(event.target as Node)) onDismiss();
    };

    document.addEventListener('pointerdown', handle);
    document.addEventListener('focusin', handle);
    return () => {
      document.removeEventListener('pointerdown', handle);
      document.removeEventListener('focusin', handle);
    };
  }, [container, active, onDismiss]);
}

/**
 * Scroll spy for in-page anchor links: reports the id of the section currently
 * dominating the viewport (spec §5.5 active-state highlighting).
 */
export function useScrollSpy(enabled: boolean): string | undefined {
  const [activeId, setActiveId] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (!enabled) return;

    const sections = [...document.querySelectorAll<HTMLElement>('[data-zellij-section][id]')];
    if (sections.length === 0) return;

    const visible = new Map<string, number>();

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const id = entry.target.id;
          if (entry.isIntersecting) visible.set(id, entry.intersectionRatio);
          else visible.delete(id);
        }

        let best: string | undefined;
        let bestRatio = 0;
        for (const [id, ratio] of visible) {
          if (ratio >= bestRatio) {
            best = id;
            bestRatio = ratio;
          }
        }
        setActiveId(best);
      },
      { threshold: [0.15, 0.4, 0.75], rootMargin: '-20% 0px -35% 0px' },
    );

    for (const section of sections) observer.observe(section);
    return () => observer.disconnect();
  }, [enabled]);

  return activeId;
}
