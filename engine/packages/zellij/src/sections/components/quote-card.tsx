import Image from 'next/image';

/**
 * One quote, rendered the same whether it stands alone or sits in the carousel.
 *
 * Its own module because the two callers are on opposite sides of the server
 * boundary. `Testimonial` is a server component and resolves asset URLs, which
 * means reading the fingerprint manifest from disk; `TestimonialCarousel` is a
 * client component and imports this card. With the card living in the server
 * file, that import pulled `node:fs` into the browser bundle — the build says
 * so, at length, and it is right.
 *
 * So the card takes a model with the URL already resolved, and knows nothing
 * about where assets live.
 */
export interface QuoteModel {
  quote: string;
  author: string;
  role?: string;
  avatarUrl?: string;
}

export function QuoteCard({ quote }: { quote: QuoteModel }) {
  return (
    <figure className="zellij-quote">
      <blockquote className="zellij-quote-text">“{quote.quote}”</blockquote>
      <figcaption className="zellij-quote-author">
        {quote.avatarUrl ? (
          <Image
            className="zellij-quote-avatar"
            src={quote.avatarUrl}
            alt=""
            width={44}
            height={44}
          />
        ) : null}
        <span>
          <span className="zellij-quote-name">{quote.author}</span>
          {quote.role ? <span className="zellij-quote-role">{quote.role}</span> : null}
        </span>
      </figcaption>
    </figure>
  );
}
