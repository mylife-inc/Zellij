import { readFileSync } from 'node:fs';
import path from 'node:path';

import { ASSET_MANIFEST_FILE, assetKey, assetUrl, isExternal, withBasePath } from './paths';

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

/*
 * Read once in production, and re-read while writing.
 *
 * The manifest is rewritten by preparation and the development watcher, so a dev
 * server that memoised it for its lifetime would serve fingerprints for assets
 * that had since been replaced — the precise failure this file exists to
 * prevent, reintroduced one layer up. The TTL matches the content runtime's.
 */
const DEV_TTL_MS = 250;

let cached: Record<string, string> | undefined;
let cachedAt = 0;

function manifest(): Record<string, string> {
  if (cached && (process.env['NODE_ENV'] === 'production' || Date.now() - cachedAt < DEV_TTL_MS)) {
    return cached;
  }

  try {
    const file = path.join(process.cwd(), 'public', '_zellij', ASSET_MANIFEST_FILE);
    cached = JSON.parse(readFileSync(file, 'utf8')) as Record<string, string>;
  } catch {
    /*
     * No manifest is a normal state, not an error: a site that has never run
     * `zellij prepare`, or one whose assets folder is empty. Unfingerprinted
     * URLs are what the engine emitted until now and they work; the only thing
     * lost is the cache busting.
     */
    cached = {};
  }

  cachedAt = Date.now();
  return cached;
}
