import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { createContentSource, resolveContentPaths, type ZellijRuntimeOptions } from '../config/runtime';
import { loadSite } from '../content/site';
import { loadMaths } from '../mdx/maths';
import { buildSearchIndex } from '../search/build-index';

/** Build search from the same resolved routes and guide bodies as rendering. */
export async function writeSearchIndex(input: string | ZellijRuntimeOptions = {}): Promise<void> {
  const options = typeof input === 'string' ? { appRoot: input } : input;
  const { appRoot } = resolveContentPaths(options);
  // Preparation must read fresh content even when several sites or repeated
  // preparations share a process. Runtime rendering has a separate memo.
  const { source } = await createContentSource(options);
  const site = await loadSite(source);
  const warn = options.onWarning ?? ((message: string) => console.warn(`[zellij] ${message}`));
  for (const warning of site.warnings) warn(warning);
  const [remarkPlugins] = await loadMaths();
  const entries = buildSearchIndex(site.guidePages, { remarkPlugins });

  const destination = path.join(appRoot, 'public', '_zellij', 'search-index.json');
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, JSON.stringify(entries), 'utf8');
  console.log(
    `[zellij] Search index → public/_zellij/search-index.json ` +
    `(${entries.length} entries from ${site.guidePages.length} guide pages)`,
  );
}
