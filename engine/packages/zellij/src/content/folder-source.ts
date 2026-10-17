import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import { parseFrontmatter } from './frontmatter';

import { ZellijContentError, aggregateErrors, formatZodError } from '../errors';
import { KNOWN_SITE_KEYS, siteConfigSchema, type SiteConfig } from '../config/schema';
import {
  menuFileSchema,
  normalizeSectionRef,
  pagesFileSchema,
  sectionFrontmatterSchema,
  validateSection,
  type AuthoredMenuItem,
  type AuthoredPage,
} from './model-schema';
import { deriveLabel, sectionName, slugifyPath } from './naming';
import { assetUrl, normalizeRoute } from './paths';
import { loadGuideFolder, readSidebarFile } from './guide-folder';
import { exists, subdirectories, walk } from './fs-utils';
import type {
  ContentSource,
  GuideTree,
  ResolvedAsset,
  SectionExample,
  SectionsPage,
} from './types';

export const SITE_FILE = 'site.yaml';
export const MENU_FILE = 'menu.yaml';
export const PAGES_FILE = 'pages.yaml';
export const SECTIONS_DIR = 'sections';
export const EXAMPLES_DIR = 'examples';
export const ASSETS_DIR = 'assets';

/** Folders that are part of the layout and so are never guide folders. */
const RESERVED_DIRS = new Set([
  SECTIONS_DIR,
  EXAMPLES_DIR,
  ASSETS_DIR,
  'public',
  'node_modules',
]);

export interface FolderContentSourceOptions {
  /** Absolute path to the content root. */
  contentDir: string;
  /** Overrides `basePath` from site.yaml; the wrapper app owns where it mounts. */
  basePath?: string;
  onWarning?: (message: string) => void;
}

/** A section file after reading, before any page has referenced it. */
interface SectionFile {
  name: string;
  sourceFile: string;
  raw: Record<string, unknown>;
}

/**
 * Reads a content root laid out per the content-model design: `site.yaml`,
 * `menu.yaml`, `pages.yaml`, `sections/`, `assets/`, and any number of guide
 * folders identified by their `_sidebar.yaml`.
 *
 * Every file is optional. A folder containing nothing but one guide folder
 * builds a complete site.
 */
export class FolderContentSource implements ContentSource {
  private configCache?: SiteConfig;
  private sectionCache?: Map<string, SectionFile>;

  constructor(private readonly options: FolderContentSourceOptions) {}

  private warn(message: string): void {
    this.options.onWarning?.(message);
  }

  private file(...parts: string[]): string {
    return path.join(this.options.contentDir, ...parts);
  }

  async loadSiteConfig(): Promise<SiteConfig> {
    if (this.configCache) return this.configCache;

    const target = this.file(SITE_FILE);
    const parsed = (await exists(target)) ? await readYaml(target, SITE_FILE) : {};

    const record = (parsed ?? {}) as Record<string, unknown>;

    // Unknown top-level keys warn rather than fail, for forward compatibility.
    for (const key of Object.keys(record)) {
      if (!KNOWN_SITE_KEYS.includes(key)) {
        this.warn(`${SITE_FILE}: unknown key "${key}" ignored (newer schema, or a typo?)`);
      }
    }

    // A content root with no site.yaml still has a name: the folder's own.
    if (record['name'] === undefined) {
      record['name'] = deriveLabel(path.basename(this.options.contentDir));
    }

    const result = siteConfigSchema.safeParse(record);
    if (!result.success) throw formatZodError(SITE_FILE, result.error);

    const config = result.data;
    if (this.options.basePath !== undefined) config.basePath = this.options.basePath;

    this.configCache = config;
    return config;
  }

  async loadMenu(): Promise<AuthoredMenuItem[] | undefined> {
    const target = this.file(MENU_FILE);
    if (!(await exists(target))) return undefined;

    const parsed = await readYaml(target, MENU_FILE);
    const result = menuFileSchema.safeParse(parsed ?? []);
    if (!result.success) throw formatZodError(MENU_FILE, result.error);
    return result.data;
  }

  /**
   * Reads every file under `sections/` once. Keyed by its path below the
   * directory with the extension dropped, which is the name pages refer to.
   */
  private async loadSectionFiles(): Promise<Map<string, SectionFile>> {
    if (this.sectionCache) return this.sectionCache;

    const dir = this.file(SECTIONS_DIR);
    const files = await walk(dir, (name) => /\.(ya?ml|mdx?)$/i.test(name));
    const map = new Map<string, SectionFile>();
    const errors: ZellijContentError[] = [];

    for (const relFile of files) {
      const label = `${SECTIONS_DIR}/${relFile}`;
      const name = sectionName(relFile);
      try {
        const raw = await readFile(path.join(dir, relFile), 'utf8');
        map.set(name, {
          name,
          sourceFile: label,
          raw: /\.mdx?$/i.test(relFile)
            ? readProseSection(label, raw)
            : ((await parseYamlString(label, raw)) as Record<string, unknown>),
        });
      } catch (err) {
        errors.push(
          err instanceof ZellijContentError
            ? err
            : new ZellijContentError(`Failed to load ${label}: ${(err as Error).message}`, label),
        );
      }
    }

    if (errors.length > 0) {
      throw aggregateErrors(`Zellij could not load ${errors.length} section file(s):`, errors);
    }

    this.sectionCache = map;
    return map;
  }

  async listPages(): Promise<SectionsPage[]> {
    const target = this.file(PAGES_FILE);
    const sections = await this.loadSectionFiles();

    if (!(await exists(target))) {
      if (sections.size > 0) {
        this.warn(
          `${SECTIONS_DIR}/ contains ${sections.size} file(s) but there is no ${PAGES_FILE}, so no page uses them.`,
        );
      }
      return [];
    }

    const parsed = await readYaml(target, PAGES_FILE);
    const result = pagesFileSchema.safeParse(parsed ?? []);
    if (!result.success) throw formatZodError(PAGES_FILE, result.error);

    const pages: SectionsPage[] = [];
    const errors: ZellijContentError[] = [];
    const used = new Set<string>();

    for (const authored of result.data) {
      try {
        pages.push(this.buildPage(authored, sections, used));
      } catch (err) {
        errors.push(
          err instanceof ZellijContentError
            ? err
            : new ZellijContentError(
                `Failed to build page "${authored.name}": ${(err as Error).message}`,
                PAGES_FILE,
              ),
        );
      }
    }

    if (errors.length > 0) {
      throw aggregateErrors(`Zellij could not build ${errors.length} page(s):`, errors);
    }

    for (const section of sections.values()) {
      if (!used.has(section.name)) {
        this.warn(`${section.sourceFile} is not referenced by any page in ${PAGES_FILE}.`);
      }
    }

    return pages;
  }

  private buildPage(
    authored: AuthoredPage,
    sections: ReadonlyMap<string, SectionFile>,
    used: Set<string>,
  ): SectionsPage {
    const where = `${PAGES_FILE} › ${authored.name}`;
    const sectionSources: string[] = [];
    const resolved = authored.sections.map((ref, index) => {
      const { section: name, ...overrides } = normalizeSectionRef(ref);
      const file = sections.get(name);

      if (!file) {
        const available = [...sections.keys()].sort();
        throw new ZellijContentError(
          `${where}: sections[${index}] refers to "${name}", which has no file.\n` +
            `  Expected ${SECTIONS_DIR}/${name}.yaml (or .md/.mdx).` +
            (available.length > 0 ? `\n  Available: ${available.join(', ')}` : ''),
          PAGES_FILE,
        );
      }

      used.add(name);
      sectionSources.push(file.sourceFile);
      // Overrides are merged before validation so a type's own defaults apply
      // to the merged result rather than to the file's value alone.
      return validateSection(`${where} › ${name}`, { ...file.raw, ...overrides });
    });

    return {
      kind: 'sections',
      name: authored.name,
      route: authored.route ? normalizeRoute(authored.route) : `/${slugifyPath(authored.name)}`,
      ...(authored.route ? { routeExplicit: true } : {}),
      sourceFile: PAGES_FILE,
      title: authored.title ?? deriveLabel(authored.name),
      description: authored.description ?? '',
      layout: authored.layout,
      ...(authored.theme ? { theme: authored.theme } : {}),
      ...(authored.motion ? { motion: authored.motion } : {}),
      ...(authored.navHighlight ? { navHighlight: authored.navHighlight } : {}),
      sections: resolved,
      sectionSources,
    };
  }

  /**
   * Every top-level subdirectory holding a `_sidebar.yaml`. Only the top level:
   * a guide folder's own subdirectories hold its pages, not further trees.
   */
  async listGuides(): Promise<GuideTree[]> {
    const names = await subdirectories(this.options.contentDir);
    const trees: GuideTree[] = [];

    for (const name of names) {
      if (RESERVED_DIRS.has(name)) continue;

      const dir = this.file(name);
      const authored = await readSidebarFile(dir);
      if (!authored) {
        this.warn(
          `${name}/ has no _sidebar.yaml, so it is not part of the site. Add one to render it as a guide.`,
        );
        continue;
      }

      trees.push(
        await loadGuideFolder({
          dir,
          relDir: name,
          authored,
          onWarning: (message) => this.warn(message),
        }),
      );
    }

    return trees;
  }

  /**
   * Reads every YAML file under `examples/`. These are not part of any page:
   * they exist so a guide can show a section's source next to what that source
   * renders. Validating them here means a documentation example that no longer
   * parses fails the build, rather than quietly documenting something the
   * engine would reject.
   */
  async listExamples(): Promise<SectionExample[]> {
    const dir = this.file(EXAMPLES_DIR);
    const files = await walk(dir, (name) => /\.ya?ml$/i.test(name));

    const examples: SectionExample[] = [];
    const errors: ZellijContentError[] = [];

    for (const relFile of files) {
      const label = `${EXAMPLES_DIR}/${relFile}`;
      try {
        const source = await readFile(path.join(dir, relFile), 'utf8');
        const raw = (await parseYamlString(label, source)) as Record<string, unknown>;
        examples.push({
          name: sectionName(relFile),
          sourceFile: label,
          source,
          section: validateSection(label, raw),
        });
      } catch (err) {
        errors.push(
          err instanceof ZellijContentError
            ? err
            : new ZellijContentError(`Failed to load ${label}: ${(err as Error).message}`, label),
        );
      }
    }

    if (errors.length > 0) {
      throw aggregateErrors(`${errors.length} example file(s) could not be loaded:`, errors);
    }
    return examples;
  }

  async resolveAsset(relPath: string): Promise<ResolvedAsset> {
    const clean = relPath.replace(/^\.?\//, '');
    const absolutePath = this.file(clean);
    if (!(await exists(absolutePath))) {
      throw new ZellijContentError(`Asset not found: ${relPath}\n  Looked in ${absolutePath}`);
    }
    return { source: relPath, url: assetUrl(relPath), absolutePath };
  }
}

/**
 * A `.md`/`.mdx` section: frontmatter carries presentation and may name a
 * different type, and the body is the content. Defaults to `markdown-prose`,
 * which is what a plain prose file is.
 */
function readProseSection(label: string, raw: string): Record<string, unknown> {
  const { data, content } = parseFrontmatter(raw);
  const fm = sectionFrontmatterSchema.safeParse(data ?? {});
  if (!fm.success) throw formatZodError(label, fm.error);

  const { type, ...presentation } = fm.data;
  return { ...data, ...presentation, type: type ?? 'markdown-prose', body: content.trim() };
}

async function readYaml(absolute: string, label: string): Promise<unknown> {
  return parseYamlString(label, await readFile(absolute, 'utf8'));
}

async function parseYamlString(label: string, raw: string): Promise<unknown> {
  try {
    return parseYaml(raw);
  } catch (cause) {
    throw new ZellijContentError(
      `Could not parse YAML in ${label}:\n  · ${(cause as Error).message}`,
      label,
    );
  }
}
