import path from 'node:path';
import { stat } from 'node:fs/promises';
import { ZellijContentError } from '../errors';
import { FolderContentSource } from '../content/folder-source';
import { LegacyContentSource } from '../content/legacy-source';
import { loadSite, type Site } from '../content/site';
import type { ContentSource } from '../content/types';
import {
  resolveContentPaths, resolveContentLocation, normalizeDeploymentBasePath, resolveDeploymentBasePath,
} from '../../deployment-config.cjs';
export { resolveContentPaths } from '../../deployment-config.cjs';

export interface ZellijRuntimeOptions {
  /**
   * Wrapper app root. Defaults to process.cwd(), which is the app root during
   * `next build` and `next dev`.
   */
  appRoot?: string;
  /**
   * The content root, absolute or relative to `appRoot`. Defaults to
   * the wrapper's zellij.config.yaml content selector, then `./content`.
   * The wrapper owns this because it owns where the site is
   * checked out; everything about the site itself is inside the folder.
   */
  contentDir?: string;
  /** Must agree with the prefix compiled by withZellij during a Next build. */
  basePath?: string;
  /** Path to the wrapper selector or legacy zellij.config.yaml. */
  configFile?: string;
  onWarning?: (message: string) => void;
}

export interface ZellijRuntime {
  site: Site;
  contentDir: string;
  /** Which layout was detected — the legacy path is removed after one release. */
  layout: 'folder' | 'legacy';
}

/**
 * Picks a content source by what the folder contains — design §12.
 *
 * A content root holding `marketing/` is the pre-content-model layout and is
 * read by the legacy source, which produces the same internal model. Anything
 * else is read as a content root proper.
 */
export async function createContentSource(
  options: ZellijRuntimeOptions = {},
  warn: (message: string) => void = options.onWarning ?? ((message) => console.warn(`[zellij] ${message}`)),
): Promise<{ source: ContentSource; contentDir: string; layout: 'folder' | 'legacy'; basePath: string }> {
  const {
    appRoot, configFile, contentDir, layout, basePath: authoredBasePath, legacyConfigUsed,
  } = resolveContentLocation(options);
  // Preserve full legacy validation even when its content path selects a new
  // folder-layout site. The shared bootstrap only reads deployment fields.
  if (legacyConfigUsed) {
    await new LegacyContentSource({ contentDir: appRoot, configFile, onWarning: warn }).loadLegacyConfig();
  }
  const builtBasePath = process.env.ZELLIJ_BASE_PATH;
  const basePath = builtBasePath === undefined
    ? normalizeDeploymentBasePath(options.basePath ?? authoredBasePath)
    : resolveDeploymentBasePath(authoredBasePath, options.basePath, builtBasePath);

  const root = await stat(contentDir).catch((error: NodeJS.ErrnoException) => {
    if (error.code !== 'ENOENT') throw error;
    return undefined;
  });
  if (!root?.isDirectory()) throw new ZellijContentError(`No content directory at ${contentDir}`, contentDir);

  if (layout === 'legacy') {
    warn(
      `${contentDir} uses the legacy marketing/ layout. It still builds, but the layout is ` +
        `deprecated — see specs/zellij-content-model-design.md §12 for the migration.`,
    );
    return {
      source: new LegacyContentSource({ contentDir, configFile, onWarning: warn }),
      contentDir,
      layout: 'legacy',
      basePath,
    };
  }

  return {
    source: new FolderContentSource({
      contentDir,
      basePath,
      onWarning: warn,
    }),
    contentDir,
    layout: 'folder',
    basePath,
  };
}

/**
 * Loads config and all content, memoized.
 *
 * One page render calls this many times — every section, every link check —
 * so without a cache a build re-reads the whole folder per component. That is
 * what the cache is for.
 *
 * It used to be permanent, on the reasoning that Next dev re-evaluates modules
 * when they change. It does — for modules *inside the Next project*. Content
 * lives outside it, often on the other side of a container mount, so Next
 * never learns a YAML file was edited, never re-evaluates this module, and the
 * first load of the process is served until it is restarted. Editing a
 * headline and reloading showed the old one for as long as the server ran.
 *
 * So in development the memo expires. The window is long enough to still
 * collapse the many calls within a single render, and short enough that saving
 * a file and switching to the browser shows the new text. A build sets
 * NODE_ENV=production and keeps the permanent cache, because content cannot
 * change underneath a build.
 */
const DEV_TTL_MS = 250;

const cache = new Map<string, { value: Promise<ZellijRuntime>; createdAt: number }>();

export function loadZellijRuntime(options: ZellijRuntimeOptions = {}): Promise<ZellijRuntime> {
  const { appRoot, configFile } = resolveContentPaths(options);
  const key = JSON.stringify([
    appRoot, configFile,
    options.contentDir === undefined ? null : path.resolve(appRoot, options.contentDir),
    options.basePath ?? null, process.env.ZELLIJ_BASE_PATH ?? null,
  ]);
  const now = Date.now();
  const cached = cache.get(key);
  if (cached && (process.env.NODE_ENV === 'production' || now - cached.createdAt < DEV_TTL_MS)) {
    return cached.value;
  }
  if (process.env.NODE_ENV !== 'production') {
    for (const [oldKey, entry] of cache) if (now - entry.createdAt >= DEV_TTL_MS) cache.delete(oldKey);
  }
  const value = (async () => {
    const warn = options.onWarning ?? ((message: string) => console.warn(`[zellij] ${message}`));
    const { source, contentDir, layout, basePath } = await createContentSource(options, warn);
    const site = await loadSite(source);
    site.config.basePath = basePath;
    for (const warning of site.warnings) warn(warning);
    return { site, contentDir, layout };
  })();
  cache.set(key, { value, createdAt: now });
  // A failed load must not poison the process after its input is repaired.
  void value.catch(() => {
    if (cache.get(key)?.value === value) cache.delete(key);
  });
  return value;
}

/** Test/dev hook — drops the memoized runtime. */
export function resetZellijRuntime(): void {
  cache.clear();
}
