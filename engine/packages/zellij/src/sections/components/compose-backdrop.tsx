'use client';

import { useEffect, useRef, useState, type ReactNode } from 'react';
import type { ComposeBackdrop } from '../compose-schema';

/** Backdrops are decorative. A video never starts merely because it is visible. */
export function ComposeBackdropMedia({ backdrop, children }: { backdrop: ComposeBackdrop; children: ReactNode }) {
  const ref = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);
  const [enhanced, setEnhanced] = useState(false);
  useEffect(() => {
    setEnhanced(true);
    const reduce = matchMedia('(prefers-reduced-motion: reduce)');
    const stop = () => { ref.current?.pause(); setPlaying(false); };
    const change = () => { if (reduce.matches) stop(); };
    reduce.addEventListener('change', change);
    window.addEventListener('beforeprint', stop);
    return () => { reduce.removeEventListener('change', change); window.removeEventListener('beforeprint', stop); };
  }, []);
  const poster = backdrop.kind === 'image' ? backdrop.src : backdrop.poster;
  return <>
    <div className="zellij-compose-backdrop">
      <img src={poster} alt={backdrop.kind === 'image' ? backdrop.alt : ''} loading="lazy" />
      {backdrop.kind === 'video' ? <video ref={ref} src={backdrop.src} poster={backdrop.poster}
        muted loop playsInline preload="none" aria-hidden="true" onPause={() => setPlaying(false)} onPlay={() => setPlaying(true)} /> : null}
    </div>
    {children}
    {backdrop.kind === 'video' ? <button hidden={!enhanced} type="button" className="zellij-compose-backdrop-control"
      onClick={() => {
        const video = ref.current;
        if (!video) return;
        if (video.paused) void video.play().catch(() => setPlaying(false));
        else video.pause();
      }} aria-label={`${playing ? 'Pause' : 'Play'} background video: ${backdrop.label}`} aria-pressed={playing}>
      {playing ? 'Pause background' : 'Play background'}
    </button> : null}
  </>;
}
