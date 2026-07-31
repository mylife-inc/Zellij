import { readFile, readdir, stat } from 'node:fs/promises';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';
import matter from 'gray-matter';

import { ZellijContentError, aggregateErrors, formatZodError } from '../errors';
import { KNOWN_CONFIG_KEYS, zellijConfigSchema, type ZellijConfig } from '../config/schema';
import {
  guideFrontmatterSchema,
  marketingPageFrontmatterSchema,
  sidebarSchema,
  validateSections,
  type SidebarEntry,
} from './page-schema';
import {
  assetUrl,
  guideRouteFromFile,
  marketingRouteFromFile,
  normalizeRoute,
} from './paths';
import type { ContentSource, GuidePage, MarketingPage, ResolvedAsset } from './types';

export interface FileSystemContentSourceOptions {
  /** Absolute path to the content directory. */
  contentDir: string;
  /** Absolute path to zellij.config.yaml. */
  configFile: string;
  /** Emitted for non-fatal problems (spec §7.1 orphan pages, §10 unknown keys). */
  onWarning?: (message: string) => void;
}

async function exists(p: string): Promise<boolean> {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
}

/** Recursively lists files under `dir` matching `test`, returned dir-relative. */
async function walk(dir: string, test: (name: string) => boolean): Promise<string[]> {
  if (!(await exists(dir))) return [];
  const out: string[] = [];

  async function recurse(current: string, prefix: string): Promise<void> {
    const entries = await readdir(current, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name.startsWith('.')) continue;
      const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        await recurse(path.join(current, entry.name), rel);
      } else if (test(entry.name)) {
        out.push(rel);
      }
    }
  }

  await recurse(dir, '');
  return out.sort();
}

export class FileSystemContentSource implements ContentSource {
  private configCache?: ZellijConfig;

  constructor(private readonly options: FileSystemContentSourceOptions) {}

  private warn(message: string): void {
    this.options.onWarning?.(message);
  }

  async loadConfig(): Promise<ZellijConfig> {
    if (this.configCache) return this.configCache;

    const { configFile } = this.options;
    if (!(await exists(configFile))) {
      throw new ZellijContentError(
        `Config file not found: ${configFile}\n` +
          `  Zellij needs a zellij.config.yaml at your app root (spec §8).`,
        configFile,
      );
    }

    const raw = await readFile(configFile, 'utf8');
    let parsed: unknown;
    try {
      parsed = parseYaml(raw);
    } catch (cause) {
      throw new ZellijContentError(
        `Could not parse YAML in ${configFile}:\n  · ${(cause as Error).message}`,
        configFile,
      );
    }

    // Unknown top-level keys warn rather than fail, for forward compatibility (spec §10).
    if (parsed && typeof parsed === 'object') {
      for (const key of Object.keys(parsed as Record<string, unknown>)) {
        if (!KNOWN_CONFIG_KEYS.includes(key)) {
          this.warn(
            `${configFile}: unknown config key "${key}" ignored (newer schema, or a typo?)`,
          );
        }
      }
    }

    const result = zellijConfigSchema.safeParse(parsed ?? {});
    if (!result.success) throw formatZodError(configFile, result.error);

    this.configCache = result.data;
    return result.data;
  }

  async listMarketingPages(): Promise<MarketingPage[]> {
    const dir = path.join(this.options.contentDir, 'marketing');
    const files = await walk(dir, (n) => /\.ya?ml$/i.test(n));

    if (files.length === 0) {
      throw new ZellijContentError(
        `No marketing pages found in ${dir}\n` +
          `  At minimum, Zellij needs marketing/landing.yaml (the "/" page, spec §4).`,
      );
    }

    const pages: MarketingPage[] = [];
    const errors: ZellijContentError[] = [];

    for (const relFile of files) {
      const abs = path.join(dir, relFile);
      const label = `marketing/${relFile}`;
      try {
        const raw = await readFile(abs, 'utf8');
        let parsed: unknown;
        try {
          parsed = parseYaml(raw);
        } catch (cause) {
          throw new ZellijContentError(
            `Could not parse YAML in ${label}:\n  · ${(cause as Error).message}`,
            label,
          );
        }

        const fm = marketingPageFrontmatterSchema.safeParse(parsed ?? {});
        if (!fm.success) throw formatZodError(label, fm.error);

        const sections = validateSections(label, fm.data.sections);
        const route = fm.data.route
          ? normalizeRoute(fm.data.route)
          : marketingRouteFromFile(relFile);

        pages.push({
          route,
          sourceFile: label,
          title: fm.data.title,
          description: fm.data.description,
          layout: fm.data.layout,
          ...(fm.data.theme ? { theme: fm.data.theme } : {}),
          ...(fm.data.motion ? { motion: fm.data.motion } : {}),
          ...(fm.data.navHighlight ? { navHighlight: fm.data.navHighlight } : {}),
          sections,
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
      throw aggregateErrors(
        `Zellij could not load ${errors.length} marketing page(s):`,
        errors,
      );
    }

    if (!pages.some((p) => p.route === '/')) {
      throw new ZellijContentError(
        `No page resolves to "/". Add marketing/landing.yaml, or set "route: /" on a page (spec §4).`,
      );
    }

    this.assertNoDuplicateRoutes(pages);
    return pages;
  }

  private assertNoDuplicateRoutes(pages: readonly MarketingPage[]): void {
    const seen = new Map<string, string>();
    const clashes: string[] = [];
    for (const page of pages) {
      const prior = seen.get(page.route);
      if (prior) {
        clashes.push(`  · "${page.route}" declared by both ${prior} and ${page.sourceFile}`);
      } else {
        seen.set(page.route, page.sourceFile);
      }
    }
    if (clashes.length > 0) {
      throw new ZellijContentError(`Duplicate marketing routes:\n${clashes.join('\n')}`);
    }
  }

  async loadGuideSidebar(): Promise<SidebarEntry[]> {
    const file = path.join(this.options.contentDir, 'guide', '_sidebar.yaml');
    if (!(await exists(file))) return [];

    const raw = await readFile(file, 'utf8');
    let parsed: unknown;
    try {
      parsed = parseYaml(raw);
    } catch (cause) {
      throw new ZellijContentError(
        `Could not parse YAML in guide/_sidebar.yaml:\n  · ${(cause as Error).message}`,
        'guide/_sidebar.yaml',
      );
    }

    const result = sidebarSchema.safeParse(parsed ?? []);
    if (!result.success) throw formatZodError('guide/_sidebar.yaml', result.error);
    return result.data;
  }

  async listGuidePages(): Promise<GuidePage[]> {
    const dir = path.join(this.options.contentDir, 'guide');
    const files = await walk(dir, (n) => /\.mdx?$/i.test(n) && !n.startsWith('_'));

    const pages: GuidePage[] = [];
    const errors: ZellijContentError[] = [];

    for (const relFile of files) {
      const abs = path.join(dir, relFile);
      const label = `guide/${relFile}`;
      try {
        const raw = await readFile(abs, 'utf8');
        const { data, content } = matter(raw);

        const fm = guideFrontmatterSchema.safeParse(data ?? {});
        if (!fm.success) throw formatZodError(label, fm.error);

        const title = fm.data.title ?? firstHeading(content) ?? deriveTitle(relFile);
        const route = fm.data.route
          ? normalizeRoute(fm.data.route)
          : guideRouteFromFile(relFile);

        pages.push({
          route,
          sourceFile: label,
          title,
          ...(fm.data.description ? { description: fm.data.description } : {}),
          body: content,
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
      throw aggregateErrors(`Zellij could not load ${errors.length} guide page(s):`, errors);
    }

    return pages;
  }

  async resolveAsset(relPath: string): Promise<ResolvedAsset> {
    const clean = relPath.replace(/^\.?\//, '');
    const absolutePath = path.join(this.options.contentDir, clean);
    if (!(await exists(absolutePath))) {
      throw new ZellijContentError(
        `Asset not found: ${relPath}\n  Looked in ${absolutePath}`,
      );
    }
    return { source: relPath, url: assetUrl(relPath), absolutePath };
  }
}

function firstHeading(markdown: string): string | undefined {
  const match = markdown.match(/^#\s+(.+)$/m);
  return match?.[1]?.trim();
}

function deriveTitle(relFile: string): string {
  const base = relFile.replace(/\.mdx?$/i, '').split('/').pop() ?? relFile;
  return base
    .split('-')
    .map((w) => (w.length > 0 ? w[0]!.toUpperCase() + w.slice(1) : w))
    .join(' ');
}
