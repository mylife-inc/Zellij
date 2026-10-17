import { cp, mkdir, readdir, realpath, rm, stat, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { readFile } from 'node:fs/promises';

import { createContentSource, resolveContentPaths, type ZellijRuntimeOptions } from '../config/runtime';
import { ZellijContentError } from '../errors';
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
export async function syncAssets(input: string | ZellijRuntimeOptions = {}): Promise<void> {
  const options = typeof input === 'string' ? { appRoot: input } : input;
  const { appRoot } = resolveContentPaths(options);
  const { contentDir } = await createContentSource(options);
  const source = path.join(contentDir, 'assets');
  const destination = path.join(appRoot, 'public', '_zellij', 'assets');

  const [sourcePath, destinationPath] = await Promise.all([physicalPath(source), physicalPath(destination)]);
  if (contains(sourcePath, destinationPath) || contains(destinationPath, sourcePath)) {
    throw new ZellijContentError('Content assets and public/_zellij/assets must not overlap.', source);
  }
  const assets = await stat(source).catch((error: NodeJS.ErrnoException) => {
    if (error.code !== 'ENOENT') throw error;
    return undefined;
  });
  if (assets && !assets.isDirectory()) {
    throw new ZellijContentError('Content assets must be a directory.', source);
  }
  const hasAssets = assets !== undefined;

  // Replace rather than merge, including when the entire assets folder was
  // removed. Re-running preparation must not retain stale assets or hashes.
  await rm(destination, { recursive: true, force: true });
  await mkdir(path.dirname(destination), { recursive: true });
  if (hasAssets) await cp(source, destination, { recursive: true });
  const fingerprints = hasAssets ? await fingerprint(destination) : {};
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

function contains(parent: string, child: string): boolean {
  const relative = path.relative(parent, child);
  return relative === '' || (!path.isAbsolute(relative) && relative !== '..' && !relative.startsWith(`..${path.sep}`));
}

/** Resolve existing ancestors too, so a public-directory symlink cannot hide overlap. */
async function physicalPath(target: string): Promise<string> {
  try {
    return await realpath(target);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
    return path.join(await physicalPath(path.dirname(target)), path.basename(target));
  }
}
