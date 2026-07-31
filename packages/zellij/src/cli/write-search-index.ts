import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import matter from 'gray-matter';

// Explicit extension: this module runs under plain Node (via the bin script),
// where ESM requires it. The graph is kept to this single hop for that reason.
import { buildSearchIndex } from '../search/build-index.ts';
import type { GuidePage } from '../content/types.ts';

/**
 * Emits the guide search index as a static asset (spec §13.6).
 *
 * Runs as a prebuild step alongside asset syncing, so the index is a plain file
 * the ⌘K modal fetches on first open — nothing is indexed at request time and
 * no search service is required.
 */
export async function writeSearchIndex(appRoot: string = process.cwd()): Promise<void> {
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

  const guidePages = await readGuidePages(path.join(contentDir, 'guide'));
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
 * A minimal guide reader. Deliberately does not go through ContentSource: that
 * module's import graph is written for the bundler, and reproducing it under
 * Node's stricter resolver would mean extension-annotating half the package.
 * Validation still happens during the build proper — this only needs enough
 * structure to index.
 */
async function readGuidePages(guideDir: string): Promise<GuidePage[]> {
  if (!(await exists(guideDir))) return [];

  const files = await walk(guideDir);
  const pages: GuidePage[] = [];

  for (const relFile of files) {
    const raw = await readFile(path.join(guideDir, relFile), 'utf8');
    const { data, content } = matter(raw);
    const frontmatter = data as { title?: string; description?: string; route?: string };

    const withoutExt = relFile.replace(/\.mdx?$/i, '');
    const route =
      frontmatter.route ??
      (withoutExt === 'index' ? '/guide' : `/guide/${withoutExt.replace(/\/index$/, '')}`);

    pages.push({
      route,
      sourceFile: `guide/${relFile}`,
      title: frontmatter.title ?? firstHeading(content) ?? withoutExt,
      ...(frontmatter.description ? { description: frontmatter.description } : {}),
      body: content,
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
  return /^#\s+(.+)$/m.exec(markdown)?.[1]?.trim();
}

async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}
