'use client';

import { useLayoutEffect, useRef, useState, type RefObject } from 'react';
import { fittingItems } from './nav-overflow';

/** Measure inert copies so hidden entries still have widths when space grows. */
export function useNavOverflow(
  nav: RefObject<HTMLElement | null>,
  measure: RefObject<HTMLUListElement | null>,
  itemCount: number,
  onChange: (previous: number, next: number) => void,
): { count: number; ready: boolean } {
  const [layout, setLayout] = useState({ count: itemCount, ready: false });
  const previous = useRef(itemCount);

  useLayoutEffect(() => {
    const container = nav.current;
    const probes = measure.current;
    if (!container || !probes) return;
    let frame = 0;
    let disposed = false;
    const update = () => {
      if (disposed || !container.getClientRects().length) return;
      const widths = [...probes.children].map((child) => child.getBoundingClientRect().width);
      const more = widths.pop() ?? 0;
      const gap = Number.parseFloat(getComputedStyle(probes).columnGap) || 0;
      // Leave a small rounding margin so fractional font metrics never overlap.
      const count = fittingItems(widths, container.getBoundingClientRect().width - 1, more, gap);
      if (count !== previous.current) onChange(previous.current, count);
      previous.current = count;
      setLayout((current) => current.ready && current.count === count ? current : { count, ready: true });
    };
    const schedule = () => {
      if (disposed) return;
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(update);
    };
    const observer = new ResizeObserver(schedule);
    observer.observe(container);
    for (const child of probes.children) observer.observe(child);
    // ResizeObserver also catches changes to fonts, theme, active weight, logo
    // dimensions and action widths through the remaining navigation space.
    document.fonts.ready.then(schedule);
    update();
    return () => {
      disposed = true;
      observer.disconnect();
      cancelAnimationFrame(frame);
    };
  }, [nav, measure, itemCount, onChange]);

  return layout;
}
