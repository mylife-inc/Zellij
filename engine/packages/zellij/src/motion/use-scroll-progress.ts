'use client';

import { useEffect, type RefObject } from 'react';

/**
 * Writes an element's scroll progress (0→1) to a CSS custom property, so the
 * animation itself stays in CSS and only ever touches transform and opacity
 * (spec §6.5 — no layout-affecting animation, to protect CLS).
 *
 * Native CSS scroll-driven animations (`animation-timeline`) are used where the
 * browser supports them; this hook is the fallback path the spec calls for, and
 * it also runs the ramps that a pure `view()` timeline cannot express. It is
 * gated by an IntersectionObserver so no scroll work happens for off-screen
 * sections, and reads are batched into a single rAF per frame.
 */
export function useScrollProgress(
  ref: RefObject<HTMLElement | null>,
  enabled: boolean,
  property = '--zellij-progress',
): void {
  useEffect(() => {
    if (!enabled) return;
    const node = ref.current;
    if (!node) return;

    let frame = 0;
    let visible = false;

    const measure = (): void => {
      frame = 0;
      const rect = node.getBoundingClientRect();
      const distance = rect.height - window.innerHeight;

      // Progress through the pinned region: 0 when the top reaches the
      // viewport top, 1 once the element has been scrolled fully past.
      const progress =
        distance <= 0
          ? clamp((window.innerHeight - rect.top) / (window.innerHeight + rect.height))
          : clamp(-rect.top / distance);

      node.style.setProperty(property, progress.toFixed(4));
    };

    const schedule = (force = false): void => {
      if ((!visible && !force) || frame) return;
      frame = requestAnimationFrame(measure);
    };

    const observer = new IntersectionObserver(
      (entries) => {
        visible = entries.some((entry) => entry.isIntersecting);
        // Measure on exit as well as entry. Without this, progress freezes at
        // whatever it was on the last visible frame, so scrolling quickly past
        // a section leaves its visual stuck mid-ramp instead of settled at its
        // end value.
        schedule(true);
      },
      { threshold: 0 },
    );

    // Wrapped rather than passed directly: an Event argument would land in
    // `force` and, being truthy, would defeat the visibility gate.
    const onScrollOrResize = (): void => schedule();

    observer.observe(node);
    window.addEventListener('scroll', onScrollOrResize, { passive: true });
    window.addEventListener('resize', onScrollOrResize, { passive: true });
    measure();

    return () => {
      observer.disconnect();
      window.removeEventListener('scroll', onScrollOrResize);
      window.removeEventListener('resize', onScrollOrResize);
      if (frame) cancelAnimationFrame(frame);
      node.style.removeProperty(property);
    };
  }, [ref, enabled, property]);
}

function clamp(value: number): number {
  return Math.min(1, Math.max(0, value));
}
