import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import matter from 'gray-matter';

// Explicit extensions: this module runs under plain Node (via the bin script),
// where ESM requires them. Both imports are leaves with no further dependencies,
// which is what keeps this graph to one hop.
import { buildSearchIndex } from '../search/build-index.ts';
import { slugifyPath, stripExtension } from '../content/naming.ts';
import type { GuidePage } from '../content/types.ts';

/**
 * Emits the guide search index as a static asset (spec §13.6).
 *
 * Runs as a prebuild step alongside asset syncing, so the index is a plain file
 * the ⌘K modal fetches on first open — nothing is indexed at request time and
 * no search service is required.
 */
export async function writeSearchIndex(appRoot: string = process.cwd()): Promise<void> {
  const contentDir = await resolveContentDir(appRoot);
  const guidePages: GuidePage[] = [];

  for (const folder of await guideFolders(contentDir)) {
    guidePages.push(...(await readGuidePages(path.join(contentDir, folder), folder)));
  }

  const entries = buildSearchIndex(guidePages);

  const destination = path.join(appRoot, 'public', '_zellij', 'search-index.json');
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, JSON.stringify(entries), 'utf8');

  console.log(
    `[zellij] Search index → public/_zellij/search-index.json ` +
      `(${entries.length} entries from ${guidePages.length} guide pages)`,
  );
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
      // Unparseable config is the build's problem to report, not the indexer's.
    }
  }
  return path.resolve(appRoot, 'content');
}

/**
 * Every top-level folder holding a `_sidebar.yaml`. The legacy layout's `guide/`
 * had no such file, so it is included on name.
 */
async function guideFolders(contentDir: string): Promise<string[]> {
  if (!(await exists(contentDir))) return [];

  const found: string[] = [];
  for (const entry of await readdir(contentDir, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name.startsWith('.')) continue;
    if (entry.name === 'sections' || entry.name === 'assets') continue;

    const isGuide =
      entry.name === 'guide' ||
      (await exists(path.join(contentDir, entry.name, '_sidebar.yaml')));
    if (isGuide) found.push(entry.name);
  }
  return found.sort();
}

/**
 * A minimal guide reader. Deliberately does not go through ContentSource: that
 * module's import graph is written for the bundler, and reproducing it under
 * Node's stricter resolver would mean extension-annotating half the package.
 * Validation still happens during the build proper — this only needs enough
 * structure to index.
 */
async function readGuidePages(guideDir: string, relDir: string): Promise<GuidePage[]> {
  if (!(await exists(guideDir))) return [];

  // The legacy `guide/` folder mounted at its own name; every other folder
  // slugifies, dropping any ordering prefix.
  const treeRoute = relDir === 'guide' ? '/guide' : `/${slugifyPath(relDir)}`;
  const files = await walk(guideDir);
  const pages: GuidePage[] = [];

  for (const relFile of files) {
    const raw = await readFile(path.join(guideDir, relFile), 'utf8');
    const { data, content } = matter(raw);
    const frontmatter = data as { title?: string; description?: string; route?: string };

    const slug = stripExtension(relFile).replace(/(^|\/)index$/, '');
    const route =
      frontmatter.route ?? (slug === '' ? treeRoute : `${treeRoute}/${slugifyPath(slug)}`);

    pages.push({
      route,
      sourceFile: `${relDir}/${relFile}`,
      title: frontmatter.title ?? firstHeading(content) ?? stripExtension(relFile),
      ...(frontmatter.description ? { description: frontmatter.description } : {}),
      body: content,
      guide: treeRoute,
    });
  }

  return pages;
}

async function walk(dir: string, prefix = ''): Promise<string[]> {
  const out: string[] = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (entry.name.startsWith('.') || entry.name.startsWith('_')) continue;
    const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) out.push(...(await walk(path.join(dir, entry.name), rel)));
    else if (/\.mdx?$/i.test(entry.name)) out.push(rel);
  }
  return out.sort();
}

function firstHeading(markdown: string): string | undefined {
  return markdown.match(/^#\s+(.+)$/m)?.[1]?.trim();
}

async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}
