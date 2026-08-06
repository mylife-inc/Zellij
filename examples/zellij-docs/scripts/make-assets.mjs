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

/*
 * The bar mark. Not trimmed: it is a circle on a transparent ground, and
 * trimming to its ink would crop the antialiased edge and leave it a hair
 * wider than tall — enough for a round thing beside text to look off-centre.
 *
 * `zellij-wordmark.png` is still in this folder and no longer derived from.
 * It was what the bar used, and letters edge to edge is why the sizing there
 * has to be so conservative; a round mark is free to be larger.
 */
const mark = decodePng(path.join(brand, 'zellij-mark.png'));

/*
 * Photographed tilework, for the two images large enough to deserve it.
 *
 * The hero and the themes panel used to be a rosette on a flat gradient, and
 * the gradient was the weakest thing on the page: a colour chosen to be
 * inoffensive behind artwork. These are the artwork.
 */
const heroField = decodePng(path.join(brand, 'tile-hero.png'));
const panelField = decodePng(path.join(brand, 'tile-panel.png'));

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

/*
 * The themes card, which used to be `DUSK` and was the wrong kind of dark.
 *
 * Two things went wrong at once, and the second was caused by the first. Next
 * to three pastel siblings a near-black card does not read as one of a set, it
 * reads as a mistake. And the rosette is drawn with a dark navy outline: on a
 * light field that outline is what gives every petal its edge, and on a field
 * darker than the outline it disappears, leaving the colours to end wherever
 * they end. What looks like a rough picture is a picture with its edges
 * swallowed.
 *
 * Lilac keeps the hue the card had — this is the card about colour, and it was
 * the purple one — at a lightness that lets the outline do its job.
 */
const LILAC = [
  [246, 242, 252],
  [212, 200, 236],
];

/**
 * Scales an image to cover `width`×`height` and centre-crops what overflows.
 *
 * Cover rather than fit: a field with letterboxing down its sides is not a
 * field. `drawOver` clips anything outside the canvas, so the crop is just a
 * negative offset.
 */
function cover(source, width, height) {
  const scale = Math.max(width / source.width, height / source.height);
  const scaled = resize(source, Math.round(source.width * scale), Math.round(source.height * scale));

  return drawOver(
    createImage(width, height),
    scaled,
    Math.round((width - scaled.width) / 2),
    Math.round((height - scaled.height) / 2),
  );
}

/**
 * Rosettes over a field.
 *
 * The field is a photograph of real tilework where one is given, and a gradient
 * otherwise. Both exist because they answer different needs: the large
 * showpieces earn a photograph, and the small cards do not — five detailed
 * mosaics in a card row compete with the words next to them, and at card size
 * the tilework is too small to be read as tilework anyway.
 *
 * Deliberately not a dense tiling of the *rosette*. It carries the "ZELLIJ"
 * wordmark at its centre, so repeating it across a canvas reads as a watermark
 * rather than as tilework — one medallion, or a small cluster, says it once and
 * says it better.
 *
 * `align` shifts the cluster's centre horizontally (0 left, 0.5 centred, 1
 * right), which is how the wide panel keeps its artwork clear of the copy.
 */
function emblem({
  width,
  height,
  from,
  to,
  field,
  scrim = 0,
  scale = 0.74,
  count = 1,
  align = 0.5,
  angle = 0.5,
}) {
  const canvas = field
    ? cover(field, width, height)
    : gradientFill(createImage(width, height), from, to, angle);

  /*
   * A photographed field needs holding back before the rosette goes on it.
   *
   * Tilework is pattern at every scale, and the rosette is a pattern too — laid
   * straight over one another they interlock into noise and the "ZELLIJ" at the
   * centre stops being legible. The gradient fields never needed this because
   * being unremarkable was their whole job.
   *
   * This is what the media-panel section already does with `scrim` at render
   * time, for the same reason and to the same end.
   */
  if (scrim > 0) drawOver(canvas, createImage(width, height, [10, 12, 20, 255]), 0, 0, scrim);

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
  /*
   * The two photographed fields carry no rosette — `count: 0`.
   *
   * Tried it with one. Tilework is pattern at every scale and so is the
   * rosette, and laid over each other they interlock into noise: the "ZELLIJ"
   * at the centre stopped being readable, and the medallion stopped reading as
   * a shape. Holding the field back with a scrim fixed the legibility and cost
   * the photograph its whites, which is most of why it is worth showing.
   *
   * So the field is the artwork here, and the mark in the bar says the name.
   * That is what the rosette was doing on a gradient: standing in for a picture
   * there wasn't one of.
   */
  'hero.png': emblem({ field: heroField, count: 0, width: 1600, height: 900 }),
  // The section applies its own scrim over this one, which is what keeps the
  // copy legible without dulling the image everywhere it is not needed.
  'panel-themes.png': emblem({ field: panelField, count: 0, width: 2000, height: 900 }),
  'og.png': emblem({ width: 1200, height: 630, from: INK[0], to: INK[1], scale: 0.82 }),

  'card-content.png': emblem({ width: 800, height: 600, from: CLAY[0], to: CLAY[1] }),
  'card-sections.png': emblem({ width: 800, height: 600, from: SEA[0], to: SEA[1] }),
  'card-themes.png': emblem({ width: 800, height: 600, from: LILAC[0], to: LILAC[1] }),
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
 * The nav mark, at four times the 28px it renders at so it stays sharp on a
 * retina display. One file for both colour modes: it is a full-colour tile on a
 * transparent ground, and reads on a light or a dark bar.
 *
 * Square, from a square source. Deriving the height from the width the way the
 * wordmark did would let a one-pixel difference in the source through as a
 * visibly oval circle.
 */
encodePng(resize(mark, 112, 112), path.join(root, 'logo.png'));

console.log(`Derived ${Object.keys(IMAGES).length} images and 1 mark from the brand artwork.`);
