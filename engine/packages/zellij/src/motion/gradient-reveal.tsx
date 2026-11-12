'use client';

import { useEffect, useRef, useState } from 'react';
import type { CSSProperties } from 'react';

/** Readable solid text remains underneath the decorative gradient at all times. */
export function GradientReveal({ text, gradient, motion }: { text: string; gradient: string; motion: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  // No JS, late hydration, reduced motion and print all have a complete heading.
  const [progress, setProgress] = useState(1);
  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
    let observer: IntersectionObserver | undefined;
    const configure = () => {
      observer?.disconnect();
      if (motion === 'none' || reduced.matches || typeof IntersectionObserver === 'undefined') {
        setProgress(1);
        return;
      }
      observer = new IntersectionObserver(entries => {
        const entry = entries[0];
        if (entry) setProgress(Math.max(0, Math.min(1, (entry.intersectionRatio - 0.15) / 0.5)));
      }, { threshold: Array.from({ length: 21 }, (_, i) => i / 20) });
      observer.observe(node);
    };
    configure();
    reduced.addEventListener('change', configure);
    return () => { observer?.disconnect(); reduced.removeEventListener('change', configure); };
  }, [motion]);
  return <span ref={ref} className="zellij-gradient-reveal" style={{ '--zellij-gradient-progress': progress } as CSSProperties}>
    <span>{text}</span>
    <span aria-hidden="true" className="zellij-gradient-reveal-paint" style={{ backgroundImage: gradient }}>{text}</span>
  </span>;
}
