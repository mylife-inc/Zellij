import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const manifest = JSON.parse(await readFile(new URL('../engine/packages/zellij/src/themes/fonts/manifest.json', import.meta.url), 'utf8'));

// Attach before navigation so even an unexpected external stylesheet request
// is caught. The fixture deliberately contains no third-party resources.
export function observeThemeFonts(page, origin, prefix) {
  const requests = [];
  page.on('request', (request) => {
    if (request.resourceType() === 'font' || /fonts\.(googleapis|gstatic)\.com/.test(request.url())) requests.push(request.url());
  });
  return async () => {
    await page.evaluate(() => document.fonts.ready);
    assert.equal(await page.locator('link[rel="preload"][as="font"]').count(), 0, 'Themes must not preload unused fonts');
    const initiallyLoaded = await page.evaluate(() => [...new Set([...document.fonts].filter((font) => font.status === 'loaded' && !font.family.includes('Fallback')).map((font) => font.family))]);
    assert(initiallyLoaded.length > 0 && initiallyLoaded.length < manifest.families.length, 'Only active theme fonts should load on navigation');
    // All families must be real usable faces, not a successful system fallback.
    // Explicit non-Latin samples exercise subsets retained from the old loader.
    for (const family of manifest.families) {
      const faces = await page.evaluate(async ({ name, variable }) => {
        const value = getComputedStyle(document.documentElement).getPropertyValue(variable);
        const loaded = await document.fonts.load(`400 20px "Zellij ${name}"`, 'Zellij Καλημέρα Привет tiếng Việt नमस्ते');
        return { value, faces: loaded.map((font) => ({ family: font.family, status: font.status })) };
      }, family);
      assert(faces.value.includes(`Zellij ${family.name}`), `Missing CSS variable: ${family.variable}`);
      assert(faces.faces.length > 0 && faces.faces.every((face) => face.status === 'loaded' && face.family.replaceAll('"', '').replaceAll("'", '') === `Zellij ${family.name}`), `Failed to load ${family.name}: ${JSON.stringify(faces)}`);
    }
    // Decode every face, including weights/subsets absent from the fixture's
    // text. This also catches missing files in a packaged engine or export.
    const allFaces = await page.evaluate(async () => {
      const faces = [...document.fonts].filter((face) => face.family.includes('Zellij ') && !face.family.includes('Fallback'));
      await Promise.all(faces.map((face) => face.load()));
      return faces.map((face) => face.status);
    });
    assert(allFaces.length >= manifest.families.reduce((count, family) => count + family.files.length, 0) && allFaces.every((status) => status === 'loaded'), 'All font weights and Unicode subsets must decode');
    assert(requests.length > 0 && requests.every((url) => url.startsWith(`${origin}${prefix}/_next/static/media/`)), `Fonts must be local and prefixed: ${JSON.stringify(requests)}`);
    const css = await page.locator('link[rel="stylesheet"]').evaluateAll(async (links) => (await Promise.all(links.map((link) => fetch(link.href).then((response) => response.text())))).join('\n'));
    assert(css.includes('SIL OPEN FONT LICENSE'), 'Deployed CSS must retain font license notices');
    for (const family of manifest.families) assert(css.includes(`/*! ${family.name}`), `Missing deployed notice for ${family.name}`);
    console.log(`Verified all ${manifest.families.length} local theme fonts, lazy loading, Unicode subsets and deployed licenses.`);
  };
}
