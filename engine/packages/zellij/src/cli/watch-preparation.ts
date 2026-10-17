import { lstat, readdir, realpath, stat } from 'node:fs/promises';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { resolveContentLocation } from '../../deployment-config.cjs';
import type { ZellijRuntimeOptions } from '../config/runtime';
import { syncAssets } from './sync-assets';
import { writeSearchIndex } from './write-search-index';

export async function prepareContent(
  options: ZellijRuntimeOptions,
  changes = { search: true, assets: true },
): Promise<void> {
  // Parse the complete site before replacing assets. A partially saved YAML or
  // MDX file should leave the last usable prepared output in place.
  if (changes.search) await writeSearchIndex(options);
  if (changes.assets) await syncAssets(options);
}

interface Snapshot { search: string; assets: string }

/** Polling also works across Docker bind mounts where native events get lost. */
async function snapshot(options: ZellijRuntimeOptions): Promise<Snapshot> {
  const location = resolveContentLocation(options);
  const physical = async (file: string) => realpath(file).catch((error: NodeJS.ErrnoException) => {
    if (error.code !== 'ENOENT') throw error;
    return file;
  });
  const [appRoot, contentDir] = await Promise.all([physical(location.appRoot), physical(location.contentDir)]);
  const { configFile } = location;
  const generated = path.join(appRoot, 'public', '_zellij');
  const contentRecords: unknown[] = [contentDir];
  const assetRecords: unknown[] = [contentDir];
  const assetRoot = path.join(contentDir, 'assets');
  const visit = async (file: string, root = false): Promise<void> => {
    if (file === generated) return;
    const records = file === assetRoot || file.startsWith(`${assetRoot}${path.sep}`) ? assetRecords : contentRecords;
    let info;
    try { info = await (root ? stat(file, { bigint: true }) : lstat(file, { bigint: true })); }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      records.push([file, 'missing']);
      return;
    }
    if (info.isDirectory()) {
      // Directory mtimes include ignored generated files. Only record their
      // children so an app-root content layout cannot watch its own output.
      records.push([file, 'directory']);
      for (const name of (await readdir(file)).sort()) {
        if (['node_modules', '.next', '.git'].includes(name)) continue;
        await visit(path.join(file, name));
      }
    } else {
      records.push([file, String(info.ino), String(info.size), String(info.mtimeNs), String(info.ctimeNs)]);
    }
  };
  await visit(configFile);
  await visit(contentDir, true);
  return { search: JSON.stringify(contentRecords), assets: JSON.stringify(assetRecords) };
}

export interface PreparationWatch {
  /** Resolves when stopped, after any preparation already in progress finishes. */
  done: Promise<void>;
  close: () => Promise<void>;
}

export async function watchPreparation(
  options: ZellijRuntimeOptions = {},
  settings: { intervalMs?: number; signal?: AbortSignal; onError?: (error: unknown) => void } = {},
): Promise<PreparationWatch> {
  const controller = new AbortController();
  const stop = () => controller.abort();
  settings.signal?.addEventListener('abort', stop, { once: true });
  if (settings.signal?.aborted) stop();
  let applied: Snapshot;
  try {
    applied = await snapshot(options);
    await prepareContent(options);
  } catch (error) {
    settings.signal?.removeEventListener('abort', stop);
    throw error;
  }
  console.log('[zellij] Watching content for asset and search updates. Reload the browser after saving.');
  const done = (async () => {
    let pending: string | undefined;
    let attempted = JSON.stringify(applied);
    let lastError = '';
    while (!controller.signal.aborted) {
      try {
        await delay(settings.intervalMs ?? 500, undefined, { signal: controller.signal });
        const current = await snapshot(options);
        const key = JSON.stringify(current);
        if (key === attempted) { pending = undefined; continue; }
        // Two equal scans debounce editor rename/write sequences. All work is
        // serial; edits during preparation are picked up on the next scan.
        if (key !== pending) { pending = key; continue; }
        attempted = key;
        pending = undefined;
        await prepareContent(options, { search: current.search !== applied.search, assets: current.assets !== applied.assets });
        // Failed preparations must not acknowledge pending asset edits: fixing
        // the prose later still needs to publish both kinds of changes.
        applied = current;
        lastError = '';
      } catch (error) {
        if (controller.signal.aborted) break;
        const message = error instanceof Error ? error.message : String(error);
        if (message !== lastError) {
          lastError = message;
          if (settings.onError) settings.onError(error);
          else console.error(`[zellij] Preparation failed; watching for a corrected edit.\n${message}`);
        }
      }
    }
  })().finally(() => settings.signal?.removeEventListener('abort', stop));
  return { done, close: async () => { stop(); await done; } };
}
