export interface FileLocation { route: string; basePath: string; routes: string[]; target?: 'file' | 'static' }
let current: FileLocation;
export function setFileLocation(value: FileLocation) { current = value; }
export function fileLocation() { if (!current) throw new Error('Offline page location is missing'); return current; }

/** Only local absolute URLs change. External links and fragment links keep their meaning. */
export function fileHref(href: string, location = fileLocation()): string {
  if (location.target === 'static') return href;
  if (!href.startsWith('/') || href.startsWith('//')) return href;
  const [, pathname = '', suffix = ''] = href.match(/^([^?#]*)(.*)$/)!;
  let route = pathname;
  if (location.basePath && (route === location.basePath || route.startsWith(`${location.basePath}/`))) {
    route = route.slice(location.basePath.length) || '/';
  }
  let decoded: string;
  try { decoded = decodeURIComponent(route); } catch { return href; }
  const normalized = decoded.replace(/\/$/, '') || '/';
  // Validate before constructing a path. Paths in site.routes are already checked by the loader.
  if (decoded.split('/').some(part => part === '..' || part === '.') || decoded.includes('\\')) {
    throw new Error(`Unsafe offline URL: ${href}`);
  }
  const depth = location.route.split('/').filter(Boolean).length;
  const prefix = depth ? '../'.repeat(depth) : './';
  const file = location.routes.includes(normalized)
    ? `${normalized === '/' ? '' : normalized.slice(1) + '/'}index.html`
    : decoded.slice(1);
  return prefix + file.split('/').map(encodeURIComponent).join('/') + suffix;
}

/** next/link and router navigation apply the deployment prefix; raw asset URLs already carry it. */
export function navigationHref(href: string, location = fileLocation()): string {
  if (location.target !== 'static') return fileHref(href, location);
  const base = location.basePath;
  if (!base || !href.startsWith('/') || href.startsWith('//') ||
      href === base || href.startsWith(base + '/') || href.startsWith(base + '?') || href.startsWith(base + '#')) return href;
  return base + href;
}

export function assetProps(value: unknown): unknown {
  if (typeof value === 'string') {
    const { basePath } = fileLocation();
    if (value.startsWith(`${basePath}/_zellij/`) || value.startsWith(`${basePath}/_next/`)) return fileHref(value);
    return value.replace(/url\(\s*(['"]?)(\/[^)'"\s]+)\1\s*\)/g,
      (_all, quote: string, url: string) => `url(${quote}${fileHref(url)}${quote})`);
  }
  if (Array.isArray(value)) return value.map(assetProps);
  // React elements must not be walked as ordinary objects.
  if (value && typeof value === 'object' && !('$$typeof' in value)) {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, assetProps(item)]));
  }
  return value;
}
