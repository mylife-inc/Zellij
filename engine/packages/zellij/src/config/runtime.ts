import path from 'node:path';
import { stat } from 'node:fs/promises';
import { ZellijContentError } from '../errors';
import { FolderContentSource } from '../content/folder-source';
import { LegacyContentSource } from '../content/legacy-source';
import { exists } from '../content/fs-utils';
import { loadSite, type Site } from '../content/site';
import type { ContentSource } from '../content/types';

export interface ZellijRuntimeOptions {
  /**
   * Wrapper app root. Defaults to process.cwd(), which is the app root during
   * `next build` and `next dev`.
   */
  appRoot?: string;
  /**
   * The content root, absolute or relative to `appRoot`. Defaults to
   * `./content`. The wrapper owns this because it owns where the site is
   * checked out; everything about the site itself is inside the folder.
   */
  contentDir?: string;
  /** Mirrors next.config basePath. Overrides `basePath` in site.yaml. */
  basePath?: string;
  /** Legacy layout only: path to zellij.config.yaml. */
  configFile?: string;
  onWarning?: (message: string) => void;
}

export interface ZellijRuntime {
  site: Site;
  contentDir: string;
  /** Which layout was detected — the legacy path is removed after one release. */
  layout: 'folder' | 'legacy';
}

export function resolveContentPaths(options: ZellijRuntimeOptions = {}): {
  appRoot: string;
  configFile: string;
} {
  const appRoot = path.resolve(options.appRoot ?? process.cwd());
  const configFile = path.resolve(appRoot, options.configFile ?? 'zellij.config.yaml');
  return { appRoot, configFile };
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
): Promise<{ source: ContentSource; contentDir: string; layout: 'folder' | 'legacy' }> {
  const { appRoot, configFile } = resolveContentPaths(options);

  let contentDir = options.contentDir
    ? path.resolve(appRoot, options.contentDir)
    : path.resolve(appRoot, 'content');

  // The old config named its own content directory, so it has to be read before
  // we know where to look.
  const hasLegacyConfig = await exists(configFile);
  if (!options.contentDir && hasLegacyConfig) {
    const bootstrap = new LegacyContentSource({ contentDir: appRoot, configFile, onWarning: warn });
    const legacy = await bootstrap.loadLegacyConfig();
    contentDir = path.resolve(appRoot, legacy.content);
  }

  const root = await stat(contentDir).catch((error: NodeJS.ErrnoException) => {
    if (error.code !== 'ENOENT') throw error;
    return undefined;
  });
  if (!root?.isDirectory()) throw new ZellijContentError(`No content directory at ${contentDir}`, contentDir);

  if (await exists(path.join(contentDir, 'marketing'))) {
    warn(
      `${contentDir} uses the legacy marketing/ layout. It still builds, but the layout is ` +
        `deprecated — see specs/zellij-content-model-design.md §12 for the migration.`,
    );
    return {
      source: new LegacyContentSource({ contentDir, configFile, onWarning: warn }),
      contentDir,
      layout: 'legacy',
    };
  }

  return {
    source: new FolderContentSource({
      contentDir,
      ...(options.basePath !== undefined ? { basePath: options.basePath } : {}),
      onWarning: warn,
    }),
    contentDir,
    layout: 'folder',
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

let cached: Promise<ZellijRuntime> | undefined;
let cachedAt = 0;

function stillFresh(): boolean {
  if (!cached) return false;
  if (process.env.NODE_ENV === 'production') return true;
  return Date.now() - cachedAt < DEV_TTL_MS;
}

export function loadZellijRuntime(options: ZellijRuntimeOptions = {}): Promise<ZellijRuntime> {
  if (stillFresh()) return cached!;

  cachedAt = Date.now();

  cached = (async () => {
    const warn = options.onWarning ?? ((message: string) => console.warn(`[zellij] ${message}`));
    const { source, contentDir, layout } = await createContentSource(options, warn);

    const site = await loadSite(source);
    if (options.basePath !== undefined) site.config.basePath = options.basePath;

    for (const warning of site.warnings) warn(warning);

    return { site, contentDir, layout };
  })();

  return cached;
}

/** Test/dev hook — drops the memoized runtime. */
export function resetZellijRuntime(): void {
  cached = undefined;
}
