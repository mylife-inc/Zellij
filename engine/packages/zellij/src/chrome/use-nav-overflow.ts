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
    const bar = container.closest<HTMLElement>('.zellij-nav-inner');
    const content = container.closest<HTMLElement>('.zellij-nav-content');
    const brand = content?.querySelector<HTMLElement>('.zellij-brand');
    const company = bar?.querySelector<HTMLElement>('.zellij-nav-company');
    const actions = bar?.querySelector<HTMLElement>('.zellij-nav-actions');
    let frame = 0;
    let disposed = false;
    const update = () => {
      if (disposed || !container.getClientRects().length) return;
      const widths = [...probes.children].map((child) => child.getBoundingClientRect().width);
      const more = widths.pop() ?? 0;
      const gap = Number.parseFloat(getComputedStyle(probes).columnGap) || 0;
      let available = container.getBoundingClientRect().width;
      if (bar && content && brand && actions) {
        const style = getComputedStyle(bar);
        const outerGap = Number.parseFloat(style.columnGap) || 0;
        const companyWidth = company?.getBoundingClientRect().width ?? 0;
        const actionsWidth = actions.getBoundingClientRect().width;
        const centered = bar.dataset.align === 'center';
        const edge = Math.max(companyWidth, actionsWidth);
        // Equal side reservations keep the product/menu group at the viewport
        // center even when utilities are much wider than the company mark.
        bar.style.setProperty('--zellij-nav-edge', `${edge}px`);
        const innerWidth = bar.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
        const contentWidth = innerWidth - (centered ? 2 * edge : companyWidth + actionsWidth)
          - outerGap * (centered || companyWidth ? 2 : 1);
        const contentGap = Number.parseFloat(getComputedStyle(content).columnGap) || 0;
        available = contentWidth - brand.getBoundingClientRect().width - contentGap;
        container.style.setProperty('--zellij-nav-min-width', `${itemCount ? more : 0}px`);
      }
      // Budget from the whole header, not the currently visible links: a
      // shrink-wrapped centered group must still restore hidden items on growth.
      const count = fittingItems(widths, Math.max(0, available - 1), more, gap);
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
    for (const element of [bar, brand, company, actions]) {
      if (element) observer.observe(element);
    }
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
