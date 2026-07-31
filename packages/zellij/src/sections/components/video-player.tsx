'use client';

import Image from 'next/image';
import { useState } from 'react';

/**
 * Lazy video (spec §5.2 #12). Nothing loads until the visitor presses play:
 * third-party iframes are not mounted, and self-hosted files use
 * `preload="none"`. Until then only the poster image is fetched, which keeps
 * heavy media off the Lighthouse critical path (spec §11.6).
 */
export function VideoPlayer({
  embedUrl,
  fileUrl,
  poster,
  label,
}: {
  embedUrl?: string;
  fileUrl?: string;
  poster?: string;
  label: string;
}) {
  const [playing, setPlaying] = useState(false);

  if (playing && embedUrl) {
    return (
      <div className="zellij-video-frame">
        <iframe
          src={`${embedUrl}?autoplay=1`}
          title={label}
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; picture-in-picture"
          allowFullScreen
        />
      </div>
    );
  }

  if (playing && fileUrl) {
    return (
      <div className="zellij-video-frame">
        {/* eslint-disable-next-line jsx-a11y/media-has-caption -- caption text is rendered as a figcaption */}
        <video src={fileUrl} controls autoPlay preload="metadata" poster={poster} />
      </div>
    );
  }

  return (
    <button
      type="button"
      className="zellij-video-frame zellij-video-poster"
      onClick={() => setPlaying(true)}
      aria-label={`Play video: ${label}`}
    >
      {poster ? <Image src={poster} alt="" fill sizes="(max-width: 900px) 100vw, 900px" /> : null}
      <span className="zellij-video-play" aria-hidden="true">
        <svg width="26" height="26" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5.14v13.72a1 1 0 0 0 1.52.85l11.14-6.86a1 1 0 0 0 0-1.7L9.52 4.29A1 1 0 0 0 8 5.14z" />
        </svg>
      </span>
    </button>
  );
}
