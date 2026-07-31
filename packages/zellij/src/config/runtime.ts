import path from 'node:path';
import { FileSystemContentSource } from '../content/filesystem-source';
import { loadSite, type Site } from '../content/site';

export interface ZellijRuntimeOptions {
  /**
   * Wrapper app root — the directory holding zellij.config.yaml.
   * Defaults to process.cwd(), which is the app root during `next build`/`next dev`.
   */
  appRoot?: string;
  /** Override the config filename. */
  configFile?: string;
  onWarning?: (message: string) => void;
}

export interface ZellijRuntime {
  site: Site;
  contentDir: string;
  configFile: string;
}

export function resolveContentPaths(options: ZellijRuntimeOptions = {}): {
  appRoot: string;
  configFile: string;
} {
  const appRoot = options.appRoot ?? process.cwd();
  const configFile = path.resolve(appRoot, options.configFile ?? 'zellij.config.yaml');
  return { appRoot, configFile };
}

/**
 * Loads config + all content once per process. Next dev re-evaluates modules on
 * change, so a module-level cache is safe and keeps SSG builds fast.
 */
let cached: Promise<ZellijRuntime> | undefined;

export function loadZellijRuntime(options: ZellijRuntimeOptions = {}): Promise<ZellijRuntime> {
  if (cached) return cached;

  cached = (async () => {
    const { appRoot, configFile } = resolveContentPaths(options);
    const warn = options.onWarning ?? ((m: string) => console.warn(`[zellij] ${m}`));

    // Read config first so we know where the content directory lives.
    const bootstrap = new FileSystemContentSource({
      contentDir: appRoot,
      configFile,
      onWarning: warn,
    });
    const config = await bootstrap.loadConfig();
    const contentDir = path.resolve(appRoot, config.content);

    const source = new FileSystemContentSource({ contentDir, configFile, onWarning: warn });
    const site = await loadSite(source);

    for (const warning of site.warnings) warn(warning);

    return { site, contentDir, configFile };
  })();

  return cached;
}

/** Test/dev hook — drops the memoized runtime. */
export function resetZellijRuntime(): void {
  cached = undefined;
}
