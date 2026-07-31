'use client';

import { useRef, type ReactNode } from 'react';
import type { MotionPreset } from '../config/schema';
import { useResolvedMotion } from './use-motion';
import { useScrollProgress } from './use-scroll-progress';

/**
 * Sticky-scroll stage — the signature "learn more" treatment (spec §6.5): the
 * visual pins while the accompanying copy scrolls past, with a scroll-linked
 * scale and opacity ramp.
 *
 * Only engages under `expressive`. Under any other preset — or reduced motion —
 * it renders its children as an ordinary block, so the section is identical to
 * the `subtle` version and no extra height is introduced.
 */
export function ScrollStage({
  motion,
  children,
  className,
  enabled = true,
}: {
  motion: MotionPreset;
  children: ReactNode;
  className?: string;
  /**
   * Whether pinning is worth its cost here. Pinning reserves roughly two
   * viewports of scroll, which only pays off when there is a visual to ramp —
   * a text-only panel would make readers scroll past a long stretch of nothing.
   */
  enabled?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const resolved = useResolvedMotion(motion);
  const expressive = resolved === 'expressive' && enabled;

  useScrollProgress(ref, expressive);

  if (!expressive) return <div className={className}>{children}</div>;

  return (
    <div className={className} ref={ref} data-zellij-stage="pinned">
      <div className="zellij-stage-sticky">{children}</div>
    </div>
  );
}

/**
 * Gentle scroll-linked parallax for a hero visual (spec §6.5). Same contract:
 * inert unless `expressive` is actually in force.
 */
export function Parallax({
  motion,
  children,
  className,
}: {
  motion: MotionPreset;
  children: ReactNode;
  className?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const resolved = useResolvedMotion(motion);
  const expressive = resolved === 'expressive';

  useScrollProgress(ref, expressive);

  return (
    <div className={className} ref={ref} data-zellij-parallax={expressive ? 'on' : 'off'}>
      {children}
    </div>
  );
}
