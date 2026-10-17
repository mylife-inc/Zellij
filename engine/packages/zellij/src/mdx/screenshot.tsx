import Image from 'next/image';
import { assetHref } from '../content/asset-href';

/**
 * Screenshots go through `next/image` so guide pages keep the same image
 * budget as marketing pages (spec §11.6). Its source needs an explicit basePath.
 */
export function Screenshot({
  src,
  caption,
  alt,
  basePath = '',
  width = 1600,
  height = 900,
}: {
  src: string;
  caption?: string;
  alt?: string;
  /** Applied to the source — see `SectionImage` for why this is not optional. */
  basePath?: string;
  width?: number;
  height?: number;
}) {
  return (
    <figure className="zellij-screenshot">
      <Image
        src={assetHref(src, basePath)}
        alt={alt ?? caption ?? ''}
        width={width}
        height={height}
        className="zellij-screenshot-image"
      />
      {caption ? <figcaption className="zellij-screenshot-caption">{caption}</figcaption> : null}
    </figure>
  );
}
