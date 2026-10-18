import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import { parseFrontmatter } from './frontmatter';

import { ZellijContentError, aggregateErrors, formatZodError } from '../errors';
import {
  guideFrontmatterSchema,
  sidebarFileSchema,
  type AuthoredSidebar,
  type SidebarEntry,
} from './model-schema';
import { deriveLabel, slugifyPath, splitOrderPrefix, stripExtension } from './naming';
import { normalizeRoute } from './paths';
import type { GuidePage, GuideTree, SidebarNode } from './types';
import { portableDocumentSchema } from './portable-document';
import { exists, walk } from './fs-utils';

export const SIDEBAR_FILE = '_sidebar.yaml';

/**
 * Reads `_sidebar.yaml` if the folder has one. Its presence is what makes a
 * folder a guide folder (design §8), so a missing file is not an error — it
 * simply means this is not one.
 */
export async function readSidebarFile(dir: string): Promise<AuthoredSidebar | undefined> {
  const file = path.join(dir, SIDEBAR_FILE);
  if (!(await exists(file))) return undefined;

  const raw = await readFile(file, 'utf8');
  let parsed: unknown;
  try {
    parsed = parseYaml(raw);
  } catch (cause) {
    throw new ZellijContentError(
      `Could not parse YAML in ${file}:\n  · ${(cause as Error).message}`,
      file,
    );
  }

  const result = sidebarFileSchema.safeParse(parsed ?? []);
  if (!result.success) throw formatZodError(file, result.error);
  return result.data;
}

export interface LoadGuideFolderOptions {
  /** Absolute path to the folder. */
  dir: string;
  /** Folder path relative to the content root — its reference name. */
  relDir: string;
  authored: AuthoredSidebar;
  /** Route to mount at, overriding the one derived from the folder name. */
  routeOverride?: string;
  onWarning?: (message: string) => void;
}

/**
 * Loads one guide folder into a tree: its pages, its resolved navigation, and
 * where it mounts. Shared by both content sources so the legacy `guide/` folder
 * and a folder named `03-appendix` are read by exactly the same code.
 */
export async function loadGuideFolder(options: LoadGuideFolderOptions): Promise<GuideTree> {
  const { dir, relDir, authored, onWarning } = options;

  const { order: prefixOrder, rest } = splitOrderPrefix(path.basename(relDir));
  const route =
    options.routeOverride ??
    (authored.route ? normalizeRoute(authored.route) : `/${slugifyPath(relDir)}`);
  const title = authored.title ?? deriveLabel(rest);

  const files = await walk(dir, (name) => (/\.mdx?$/i.test(name) || name.endsWith('.zeldoc.json')) && !name.startsWith('_'));

  const pages: GuidePage[] = [];
  /** Authored slug (on-disk path minus extension) → page, for sidebar lookup. */
  const bySlug = new Map<string, GuidePage>();
  const errors: ZellijContentError[] = [];

  for (const relFile of files) {
    const label = `${relDir}/${relFile}`;
    try {
      const raw = await readFile(path.join(dir, relFile), 'utf8');
      if (relFile.endsWith('.zeldoc.json')) {
        const result = portableDocumentSchema.safeParse(JSON.parse(raw));
        if (!result.success) throw formatZodError(label, result.error);
        const portable = result.data;
        const page: GuidePage = { route: normalizeRoute(portable.route), sourceFile: label,
          title: portable.title, description: portable.description, body: '', portable, guide: route };
        pages.push(page);
        bySlug.set(relFile.slice(0, -'.zeldoc.json'.length), page);
        continue;
      }
      const { data, content } = parseFrontmatter(raw);

      const fm = guideFrontmatterSchema.safeParse(data ?? {});
      if (!fm.success) throw formatZodError(label, fm.error);

      const slug = stripExtension(relFile);
      const pageTitle = fm.data.title ?? firstHeading(content) ?? deriveLabel(path.basename(slug));

      const page: GuidePage = {
        route: fm.data.route ? normalizeRoute(fm.data.route) : pageRoute(route, slug),
        sourceFile: label,
        title: pageTitle,
        ...(fm.data.description ? { description: fm.data.description } : {}),
        // The layout renders the title as the page's h1, so a leading h1 in the
        // body is removed rather than rendered twice. This keeps the title
        // authoritative in one place — used for <title>, the sidebar,
        // breadcrumbs, search, and the visible heading alike.
        body: stripLeadingH1(content),
        guide: route,
      };

      pages.push(page);
      bySlug.set(slug, page);
    } catch (err) {
      errors.push(
        err instanceof ZellijContentError
          ? err
          : new ZellijContentError(`Failed to load ${label}: ${(err as Error).message}`, label),
      );
    }
  }

  if (errors.length > 0) {
    throw aggregateErrors(`Zellij could not load ${errors.length} page(s) in ${relDir}:`, errors);
  }

  const sidebar = resolveSidebar({
    entries: authored.nav,
    bySlug,
    pages,
    treeRoute: route,
    relDir,
    onWarning,
  });

  return {
    kind: 'guide',
    name: relDir,
    route,
    title,
    ...(authored.description ? { description: authored.description } : {}),
    ...(authored.order ?? prefixOrder ? { order: authored.order ?? prefixOrder } : {}),
    sourceDir: relDir,
    sidebar,
    pages,
  };
}

/** `index` maps to the tree root; everything else hangs below it. */
function pageRoute(treeRoute: string, slug: string): string {
  const trimmed = slug.replace(/(^|\/)index$/, '');
  if (trimmed === '') return treeRoute;
  return normalizeRoute(`${treeRoute === "/" ? "" : treeRoute}/${slugifyPath(trimmed)}`);
}

interface ResolveSidebarOptions {
  entries: readonly SidebarEntry[];
  bySlug: ReadonlyMap<string, GuidePage>;
  pages: readonly GuidePage[];
  treeRoute: string;
  relDir: string;
  onWarning?: (message: string) => void;
}

/**
 * Turns authored entries into resolved links. Entries with no matching file are
 * a build error; files missing from the navigation are a warning (design §11).
 */
function resolveSidebar(options: ResolveSidebarOptions): SidebarNode[] {
  const { entries, bySlug, pages, treeRoute, relDir, onWarning } = options;
  const referenced = new Set<string>();
  const missing: string[] = [];

  function resolveLeaf(slug: string): SidebarNode | undefined {
    const page = bySlug.get(slug) ?? bySlug.get(stripExtension(slug));
    if (!page) {
      missing.push(`  · "${slug}" → expected ${relDir}/${slug}.mdx (no such file)`);
      return undefined;
    }
    referenced.add(page.route);
    return { kind: 'link', label: page.title, route: page.route };
  }

  /**
   * Walks the authored tree to any depth. An earlier version resolved only
   * string entries one level down, so a group nested inside a group was
   * silently discarded — valid per the schema, gone from the output, with no
   * error to say so.
   */
  function resolveNodes(nodes: readonly SidebarEntry[]): SidebarNode[] {
    const out: SidebarNode[] = [];

    for (const node of nodes) {
      if (typeof node === 'string') {
        const leaf = resolveLeaf(node);
        if (leaf) out.push(leaf);
        continue;
      }

      if ('href' in node) {
        out.push({ kind: 'link', label: node.label, route: node.href, external: true });
        continue;
      }
      if ('page' in node) {
        const leaf = resolveLeaf(node.page);
        if (leaf) out.push({ ...leaf, label: node.label });
        continue;
      }
      const children = resolveNodes(node.items);
      // An empty group would render as an unopenable heading.
      if (children.length > 0) out.push({ kind: 'group', label: node.label, children });
    }

    return out;
  }

  const tree = resolveNodes(entries);

  if (missing.length > 0) {
    throw new ZellijContentError(
      `${relDir}/${SIDEBAR_FILE} references pages that do not exist:\n${missing.join('\n')}`,
      `${relDir}/${SIDEBAR_FILE}`,
    );
  }

  for (const page of pages) {
    if (!referenced.has(page.route) && page.route !== treeRoute) {
      onWarning?.(
        `${page.sourceFile} is not listed in ${relDir}/${SIDEBAR_FILE} — the page is still generated, but nothing links to it.`,
      );
    }
  }

  return tree;
}

function firstHeading(markdown: string): string | undefined {
  const match = markdown.match(/^#\s+(.+)$/m);
  return match?.[1]?.trim();
}

/**
 * Removes a document-opening h1. Only a leading one: an `#` appearing later is
 * either intentional or inside a code fence, and neither should be touched.
 */
function stripLeadingH1(markdown: string): string {
  return markdown.replace(/^\s*#\s+.+?(\n|$)/, '').replace(/^\n+/, '');
}
