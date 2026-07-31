'use client';

import { useEffect, useRef, useState, type ElementType, type ReactNode } from 'react';

/**
 * Entrance reveal for the `subtle` preset (spec §6.5): children fade and
 * translate in as they enter the viewport, with a small stagger between
 * siblings.
 *
 * Transform/opacity only — never layout-affecting properties — so revealing
 * cannot contribute to CLS (spec §6.5, §11.6). Under `motion: none` or
 * `prefers-reduced-motion` nothing is animated and content renders visible from
 * the first paint.
 */
export function Reveal({
  children,
  as: Tag = 'div',
  motion = 'subtle',
  index = 0,
  className,
  direction = 'up',
  // Forwarded to the element so callers can pass data-* hooks (e.g. the
  // section header's data-align) without needing a wrapper element.
  ...rest
}: {
  children: ReactNode;
  as?: ElementType;
  motion?: 'none' | 'subtle' | 'expressive';
  /** Position among siblings, used for stagger. */
  index?: number;
  className?: string;
  direction?: 'up' | 'left' | 'right';
} & Record<string, unknown>) {
  const ref = useRef<HTMLElement>(null);
  const [revealed, setRevealed] = useState(motion === 'none');

  useEffect(() => {
    if (motion === 'none') {
      setRevealed(true);
      return;
    }

    // The accessibility override is mandatory and beats config (spec §6.5).
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
    if (reduced.matches) {
      setRevealed(true);
      return;
    }

    const node = ref.current;
    if (!node) return;

    // Already in view on load (above the fold): reveal without waiting.
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setRevealed(true);
            observer.disconnect();
          }
        }
      },
      { rootMargin: '0px 0px -10% 0px', threshold: 0.05 },
    );

    observer.observe(node);
    return () => observer.disconnect();
  }, [motion]);

  return (
    <Tag
      ref={ref}
      className={className}
      data-zellij-reveal={revealed ? 'in' : 'out'}
      data-zellij-reveal-direction={direction}
      style={index > 0 ? { transitionDelay: `calc(var(--zellij-motion-stagger) * ${index})` } : undefined}
      {...rest}
    >
      {children}
    </Tag>
  );
}
