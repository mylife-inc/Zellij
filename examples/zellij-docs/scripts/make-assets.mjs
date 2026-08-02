/**
 * Derives the docs site's imagery from the brand artwork in `content/assets/brand`.
 *
 * The source files are real designed assets and are committed. Everything else
 * — the nav wordmark at bar height, the fields the rosette sits on — is
 * generated here, so sizes and palettes stay in one place and no image
 * dependency is added to a project that has none.
 */
import { mkdir, rm } from 'node:fs/promises';
import path from 'node:path';

import {
  createImage,
  decodePng,
  drawOver,
  encodePng,
  gradientFill,
  resize,
  trim,
} from '../../tools/image.mjs';

const root = path.resolve(import.meta.dirname, '..', 'content', 'assets');
const brand = path.join(root, 'brand');

const rosette = trim(decodePng(path.join(brand, 'zellij-rosette.png')));
const wordmark = trim(decodePng(path.join(brand, 'zellij-wordmark.png')));

/* ------------------------------------------------------------------ fields */

const INK = [
  [16, 18, 28],
  [34, 40, 66],
];
const SEA = [
  [232, 242, 240],
  [178, 212, 206],
];
const CLAY = [
  [250, 242, 234],
  [232, 206, 182],
];
const DUSK = [
  [22, 20, 34],
  [58, 42, 78],
];
const SLATE = [
  [238, 241, 246],
  [198, 209, 226],
];

/**
 * Rosettes over a gradient field.
 *
 * Deliberately not a dense tiling. The rosette carries the "ZELLIJ" wordmark at
 * its centre, so repeating it across a canvas reads as a watermark rather than
 * as tilework — one medallion, or a small cluster, says it once and says it
 * better.
 *
 * `align` shifts the cluster's centre horizontally (0 left, 0.5 centred, 1
 * right), which is how the wide panel keeps its artwork clear of the copy.
 */
function emblem({ width, height, from, to, scale = 0.74, count = 1, align = 0.5, angle = 0.5 }) {
  const canvas = gradientFill(createImage(width, height), from, to, angle);

  const size = Math.round(height * scale);
  const tile = resize(rosette, size, Math.round((size * rosette.height) / rosette.width));

  // Overlapping slightly interlocks the points, the way laid tile does.
  const step = tile.width * 0.82;
  const spread = step * (count - 1);
  const centre = tile.width / 2 + (width - tile.width - spread) * align;

  for (let i = 0; i < count; i += 1) {
    drawOver(
      canvas,
      tile,
      Math.round(centre + i * step - tile.width / 2),
      Math.round((height - tile.height) / 2),
    );
  }

  return canvas;
}

/* ------------------------------------------------------------------ output */

// Clear what the previous generator left behind; otherwise it lingers in
// public/ after the next asset sync.
await rm(path.join(root, 'tile'), { recursive: true, force: true });
for (const stale of ['hero-mosaic.png', 'logo.svg', 'logo-dark.svg']) {
  await rm(path.join(root, stale), { force: true });
}
await mkdir(root, { recursive: true });

const IMAGES = {
  'hero.png': emblem({ width: 1600, height: 900, from: INK[0], to: INK[1], scale: 0.82 }),
  'panel-themes.png': emblem({
    width: 2000,
    height: 900,
    from: DUSK[0],
    to: DUSK[1],
    scale: 0.78,
    count: 2,
    // The copy sits over the left of this panel, so the artwork keeps right.
    align: 0.86,
  }),
  'og.png': emblem({ width: 1200, height: 630, from: INK[0], to: INK[1], scale: 0.82 }),

  'card-content.png': emblem({ width: 800, height: 600, from: CLAY[0], to: CLAY[1] }),
  'card-sections.png': emblem({ width: 800, height: 600, from: SEA[0], to: SEA[1] }),
  'card-themes.png': emblem({ width: 800, height: 600, from: DUSK[0], to: DUSK[1] }),
  'card-guides.png': emblem({ width: 800, height: 600, from: SLATE[0], to: SLATE[1] }),
  'card-cli.png': emblem({ width: 800, height: 600, from: INK[0], to: INK[1] }),

  'bento-model.png': emblem({ width: 1200, height: 750, from: SEA[0], to: SEA[1], scale: 0.8 }),
  'bento-themes.png': emblem({ width: 1200, height: 750, from: DUSK[0], to: DUSK[1], scale: 0.8 }),

  /*
   * Artwork for the documentation examples under `content/examples`. A gallery
   * needs six images and a team grid three before either shows what it does, so
   * they are derived here alongside everything else rather than committed.
   */
  'example/tile-1.png': emblem({ width: 900, height: 900, from: SEA[0], to: SEA[1] }),
  'example/tile-2.png': emblem({ width: 900, height: 900, from: CLAY[0], to: CLAY[1], angle: 0.2 }),
  'example/tile-3.png': emblem({ width: 900, height: 900, from: DUSK[0], to: DUSK[1], angle: 0.8 }),
  'example/tile-4.png': emblem({ width: 900, height: 900, from: SLATE[0], to: SLATE[1] }),
  'example/tile-5.png': emblem({ width: 900, height: 900, from: INK[0], to: INK[1], angle: 0.3 }),
  'example/tile-6.png': emblem({ width: 900, height: 900, from: SEA[1], to: SEA[0], angle: 0.6 }),

  'example/person-1.png': emblem({ width: 600, height: 600, from: SEA[0], to: SEA[1], scale: 0.6 }),
  'example/person-2.png': emblem({ width: 600, height: 600, from: CLAY[0], to: CLAY[1], scale: 0.6 }),
  'example/person-3.png': emblem({ width: 600, height: 600, from: DUSK[0], to: DUSK[1], scale: 0.6 }),

  'example/wide-1.png': emblem({ width: 1600, height: 900, from: DUSK[0], to: DUSK[1], scale: 0.7 }),
  'example/wide-2.png': emblem({ width: 1600, height: 900, from: SLATE[0], to: SLATE[1], scale: 0.7 }),
  'example/poster.png': emblem({ width: 1600, height: 900, from: INK[0], to: INK[1], scale: 0.7 }),

  /*
   * Promo tiles carry their copy over the artwork, so the emblem is pushed to
   * one side and scaled down. Centred artwork behind centred copy is what the
   * first attempt looked like, and neither was readable.
   */
  'example/promo-wide.png': emblem({
    width: 2000,
    height: 900,
    from: INK[0],
    to: INK[1],
    scale: 0.42,
    align: 0.95,
  }),
  // `count: 0` leaves the field bare. These tiles are close to square, so the
  // centred copy reaches every part of them and there is nowhere an emblem can
  // sit without landing under a word.
  'example/promo-1.png': emblem({ width: 1000, height: 900, from: DUSK[0], to: DUSK[1], count: 0, angle: 0.3 }),
  'example/promo-2.png': emblem({ width: 1000, height: 900, from: SEA[0], to: SEA[1], count: 0, angle: 0.7 }),

  /* Stand-in customer wordmarks for the logo strip. */
  'example/mark-1.png': emblem({ width: 480, height: 200, from: SLATE[0], to: SLATE[0], scale: 0.7 }),
  'example/mark-2.png': emblem({ width: 480, height: 200, from: SLATE[0], to: SLATE[0], scale: 0.6 }),
  'example/mark-3.png': emblem({ width: 480, height: 200, from: SLATE[0], to: SLATE[0], scale: 0.8 }),
  'example/mark-4.png': emblem({ width: 480, height: 200, from: SLATE[0], to: SLATE[0], scale: 0.5 }),
};

await mkdir(path.join(root, 'example'), { recursive: true });

for (const [name, image] of Object.entries(IMAGES)) {
  encodePng(image, path.join(root, name));
}

/*
 * The nav wordmark, at four times the 24px it renders at so it stays sharp on a
 * retina display. One file for both colour modes: the mark's greens, teals and
 * blues sit in the middle of the range and read on a light or a dark bar.
 */
encodePng(
  resize(wordmark, Math.round((96 * wordmark.width) / wordmark.height), 96),
  path.join(root, 'logo.png'),
);

console.log(`Derived ${Object.keys(IMAGES).length} images and 1 wordmark from the brand artwork.`);
