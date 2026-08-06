import { cp, mkdir, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import { readFile } from 'node:fs/promises';

import { ASSET_MANIFEST_FILE } from '../content/paths';

/**
 * Copies the content directory's assets into the app's `public/_zellij/assets`
 * so `next/image` can optimize them and every URL picks up `basePath`
 * automatically (spec §13.4).
 *
 * Runs as a prebuild/predev step. Content lives outside the Next app, so this
 * is the bridge between the authored path (`assets/hero.png`) and the served
 * path (`/_zellij/assets/hero.png`).
 */
export async function syncAssets(appRoot: string = process.cwd()): Promise<void> {
  const contentDir = await resolveContentDir(appRoot);
  const source = path.join(contentDir, 'assets');
  const destination = path.join(appRoot, 'public', '_zellij', 'assets');

  if (!(await exists(source))) {
    console.log(`[zellij] No assets directory at ${source} — nothing to sync.`);
    return;
  }

  // Replace rather than merge, so deleted assets do not linger in public/.
  await rm(destination, { recursive: true, force: true });
  await mkdir(path.dirname(destination), { recursive: true });
  await cp(source, destination, { recursive: true });

  const fingerprints = await fingerprint(destination);
  await writeFile(
    path.join(appRoot, 'public', '_zellij', ASSET_MANIFEST_FILE),
    `${JSON.stringify(fingerprints, null, 2)}\n`,
  );

  /*
   * Drop Next's optimized-image cache.
   *
   * The cache is keyed by request URL, and a replaced asset keeps the same one,
   * so `next build` alone will happily serve the previous version of an image
   * that changed on disk — indefinitely, and with no warning. Editing an image
   * and seeing the old one is a genuinely disorienting way to lose an hour.
   *
   * Both locations, because Turbopack moved it. Under Next 16 the dev server
   * caches in `.next/dev/cache/images` and clearing only the old path silently
   * does nothing — which is the hour described above, spent confirming over and
   * over that the bytes on disk and the bytes on the wire were the new ones
   * while the browser kept painting the old.
   */
  for (const cache of [
    path.join(appRoot, '.next', 'cache', 'images'),
    path.join(appRoot, '.next', 'dev', 'cache', 'images'),
  ]) {
    await rm(cache, { recursive: true, force: true });
  }

  const count = Object.keys(fingerprints).length;
  console.log(
    `[zellij] Synced assets → public/_zellij/assets (${count} fingerprinted, image cache cleared)`,
  );
}

/**
 * A short content hash for every synced asset, keyed by its path inside the
 * assets folder.
 *
 * Asset filenames are the author's, and they are stable: `logo.png` stays
 * `logo.png` across a rebrand. Next fingerprints its own bundles, so a changed
 * stylesheet arrives at a new URL and no cache can hold the old one — content
 * assets had no such thing. Behind a CDN that means replacing an image leaves
 * every returning visitor with the previous one until their cache expires, with
 * no way to force the issue short of renaming the file in the content folder.
 *
 * Eight hex characters of SHA-256. Enough that two different images colliding
 * is not a thing that will happen, short enough to read in a URL.
 */
async function fingerprint(root: string): Promise<Record<string, string>> {
  const manifest: Record<string, string> = {};

  const walk = async (dir: string): Promise<void> => {
    for (const entry of await readdir(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        await walk(full);
        continue;
      }
      if (!entry.isFile()) continue;

      const hash = createHash('sha256').update(await readFile(full)).digest('hex').slice(0, 8);
      // Posix separators: this is a URL key, not a filesystem path.
      manifest[path.relative(root, full).split(path.sep).join('/')] = hash;
    }
  };

  await walk(root);
  return manifest;
}

/**
 * `./content` unless the legacy config names somewhere else. The content root
 * carries its own identity now, so no config file is required to find it.
 */
async function resolveContentDir(appRoot: string): Promise<string> {
  const configFile = path.join(appRoot, 'zellij.config.yaml');
  if (await exists(configFile)) {
    try {
      const config = parseYaml(await readFile(configFile, 'utf8')) as { content?: string } | null;
      return path.resolve(appRoot, config?.content ?? './content');
    } catch {
      // Unparseable config is the build's problem to report, not this step's.
    }
  }
  return path.resolve(appRoot, 'content');
}

async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}
