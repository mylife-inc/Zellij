'use client';

import { useState } from 'react';
import { QuoteCard, type QuoteModel } from './testimonial';

/**
 * Carousel of up to 5 quotes (spec §5.2 #7). All slides stay in the DOM so the
 * quotes remain crawlable and reachable without JS; only visibility changes.
 */
export function TestimonialCarousel({ quotes }: { quotes: QuoteModel[] }) {
  const [active, setActive] = useState(0);

  const go = (next: number): void => setActive((next + quotes.length) % quotes.length);

  return (
    <div className="zellij-carousel" role="group" aria-roledescription="carousel">
      <div className="zellij-carousel-viewport">
        {quotes.map((quote, index) => (
          <div
            className="zellij-carousel-slide"
            key={quote.author}
            data-active={index === active}
            aria-hidden={index !== active}
            // Keeps links in hidden slides out of the tab order.
            inert={index !== active}
          >
            <QuoteCard quote={quote} />
          </div>
        ))}
      </div>

      <div className="zellij-carousel-controls">
        <button
          type="button"
          className="zellij-icon-button"
          onClick={() => go(active - 1)}
          aria-label="Previous testimonial"
        >
          <Chevron direction="left" />
        </button>

        <div className="zellij-carousel-dots">
          {quotes.map((quote, index) => (
            <button
              key={quote.author}
              type="button"
              className="zellij-carousel-dot"
              data-active={index === active}
              aria-label={`Show testimonial ${index + 1} of ${quotes.length}`}
              aria-current={index === active}
              onClick={() => setActive(index)}
            />
          ))}
        </div>

        <button
          type="button"
          className="zellij-icon-button"
          onClick={() => go(active + 1)}
          aria-label="Next testimonial"
        >
          <Chevron direction="right" />
        </button>
      </div>
    </div>
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
