/** Public URL prefix under which content assets are exposed (spec §13.4). */
export const ASSET_PUBLIC_PREFIX = '/_zellij/assets';

/** Where the build-time guide search index is written (spec §13.6). */
export const SEARCH_INDEX_PATH = '/_zellij/search-index.json';

/** Normalizes a basePath: "" or "/products/acme/docs" — never a trailing slash. */
export function normalizeBasePath(basePath: string | undefined): string {
  if (!basePath) return '';
  const trimmed = basePath.trim().replace(/\/+$/, '');
  if (trimmed === '') return '';
  return trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
}

/**
 * Prefixes an internal, root-relative URL with basePath. External URLs and
 * anchors pass through untouched. This is the only place prefixing happens for
 * non-`next/image` URLs — `next/image` and `next/link` apply basePath themselves.
 */
export function withBasePath(url: string, basePath: string): string {
  if (isExternal(url) || url.startsWith('#')) return url;
  const base = normalizeBasePath(basePath);
  if (base === '') return url;
  return url.startsWith('/') ? `${base}${url}` : `${base}/${url}`;
}

export function isExternal(url: string): boolean {
  return /^(https?:)?\/\//.test(url) || url.startsWith('mailto:') || url.startsWith('tel:');
}

/**
 * Maps a content-relative asset path ("assets/hero.png" or "./assets/hero.png")
 * to its public URL. Returns a root-relative URL; callers that don't go through
 * `next/image` must additionally apply `withBasePath`.
 */
export function assetUrl(relPath: string): string {
  if (isExternal(relPath)) return relPath;
  const clean = relPath.replace(/^\.?\//, '').replace(/^assets\//, '');
  return `${ASSET_PUBLIC_PREFIX}/${clean}`;
}

/**
 * Derives a marketing route from a content-relative filename.
 * `landing.yaml` → "/", `features.yaml` → "/features", `solutions/startups.yaml`
 * → "/solutions/startups" (spec §4).
 */
export function marketingRouteFromFile(relFile: string): string {
  const withoutExt = relFile.replace(/\.ya?ml$/i, '');
  if (withoutExt === 'landing' || withoutExt === 'index') return '/';
  return `/${withoutExt}`;
}

/**
 * Derives a guide route from a content-relative filename.
 * `index.mdx` → "/guide", `quickstart.mdx` → "/guide/quickstart",
 * `config/database.mdx` → "/guide/config/database" (spec §4).
 */
export function guideRouteFromFile(relFile: string): string {
  const withoutExt = relFile.replace(/\.mdx?$/i, '');
  if (withoutExt === 'index') return '/guide';
  const trimmed = withoutExt.replace(/\/index$/, '');
  return `/guide/${trimmed}`;
}

/** Normalizes an author-supplied route override to a canonical form. */
export function normalizeRoute(route: string): string {
  if (route === '/' || route === '') return '/';
  const withLeading = route.startsWith('/') ? route : `/${route}`;
  return withLeading.replace(/\/+$/, '') || '/';
}

/** Splits a route into Next.js catch-all segments. "/" → [], "/a/b" → ["a","b"]. */
export function routeToSegments(route: string): string[] {
  const normalized = normalizeRoute(route);
  if (normalized === '/') return [];
  return normalized.slice(1).split('/');
}

/** Inverse of routeToSegments. */
export function segmentsToRoute(segments: string[] | undefined): string {
  if (!segments || segments.length === 0) return '/';
  return `/${segments.join('/')}`;
}

/** Separates a href into its route and optional anchor: "/f#sec" → ["/f","sec"]. */
export function splitAnchor(href: string): [string, string | undefined] {
  const hashIndex = href.indexOf('#');
  if (hashIndex === -1) return [href, undefined];
  const route = href.slice(0, hashIndex);
  const anchor = href.slice(hashIndex + 1);
  return [route === '' ? '' : normalizeRoute(route), anchor || undefined];
}
