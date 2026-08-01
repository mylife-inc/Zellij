'use client';

import { useEffect, useState } from 'react';
import type { MotionPreset } from '../config/schema';

/**
 * Resolves the motion preset actually in force.
 *
 * `prefers-reduced-motion` collapses any preset to `none`, and that override is
 * mandatory regardless of configuration (spec §6.5). Every motion behaviour in
 * the engine goes through this hook so the rule cannot be forgotten in one
 * component.
 *
 * Returns `none` until mounted, so nothing animates during hydration and the
 * server-rendered markup is the resting state.
 */
export function useResolvedMotion(preset: MotionPreset): MotionPreset {
  const [resolved, setResolved] = useState<MotionPreset>('none');

  useEffect(() => {
    if (preset === 'none') {
      setResolved('none');
      return;
    }

    const query = window.matchMedia('(prefers-reduced-motion: reduce)');
    const apply = (): void => setResolved(query.matches ? 'none' : preset);

    apply();
    query.addEventListener('change', apply);
    return () => query.removeEventListener('change', apply);
  }, [preset]);

  return resolved;
}

/** True only when scroll-driven effects should run. */
export function useExpressive(preset: MotionPreset): boolean {
  return useResolvedMotion(preset) === 'expressive';
}
