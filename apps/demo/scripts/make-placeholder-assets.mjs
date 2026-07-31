/**
 * Generates placeholder demo assets (no binary files in git, no image deps).
 *
 * The demo is Zellij's visual test bed, so it needs real images with real
 * dimensions to exercise next/image, aspect ratios, and layout — but the actual
 * pixels are irrelevant. These are simple gradients with a header band.
 */
import { deflateSync } from 'node:zlib';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buf) {
  let c = -1;
  for (const byte of buf) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const typeAndData = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData));
  return Buffer.concat([length, typeAndData, crc]);
}

function png(width, height, paint) {
  const raw = Buffer.alloc(height * (width * 3 + 1));
  let offset = 0;
  for (let y = 0; y < height; y += 1) {
    raw[offset] = 0; // filter: none
    offset += 1;
    for (let x = 0; x < width; x += 1) {
      const [r, g, b] = paint(x / width, y / height);
      raw[offset] = r;
      raw[offset + 1] = g;
      raw[offset + 2] = b;
      offset += 3;
    }
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // truecolor
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

const mix = (a, b, t) => Math.round(a + (b - a) * t);

/** A UI-ish placeholder: title band over a soft diagonal gradient. */
function surface(from, to) {
  return (u, v) => {
    if (v < 0.08) return [24, 27, 33];
    const t = (u + v) / 2;
    return [mix(from[0], to[0], t), mix(from[1], to[1], t), mix(from[2], to[2], t)];
  };
}

const WARM = surface([252, 240, 232], [232, 106, 51]);
const COOL = surface([238, 242, 248], [90, 122, 168]);
const DEEP = surface([32, 36, 46], [90, 70, 120]);

const IMAGES = [
  ['hero-dashboard.png', 1600, 900, WARM],
  ['shot-workflows.png', 1200, 800, COOL],
  ['shot-approvals.png', 1200, 800, WARM],
  ['shot-reporting.png', 1200, 800, COOL],
  ['shot-export.png', 1200, 800, WARM],
  ['showcase-platform.png', 1600, 900, DEEP],
  ['showcase-security.png', 1600, 900, DEEP],
  ['video-poster.png', 1280, 720, DEEP],
  ['og.png', 1200, 630, WARM],
  ['avatars/dana.png', 160, 160, WARM],
  ['avatars/samir.png', 160, 160, COOL],
];

const LOGO_COLORS = {
  northwind: '#5A7AA8',
  globex: '#E86A33',
  initech: '#3F8F6F',
  umbrella: '#8A5A9E',
};

function wordmarkSvg(name, color) {
  const label = name.charAt(0).toUpperCase() + name.slice(1);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 48" width="200" height="48" role="img" aria-label="${label}">
  <rect x="2" y="12" width="24" height="24" rx="6" fill="${color}"/>
  <text x="36" y="31" font-family="system-ui, sans-serif" font-size="19" font-weight="700" fill="currentColor">${label}</text>
</svg>
`;
}

const root = path.resolve(import.meta.dirname, '..', 'content', 'assets');

await mkdir(path.join(root, 'logos'), { recursive: true });
await mkdir(path.join(root, 'avatars'), { recursive: true });

for (const [name, width, height, paint] of IMAGES) {
  await writeFile(path.join(root, name), png(width, height, paint));
}

for (const [name, color] of Object.entries(LOGO_COLORS)) {
  await writeFile(path.join(root, 'logos', `${name}.svg`), wordmarkSvg(name, color));
}

await writeFile(path.join(root, 'logo.svg'), wordmarkSvg('acme', '#E86A33'));

console.log(`Generated ${IMAGES.length} images and ${Object.keys(LOGO_COLORS).length + 1} logos.`);
