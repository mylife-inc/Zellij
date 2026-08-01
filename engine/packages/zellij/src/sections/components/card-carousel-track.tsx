'use client';

import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';

/**
 * The scrolling half of `card-carousel`.
 *
 * Scrolling is the browser's own — `scroll-snap` on a plain overflow container —
 * so the row works with a trackpad, a touch drag, shift+wheel and keyboard
 * scrolling before any of this runs. The buttons are a convenience on top, and
 * the cards themselves are rendered on the server so they are in the HTML
 * whether or not this hydrates.
 */
export function CardCarouselTrack({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  const trackRef = useRef<HTMLDivElement>(null);
  const [atStart, setAtStart] = useState(true);
  const [atEnd, setAtEnd] = useState(true);

  const measure = useCallback(() => {
    const node = trackRef.current;
    if (!node) return;

    const max = node.scrollWidth - node.clientWidth;
    // A pixel of slack: fractional scroll positions and zoom levels mean
    // scrollLeft rarely lands exactly on the bound.
    setAtStart(node.scrollLeft <= 1);
    setAtEnd(node.scrollLeft >= max - 1);
  }, []);

  useEffect(() => {
    const node = trackRef.current;
    if (!node) return;

    measure();
    node.addEventListener('scroll', measure, { passive: true });

    // The bounds change with the viewport, and with late-loading card imagery.
    const observer = new ResizeObserver(measure);
    observer.observe(node);

    return () => {
      node.removeEventListener('scroll', measure);
      observer.disconnect();
    };
  }, [measure]);

  const page = useCallback((direction: 1 | -1): void => {
    const node = trackRef.current;
    if (!node) return;

    // One card plus its gap, so a click always lands on a snap point. Reading
    // the first card rather than assuming a width keeps this correct for every
    // `cardWidth` and at every breakpoint.
    const card = node.querySelector<HTMLElement>('[data-zellij-card]');
    const step = card ? card.offsetWidth + gapOf(node) : node.clientWidth * 0.8;

    node.scrollBy({ left: step * direction, behavior: 'smooth' });
  }, []);

  // Nothing overflows: the row fits, so controls would be inert decoration.
  const scrollable = !(atStart && atEnd);

  return (
    <>
      <div
        ref={trackRef}
        className="zellij-carousel-track"
        role="group"
        aria-label={label}
        // A scrollable region needs to be reachable by keyboard even when the
        // cards inside it are not focusable (WCAG 2.1.1).
        tabIndex={0}
        data-at-start={atStart}
        data-at-end={atEnd}
      >
        {children}
      </div>

      {scrollable ? (
        <div className="zellij-carousel-arrows">
          <button
            type="button"
            className="zellij-icon-button"
            onClick={() => page(-1)}
            disabled={atStart}
            aria-label="Scroll left"
          >
            <Chevron direction="left" />
          </button>
          <button
            type="button"
            className="zellij-icon-button"
            onClick={() => page(1)}
            disabled={atEnd}
            aria-label="Scroll right"
          >
            <Chevron direction="right" />
          </button>
        </div>
      ) : null}
    </>
  );
}

/** The track's column gap in pixels, so a page step clears the space too. */
function gapOf(node: HTMLElement): number {
  const gap = Number.parseFloat(getComputedStyle(node).columnGap);
  return Number.isFinite(gap) ? gap : 0;
}

function Chevron({ direction }: { direction: 'left' | 'right' }) {
  return (
    <svg
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
      <path d={direction === 'left' ? 'm15 18-6-6 6-6' : 'm9 18 6-6-6-6'} />
    </svg>
  );
}
