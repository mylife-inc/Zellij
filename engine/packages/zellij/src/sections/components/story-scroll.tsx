'use client';

import { useEffect, useRef, useState, type ReactNode } from 'react';
import type { MotionPreset } from '../../config/schema';
import { useResolvedMotion } from '../../motion/use-motion';

const clamp = (value: number) => Math.max(0, Math.min(1, value));

/** Keep all reading content in the DOM; pin only scenes that fit the viewport.
 * A seek in flight finishes before the newest requested frame is applied. */
export function StoryScroll({ label, distance, mediaSide, motion, children }: {
  label: string; distance: number; mediaSide: 'start' | 'end'; motion: MotionPreset; children: ReactNode;
}) {
  const root = useRef<HTMLDivElement>(null);
  const resolved = useResolvedMotion(motion);
  const [active, setActive] = useState(false);
  const [off, setOff] = useState(false);
  useEffect(() => {
    const box = root.current;
    if (!box || off || resolved !== 'expressive' || typeof ResizeObserver === 'undefined' || typeof IntersectionObserver === 'undefined') {
      setActive(false);
      return;
    }
    const scenes = [...box.querySelectorAll<HTMLElement>(':scope > .zellij-story-scenes > .zellij-story-scene')];
    const videos = [...box.querySelectorAll<HTMLVideoElement>('video[data-story-video]')];
    const near = new Set<Element>();
    const targets = new Map<HTMLVideoElement, number>();
    const loaded = new Set<HTMLVideoElement>();
    const forced = matchMedia('(forced-colors: active)');
    const print = matchMedia('print');
    let enabled = false, frame = 0, top = 0, travel = 0;
    const fail = () => setOff(true);
    const seek = (video: HTMLVideoElement) => {
      const target = targets.get(video);
      if (!enabled || document.hidden || target === undefined || video.readyState < 1 || video.seeking) return;
      if (!Number.isFinite(video.duration) || Number(video.dataset.end) > video.duration + 0.05 || Number(video.dataset.start) >= video.duration) { fail(); return; }
      const time = Math.min(target, Math.max(0, video.duration - 0.01));
      if (Math.abs(video.currentTime - time) < 0.04) return;
      try { video.currentTime = time; } catch { fail(); }
    };
    const paint = () => {
      frame = 0;
      if (!enabled || document.hidden) return;
      for (const scene of scenes) {
        const progress = clamp((top - scene.getBoundingClientRect().top) / travel);
        scene.dataset.progress = progress.toFixed(3);
        const video = scene.querySelector<HTMLVideoElement>('video[data-story-video]');
        if (!video) continue;
        targets.set(video, Number(video.dataset.start) + (Number(video.dataset.end) - Number(video.dataset.start)) * progress);
        if (near.has(scene) && !loaded.has(video)) {
          loaded.add(video); video.preload = 'metadata'; video.load();
        }
        if (loaded.has(video)) seek(video);
      }
    };
    const queue = () => { if (near.size && !frame && !document.hidden) frame = requestAnimationFrame(paint); };
    const reset = () => {
      for (const scene of scenes) { scene.style.removeProperty('min-height'); delete scene.dataset.progress; }
      box.style.removeProperty('--zellij-story-top');
      for (const video of videos) { video.pause(); video.controls = true; video.preload = 'none'; }
      targets.clear();
    };
    const measure = () => {
      box.style.setProperty('--zellij-story-top', 'calc(var(--zellij-nav-height) + 1.5rem)');
      // The measuring probe is independent of the current pinning state.
      const probe = box.querySelector<HTMLElement>('.zellij-story-copy');
      top = probe ? parseFloat(getComputedStyle(probe).top) || 80 : 80;
      travel = innerHeight * distance;
      const parts = scenes.map(scene => [...scene.children] as HTMLElement[]);
      const fits = innerWidth >= 760 && box.clientWidth >= 760 && !forced.matches && !print.matches &&
        parts.every(pair => pair.every(part => part.offsetHeight < innerHeight - top - 40));
      if (enabled && !fits) reset();
      enabled = fits; setActive(fits); box.dataset.storyActive = String(fits);
      if (!fits) { box.style.removeProperty('--zellij-story-top'); return; }
      parts.forEach((pair, i) => { scenes[i]!.style.minHeight = `${Math.max(...pair.map(part => part.offsetHeight)) + travel}px`; });
      for (const video of videos) { video.pause(); video.controls = false; }
      paint();
    };
    const mediaReady = (event: Event) => seek(event.currentTarget as HTMLVideoElement);
    const keyboard = (event: KeyboardEvent) => {
      if (enabled && ['Tab', 'ArrowDown', 'ArrowUp', 'PageDown', 'PageUp', 'Home', 'End', ' '].includes(event.key)) setOff(true);
    };
    const focus = () => { if (enabled) setOff(true); };
    const resize = new ResizeObserver(measure);
    resize.observe(box);
    for (const scene of scenes) for (const part of scene.children) resize.observe(part);
    const intersection = new IntersectionObserver(entries => {
      for (const entry of entries) { if (entry.isIntersecting) near.add(entry.target); else near.delete(entry.target); }
      paint(); // A fast scroll must still settle at the endpoint.
    }, {rootMargin: '200px'});
    for (const scene of scenes) intersection.observe(scene);
    for (const video of videos) {
      video.addEventListener('loadedmetadata', mediaReady); video.addEventListener('seeked', mediaReady); video.addEventListener('error', fail);
    }
    window.addEventListener('scroll', queue, {passive: true});
    window.addEventListener('resize', measure, {passive: true});
    window.addEventListener('keydown', keyboard);
    box.addEventListener('focusin', focus);
    document.addEventListener('visibilitychange', queue);
    forced.addEventListener('change', measure); print.addEventListener('change', measure);
    measure();
    return () => {
      enabled = false; resize.disconnect(); intersection.disconnect();
      window.removeEventListener('scroll', queue); window.removeEventListener('resize', measure); window.removeEventListener('keydown', keyboard);
      box.removeEventListener('focusin', focus); document.removeEventListener('visibilitychange', queue);
      forced.removeEventListener('change', measure); print.removeEventListener('change', measure);
      for (const video of videos) {
        video.removeEventListener('loadedmetadata', mediaReady); video.removeEventListener('seeked', mediaReady); video.removeEventListener('error', fail);
      }
      if (frame) cancelAnimationFrame(frame);
      reset(); box.dataset.storyActive = 'false';
    };
  }, [resolved, off, distance]);
  return <div ref={root} className="zellij-compose-story" role="group" aria-label={label} data-story-active={active} data-media-side={mediaSide}>
    <button type="button" className="zellij-story-reading" hidden={!active && !off} aria-disabled={off} onClick={() => setOff(true)}>
      {off ? 'Scroll effects off' : 'Read without scroll effects'}
    </button>
    <div className="zellij-story-scenes">{children}</div>
  </div>;
}
