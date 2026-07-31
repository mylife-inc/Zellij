import { cp, mkdir, rm, stat } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import { readFile } from 'node:fs/promises';

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
  const configFile = path.join(appRoot, 'zellij.config.yaml');

  let contentDir: string;
  try {
    const raw = await readFile(configFile, 'utf8');
    const config = parseYaml(raw) as { content?: string } | null;
    contentDir = path.resolve(appRoot, config?.content ?? './content');
  } catch (cause) {
    throw new Error(
      `[zellij] Could not read ${configFile} to locate the content directory.\n` +
        `  ${(cause as Error).message}`,
    );
  }

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

  console.log(`[zellij] Synced assets → public/_zellij/assets`);
}

async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}
