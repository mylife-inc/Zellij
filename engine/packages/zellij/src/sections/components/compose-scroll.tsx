'use client';

import { useEffect, useRef, useState, type ReactNode } from 'react';
import type { ComposeScroll } from '../compose-schema';
import type { MotionPreset } from '../../config/schema';
import { useResolvedMotion } from '../../motion/use-motion';

const clamp = (n: number) => Math.max(0, Math.min(1, n));
export function ComposeScrollStage({ settings, motion, children }: { settings: ComposeScroll; motion: MotionPreset; children: ReactNode }) {
  const outer = useRef<HTMLDivElement>(null);
  const inner = useRef<HTMLDivElement>(null);
  const resolved = useResolvedMotion(motion);
  const [active, setActive] = useState(false);
  useEffect(() => {
    const box = outer.current, content = inner.current;
    if (!box || !content || resolved !== 'expressive' || typeof ResizeObserver === 'undefined' || typeof IntersectionObserver === 'undefined') {
      setActive(false);
      return;
    }
    let frame = 0, visible = true, enabled = false, distance = 0, top = 0;
    const reset = () => {
      box.style.removeProperty('height');
      content.style.removeProperty('transform');
      content.style.removeProperty('opacity');
      content.style.removeProperty('top');
    };
    const paint = () => {
      frame = 0;
      if (!enabled) return;
      const rect = box.getBoundingClientRect();
      const raw = settings.pin ? (top - rect.top) / distance : (innerHeight - rect.top) / (innerHeight + rect.height);
      const progress = clamp((raw - settings.start) / (settings.end - settings.start));
      const mix = (key: 'y' | 'scale' | 'opacity') => settings.from[key] + (settings.to[key] - settings.from[key]) * progress;
      content.style.transform = `translateY(${mix('y')}px) scale(${mix('scale')})`;
      content.style.opacity = String(mix('opacity'));
    };
    const queue = () => { if (visible && !frame) frame = requestAnimationFrame(paint); };
    const measure = () => {
      content.style.top = 'calc(var(--zellij-nav-height) + 1.5rem)';
      top = parseFloat(getComputedStyle(content).top) || 80;
      enabled = innerWidth >= 760 && box.clientWidth >= 760 && content.offsetHeight < innerHeight - top - 40;
      distance = innerHeight * settings.distance;
      setActive(enabled);
      reset();
      if (enabled) {
        if (settings.pin) box.style.height = `${content.offsetHeight + distance}px`;
        content.style.top = `${top}px`;
        paint();
      }
    };
    const size = new ResizeObserver(measure);
    size.observe(content);
    const intersection = new IntersectionObserver(entries => {
      visible = entries.some(entry => entry.isIntersecting);
      // Settle at an endpoint even after a fast scroll beyond the stage.
      paint();
    });
    intersection.observe(box);
    window.addEventListener('scroll', queue, { passive: true });
    window.addEventListener('resize', measure, { passive: true });
    measure();
    return () => {
      size.disconnect(); intersection.disconnect();
      window.removeEventListener('scroll', queue); window.removeEventListener('resize', measure);
      if (frame) cancelAnimationFrame(frame);
      reset();
    };
  }, [resolved, settings]);
  return <div ref={outer} className="zellij-compose-scroll" data-scroll-active={active} data-scroll-pin={settings.pin}>
    <div ref={inner} className="zellij-compose-scroll-content">{children}</div>
  </div>;
}
