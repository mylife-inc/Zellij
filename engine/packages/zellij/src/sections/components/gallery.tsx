import type { SectionProps } from '../types';
import type { GallerySection } from '../schemas';
import { assetUrl, withBasePath } from '../../content/paths';
import { SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { GalleryLightbox, type LightboxImage } from './gallery-lightbox';

/** A grid of images, each openable full-size unless the author opts out. */
export function Gallery({ section, site, defaultMotion }: SectionProps<GallerySection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  const tiles = section.items.map((item, index) => {
    const figure = (
      <figure className="zellij-gallery-figure">
        <SectionImage
          basePath={basePath}
          src={item.image}
          alt={item.alt}
          ratio="4 / 3"
          sizes="(max-width: 700px) 100vw, 33vw"
        />
        {item.caption ? (
          <figcaption className="zellij-gallery-caption">{item.caption}</figcaption>
        ) : null}
      </figure>
    );

    return (
      <Reveal
        as="li"
        key={`${item.image}-${index}`}
        motion={motion}
        index={index}
        className="zellij-gallery-item"
      >
        {section.lightbox ? (
          <button
            type="button"
            className="zellij-gallery-opener"
            data-lightbox-index={index}
            // The alt already names the image; this says what the button does.
            aria-label={item.alt ? `View ${item.alt} full size` : `View image ${index + 1} full size`}
          >
            {figure}
          </button>
        ) : (
          figure
        )}
      </Reveal>
    );
  });

  const grid = (
    <ul className="zellij-gallery-grid" data-columns={section.columns}>
      {tiles}
    </ul>
  );

  // Full-size sources are plain <img> in the dialog, so they need the prefix
  // applied here — next/image would have done it, but cannot size them.
  const images: LightboxImage[] = section.items.map((item) => ({
    src: withBasePath(assetUrl(item.image), basePath),
    alt: item.alt,
    ...(item.caption ? { caption: item.caption } : {}),
  }));

  return (
    <SectionShell
      type="gallery"
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      background={section.background}
      padding={section.padding}
      className="zellij-gallery"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      {section.lightbox ? <GalleryLightbox images={images}>{grid}</GalleryLightbox> : grid}
    </SectionShell>
  );
}
