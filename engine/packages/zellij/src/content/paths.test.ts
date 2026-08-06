import { describe, expect, it } from 'vitest';
import {
  assetKey,
  assetUrl,
  guideRouteFromFile,
  isExternal,
  marketingRouteFromFile,
  normalizeBasePath,
  normalizeRoute,
  routeToSegments,
  segmentsToRoute,
  splitAnchor,
  withBasePath,
} from './paths';

describe('marketingRouteFromFile', () => {
  it('maps landing.yaml to the site root', () => {
    expect(marketingRouteFromFile('landing.yaml')).toBe('/');
  });

  it('maps a top-level page to its slug', () => {
    expect(marketingRouteFromFile('features.yaml')).toBe('/features');
  });

  it('preserves nested folders', () => {
    expect(marketingRouteFromFile('solutions/startups.yaml')).toBe('/solutions/startups');
  });
});

describe('guideRouteFromFile', () => {
  it('maps index.mdx to /guide', () => {
    expect(guideRouteFromFile('index.mdx')).toBe('/guide');
  });

  it('maps a page to /guide/<slug>', () => {
    expect(guideRouteFromFile('quickstart.mdx')).toBe('/guide/quickstart');
  });

  it('preserves nested folders', () => {
    expect(guideRouteFromFile('config/database.mdx')).toBe('/guide/config/database');
  });
});

describe('basePath handling', () => {
  it('normalizes a missing or bare basePath to empty', () => {
    expect(normalizeBasePath(undefined)).toBe('');
    expect(normalizeBasePath('')).toBe('');
    expect(normalizeBasePath('/')).toBe('');
  });

  it('adds a leading slash and strips a trailing one', () => {
    expect(normalizeBasePath('products/acme/docs/')).toBe('/products/acme/docs');
  });

  it('prefixes internal URLs', () => {
    expect(withBasePath('/features', '/products/acme/docs')).toBe(
      '/products/acme/docs/features',
    );
  });

  it('leaves external URLs and bare anchors alone', () => {
    expect(withBasePath('https://example.com', '/base')).toBe('https://example.com');
    expect(withBasePath('#pricing', '/base')).toBe('#pricing');
  });

  it('is a no-op when no basePath is configured', () => {
    expect(withBasePath('/features', '')).toBe('/features');
  });
});

describe('assetUrl', () => {
  it('maps a content-relative asset to the public prefix', () => {
    expect(assetUrl('assets/hero.png')).toBe('/_zellij/assets/hero.png');
  });

  it('tolerates a leading ./', () => {
    expect(assetUrl('./assets/logos/a.svg')).toBe('/_zellij/assets/logos/a.svg');
  });

  it('passes external URLs through', () => {
    expect(assetUrl('https://cdn.example.com/a.png')).toBe('https://cdn.example.com/a.png');
  });
});

describe('assetKey', () => {
  /*
   * The key a fingerprint is stored under. It has to agree with the path
   * `zellij prepare` walked to in `public/_zellij/assets`, or every lookup
   * misses and the fingerprinting silently does nothing.
   */
  it('strips the assets prefix and any leading ./', () => {
    expect(assetKey('assets/hero.png')).toBe('hero.png');
    expect(assetKey('./assets/hero.png')).toBe('hero.png');
    expect(assetKey('hero.png')).toBe('hero.png');
  });

  it('keeps nesting, which is how the manifest is keyed', () => {
    expect(assetKey('assets/brand/logo.png')).toBe('brand/logo.png');
  });

  it('agrees with assetUrl about what the path is', () => {
    for (const input of ['assets/hero.png', './assets/brand/logo.png', 'a/b/c.svg']) {
      expect(assetUrl(input)).toBe(`/_zellij/assets/${assetKey(input)}`);
    }
  });
});

describe('route helpers', () => {
  it('round-trips routes through segments', () => {
    expect(routeToSegments('/')).toEqual([]);
    expect(routeToSegments('/solutions/startups')).toEqual(['solutions', 'startups']);
    expect(segmentsToRoute(undefined)).toBe('/');
    expect(segmentsToRoute(['solutions', 'startups'])).toBe('/solutions/startups');
  });

  it('normalizes routes', () => {
    expect(normalizeRoute('features')).toBe('/features');
    expect(normalizeRoute('/features/')).toBe('/features');
    expect(normalizeRoute('/')).toBe('/');
  });

  it('splits anchors from routes', () => {
    expect(splitAnchor('/features#security')).toEqual(['/features', 'security']);
    expect(splitAnchor('/features')).toEqual(['/features', undefined]);
    expect(splitAnchor('#top')).toEqual(['', 'top']);
  });

  it('detects external links', () => {
    expect(isExternal('https://example.com')).toBe(true);
    expect(isExternal('mailto:a@b.co')).toBe(true);
    expect(isExternal('/features')).toBe(false);
  });
});
