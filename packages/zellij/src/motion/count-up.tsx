'use client';

import { useEffect, useRef, useState } from 'react';
import type { MotionPreset } from '../config/schema';
import { useResolvedMotion } from './use-motion';

/** Splits "99.99%" or "12k" into a number and its trailing unit. */
function parseValue(value: string): { number: number; suffix: string; decimals: number } | undefined {
  const match = /^(-?[\d,]*\.?\d+)(.*)$/.exec(value.trim());
  if (!match) return undefined;

  const numeric = match[1]!.replace(/,/g, '');
  const parsed = Number.parseFloat(numeric);
  if (!Number.isFinite(parsed)) return undefined;

  const decimals = numeric.includes('.') ? numeric.split('.')[1]!.length : 0;
  return { number: parsed, suffix: match[2] ?? '', decimals };
}

/**
 * Counts a stat up when it enters the viewport, under the `expressive` preset
 * only (spec §6.5). Non-numeric values, other presets, and reduced-motion all
 * render the literal string, so the number is never wrong or missing.
 */
export function CountUp({
  value,
  motion,
  prefix = '',
  suffix = '',
}: {
  value: string;
  motion: MotionPreset;
  prefix?: string;
  suffix?: string;
}) {
  // The hook is the single place reduced-motion collapses a preset (spec §6.5).
  const resolved = useResolvedMotion(motion);
  const parsed = resolved === 'expressive' ? parseValue(value) : undefined;
  const ref = useRef<HTMLSpanElement>(null);
  const [display, setDisplay] = useState<string>(value);

  useEffect(() => {
    if (!parsed) {
      setDisplay(value);
      return;
    }

    const node = ref.current;
    if (!node) return;

    let frame = 0;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        observer.disconnect();

        const duration = 1200;
        const start = performance.now();

        const tick = (now: number): void => {
          const t = Math.min((now - start) / duration, 1);
          // easeOutExpo — fast start, settled finish.
          const eased = t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
          const current = parsed.number * eased;

          setDisplay(
            `${current.toFixed(parsed.decimals)}${parsed.suffix}`,
          );

          if (t < 1) frame = requestAnimationFrame(tick);
          else setDisplay(value);
        };

        frame = requestAnimationFrame(tick);
      },
      { threshold: 0.4 },
    );

    observer.observe(node);
    return () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
    };
  }, [parsed, value]);

  return (
    <span ref={ref}>
      {prefix}
      {display}
      {suffix}
    </span>
  );
}
