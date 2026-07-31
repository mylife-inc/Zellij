import Image from 'next/image';
import { assetUrl } from '../content/paths';

/**
 * Screenshots go through `next/image` so guide pages keep the same image
 * budget as marketing pages (spec §11.6). `next/image` applies basePath itself.
 */
export function Screenshot({
  src,
  caption,
  alt,
  width = 1600,
  height = 900,
}: {
  src: string;
  caption?: string;
  alt?: string;
  width?: number;
  height?: number;
}) {
  return (
    <figure className="zellij-screenshot">
      <Image
        src={assetUrl(src)}
        alt={alt ?? caption ?? ''}
        width={width}
        height={height}
        className="zellij-screenshot-image"
      />
      {caption ? <figcaption className="zellij-screenshot-caption">{caption}</figcaption> : null}
    </figure>
  );
}
