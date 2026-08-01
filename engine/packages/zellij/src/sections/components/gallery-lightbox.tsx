'use client';

import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import type React from 'react';

export interface LightboxImage {
  src: string;
  alt: string;
  caption?: string;
}

/**
 * Click-to-enlarge for a gallery, on the native `<dialog>` element.
 *
 * `showModal()` brings the focus trap, the inert backdrop and Escape-to-close
 * with it, so none of that is reimplemented here — only arrow-key paging is,
 * which the platform has no opinion about.
 */
export function GalleryLightbox({
  images,
  children,
}: {
  images: LightboxImage[];
  /**
   * The server-rendered tiles. Each opener carries `data-lightbox-index`; a
   * render prop would be the obvious API but a function cannot cross the
   * server/client boundary, so the click is delegated instead and the tiles
   * stay server components.
   */
  children: ReactNode;
}) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [index, setIndex] = useState(0);

  const step = useCallback(
    (delta: number) => setIndex((current) => (current + delta + images.length) % images.length),
    [images.length],
  );

  useEffect(() => {
    const node = dialogRef.current;
    if (!node) return;

    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'ArrowRight') {
        event.preventDefault();
        step(1);
      } else if (event.key === 'ArrowLeft') {
        event.preventDefault();
        step(-1);
      }
    };

    node.addEventListener('keydown', onKeyDown);
    return () => node.removeEventListener('keydown', onKeyDown);
  }, [step]);

  const current = images[index];

  const onTileClick = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    const opener = (event.target as HTMLElement).closest<HTMLElement>('[data-lightbox-index]');
    if (!opener) return;

    const next = Number(opener.dataset['lightboxIndex']);
    if (!Number.isInteger(next)) return;

    setIndex(next);
    dialogRef.current?.showModal();
  }, []);

  return (
    <>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events --
          the openers inside are buttons, so keyboard activation already works
          and reaches this handler as a click. */}
      <div onClick={onTileClick}>{children}</div>

      <dialog
        ref={dialogRef}
        className="zellij-lightbox"
        aria-label="Image viewer"
        // Clicking the backdrop closes: the dialog element itself is the
        // backdrop, and the figure inside stops the event.
        onClick={(event) => {
          if (event.target === dialogRef.current) dialogRef.current?.close();
        }}
      >
        {current ? (
          <figure className="zellij-lightbox-figure">
            {/* A plain <img>: the source is already a public, prefixed URL and
                the size here is "as large as fits", which next/image cannot
                express without knowing the viewport at build time. */}
            <img className="zellij-lightbox-image" src={current.src} alt={current.alt} />
            {current.caption ? (
              <figcaption className="zellij-lightbox-caption">{current.caption}</figcaption>
            ) : null}
          </figure>
        ) : null}

        <div className="zellij-lightbox-controls">
          <button
            type="button"
            className="zellij-icon-button"
            onClick={() => step(-1)}
            aria-label="Previous image"
          >
            <Chevron direction="left" />
          </button>
          <p className="zellij-lightbox-count" aria-live="polite">
            {index + 1} of {images.length}
          </p>
          <button
            type="button"
            className="zellij-icon-button"
            onClick={() => step(1)}
            aria-label="Next image"
          >
            <Chevron direction="right" />
          </button>
        </div>

        <button
          type="button"
          className="zellij-lightbox-close"
          onClick={() => dialogRef.current?.close()}
          aria-label="Close image viewer"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            aria-hidden="true"
          >
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </dialog>
    </>
  );
}

function Chevron({ direction }: { direction: 'left' | 'right' }) {
  return (
    <svg
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={direction === 'left' ? 'm15 18-6-6 6-6' : 'm9 18 6-6-6-6'} />
    </svg>
  );
}
