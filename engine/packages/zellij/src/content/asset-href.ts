import { readFileSync, statSync } from 'node:fs';
import path from 'node:path';

import { ASSET_MANIFEST_FILE, ASSET_PUBLIC_PREFIX, assetKey, assetUrl, isExternal, withBasePath } from './paths';

/**
 * The URL to use for a content asset: public path, base path, and the content
 * fingerprint that lets it be cached forever and still change.
 *
 * Server-only. It reads the manifest from disk, so it must never be imported
 * from a client component — which is why it does not live in `paths.ts`, whose
 * other exports the search modal needs in the browser.
 *
 * Every image on a Zellij site goes through here. `assetUrl` on its own is
 * still correct, just uncacheable in the way that matters: a stable filename
 * means a changed image keeps its old URL, and a CDN or a phone will go on
 * serving what it already has. The query string moves with the bytes, so the
 * new image is a new URL and the old one is nobody's to keep.
 */
export function assetHref(relPath: string, basePath = ''): string {
  if (isExternal(relPath)) return relPath;

  const url = withBasePath(assetUrl(relPath), basePath);
  const hash = manifest()[assetKey(relPath)];

  return hash ? `${url}?v=${hash}` : url;
}

/** Markdown imports already carry public URLs, including encoded filenames. */
export function publicAssetHref(src: string, basePath = ''): string {
  if (!src.startsWith(`${ASSET_PUBLIC_PREFIX}/`)) return src;
  try {
    const url = new URL(src, 'http://zellij.invalid');
    const key = decodeURIComponent(url.pathname.slice(ASSET_PUBLIC_PREFIX.length + 1));
    const hash = manifest()[key];
    if (hash) {
      url.searchParams.set('v', hash);
      return withBasePath(`${url.pathname}${url.search}${url.hash}`, basePath);
    }
  } catch { /* An invalid escape must not crash an otherwise renderable image. */ }
  return withBasePath(src, basePath);
}

/*
 * Read once in production, and re-read while writing.
 *
 * The manifest is rewritten by preparation and the development watcher, so a dev
 * server that memoised it for its lifetime would serve fingerprints for assets
 * that had since been replaced — the precise failure this file exists to
 * prevent, reintroduced one layer up. Preparation atomically replaces the
 * manifest, so its identity/mtime invalidates the cache even if an automatic
 * reload arrives within the old 250 ms memo window.
 */
let cached: Record<string, string> | undefined;
let cachedSignature: string | undefined;

function manifest(): Record<string, string> {
  if (cached && process.env['NODE_ENV'] === 'production') return cached;

  try {
    const file = path.join(process.cwd(), 'public', '_zellij', ASSET_MANIFEST_FILE);
    const metadata = statSync(file, { bigint: true });
    const signature = `${file}:${metadata.ino}:${metadata.mtimeNs}:${metadata.ctimeNs}:${metadata.size}`;
    if (cached && signature === cachedSignature) return cached;
    cached = JSON.parse(readFileSync(file, 'utf8')) as Record<string, string>;
    cachedSignature = signature;
  } catch {
    /*
     * No manifest is a normal state, not an error: a site that has never run
     * `zellij prepare`, or one whose assets folder is empty. Unfingerprinted
     * URLs are what the engine emitted until now and they work; the only thing
     * lost is the cache busting.
     */
    cached = {};
    cachedSignature = undefined;
  }

  return cached;
}
