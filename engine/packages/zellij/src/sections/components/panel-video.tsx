'use client';

import { useEffect, useRef, useState } from 'react';

/**
 * Background video for a media panel.
 *
 * Autoplay is decided in JavaScript rather than by the `autoplay` attribute:
 * `prefers-reduced-motion` must win, and an attribute would have the video
 * moving before any effect could stop it. Without motion the poster stands in,
 * which is what the still frame is for.
 */
export function PanelVideo({
  src,
  poster,
  label,
}: {
  src: string;
  poster?: string;
  label: string;
}) {
  const ref = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');

    const apply = (): void => {
      if (reduced.matches) {
        node.pause();
        node.currentTime = 0;
        setPlaying(false);
        return;
      }
      // A rejected play() is normal — a background video is not worth surfacing
      // an error for, and the poster is already showing.
      void node.play().then(
        () => setPlaying(true),
        () => setPlaying(false),
      );
    };

    apply();
    reduced.addEventListener('change', apply);
    return () => reduced.removeEventListener('change', apply);
  }, []);

  return (
    <video
      ref={ref}
      className="zellij-panel-video"
      data-playing={playing}
      {...(poster ? { poster } : {})}
      src={src}
      muted
      loop
      playsInline
      preload="metadata"
      aria-label={label || undefined}
      // Decorative background media: no controls, and nothing for a screen
      // reader unless the author gave it a label.
      {...(label ? {} : { 'aria-hidden': true })}
    />
  );
}
