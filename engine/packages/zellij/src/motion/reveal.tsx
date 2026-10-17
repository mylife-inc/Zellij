'use client';

import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type ElementType,
  type ReactNode,
} from 'react';

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
  // Taken by name rather than left in `rest`: this component sets `style` for
  // the stagger delay, and a caller's style spread afterwards would replace it.
  style,
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
  style?: CSSProperties;
} & Record<string, unknown>) {
  const ref = useRef<HTMLElement>(null);
  // Server content stays readable if JavaScript is disabled, blocked or late.
  // Only an installed observer may hide an offscreen element for its entrance.
  const [state, setState] = useState<'pending' | 'out' | 'in'>('pending');
  const revealed = useRef(false);

  useEffect(() => {
    const show = (): void => {
      revealed.current = true;
      setState('in');
    };

    // The accessibility override is mandatory and beats config (spec §6.5).
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
    if (
      revealed.current || motion === 'none' || reduced.matches ||
      typeof IntersectionObserver === 'undefined'
    ) {
      show();
      return;
    }

    const node = ref.current;
    if (!node) return;

    // Never hide content the reader already reached while hydration was loading.
    const rect = node.getBoundingClientRect();
    if (
      rect.bottom > 0 && rect.top < window.innerHeight &&
      rect.right > 0 && rect.left < window.innerWidth
    ) {
      show();
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            show();
            observer.disconnect();
          }
        }
      },
      { rootMargin: '0px 0px -10% 0px', threshold: 0.05 },
    );

    observer.observe(node);
    setState('out');
    return () => observer.disconnect();
  }, [motion]);

  return (
    <Tag
      ref={ref}
      className={className}
      data-zellij-reveal={state}
      data-zellij-reveal-direction={direction}
      style={
        index > 0
          ? { transitionDelay: `calc(var(--zellij-motion-stagger) * ${index})`, ...style }
          : style
      }
      {...rest}
    >
      {children}
    </Tag>
  );
}
