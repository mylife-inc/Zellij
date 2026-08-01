import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { z } from 'zod';
import { parse as parseYaml } from 'yaml';

import { ZellijContentError, aggregateErrors, formatZodError } from '../errors';
import {
  analyticsSchema,
  announcementSchema,
  ctaSchema,
  motionPresetSchema,
  overridesSchema,
  pageLayoutSchema,
  siteConfigSchema,
  socialSchema,
  themeNameSchema,
  type SiteConfig,
} from '../config/schema';
import { validateSections, type AuthoredMenuItem } from './model-schema';
import { assetUrl, normalizeRoute } from './paths';
import { loadGuideFolder, readSidebarFile } from './guide-folder';
import { exists, walk } from './fs-utils';
import type { ContentSource, GuideTree, ResolvedAsset, SectionsPage } from './types';

/* --------------------------------------------------- the old config file */

const legacyNavLeafSchema = z.object({
  label: z.string().min(1),
  href: z.string().min(1),
  description: z.string().optional(),
  icon: z.string().optional(),
});

const legacyNavItemSchema = z.object({
  label: z.string().min(1),
  href: z.string().min(1).optional(),
  children: z.array(legacyNavLeafSchema).min(1).optional(),
  mega: z
    .object({
      groups: z
        .array(z.object({ heading: z.string().min(1), items: z.array(legacyNavLeafSchema).min(1) }))
        .min(1),
    })
    .optional(),
});

export const legacyConfigSchema = z.object({
  product: z.object({
    name: z.string().min(1),
    tagline: z.string().optional(),
    logo: z.string().optional(),
    logoDark: z.string().optional(),
  }),
  content: z.string().min(1).default('./content'),
  basePath: z.string().default(''),
  siteUrl: z.url().optional(),
  ogImage: z.string().optional(),
  theme: themeNameSchema.default('paper'),
  mode: z.enum(['light', 'dark', 'system']).default('system'),
  motion: motionPresetSchema.default('subtle'),
  overrides: overridesSchema.default({}),
  nav: z
    .object({
      sticky: z.boolean().default(true),
      align: z.enum(['left', 'center']).default('left'),
      hideOnScrollDown: z.boolean().default(false),
      transparentOverHero: z.boolean().default(true),
      scrollThreshold: z.number().int().nonnegative().default(80),
      themeToggle: z.boolean().default(false),
      themeSwitcher: z.boolean().default(false),
      search: z.boolean().default(false),
      links: z.array(legacyNavItemSchema).default([]),
      cta: z.array(ctaSchema).default([]),
      social: socialSchema.optional(),
      announcement: announcementSchema.optional(),
    })
    .prefault({}),
  footer: z
    .object({
      columns: z
        .array(
          z.object({
            heading: z.string().min(1),
            links: z.array(z.object({ label: z.string().min(1), href: z.string().min(1) })).min(1),
          }),
        )
        .default([]),
      legal: z.string().optional(),
      social: socialSchema.optional(),
    })
    .prefault({}),
  analytics: analyticsSchema.prefault({}),
});

export type LegacyConfig = z.infer<typeof legacyConfigSchema>;

const legacyPageSchema = z.object({
  title: z.string().min(1),
  description: z.string().min(1),
  route: z.string().optional(),
  layout: pageLayoutSchema.default('stack'),
  theme: themeNameSchema.optional(),
  motion: motionPresetSchema.optional(),
  navHighlight: z.string().optional(),
  sections: z.array(z.record(z.string(), z.unknown())).default([]),
});

/** Restates the old config in the current shape, so nothing downstream forks. */
export function toSiteConfig(legacy: LegacyConfig): SiteConfig {
  return siteConfigSchema.parse({
    name: legacy.product.name,
    ...(legacy.product.tagline ? { tagline: legacy.product.tagline } : {}),
    brand: legacy.product.logo
      ? {
          logo: {
            src: legacy.product.logo,
            ...(legacy.product.logoDark ? { dark: legacy.product.logoDark } : {}),
            alt: legacy.product.name,
          },
        }
      : {},
    theme: legacy.theme,
    mode: legacy.mode,
    motion: legacy.motion,
    overrides: legacy.overrides,
    basePath: legacy.basePath,
    ...(legacy.siteUrl ? { siteUrl: legacy.siteUrl } : {}),
    ...(legacy.ogImage ? { ogImage: legacy.ogImage } : {}),
    nav: {
      sticky: legacy.nav.sticky,
      align: legacy.nav.align,
      hideOnScrollDown: legacy.nav.hideOnScrollDown,
      transparentOverHero: legacy.nav.transparentOverHero,
      scrollThreshold: legacy.nav.scrollThreshold,
      themeToggle: legacy.nav.themeToggle,
      themeSwitcher: legacy.nav.themeSwitcher,
      search: legacy.nav.search,
      cta: legacy.nav.cta,
      ...(legacy.nav.social ? { social: legacy.nav.social } : {}),
      ...(legacy.nav.announcement ? { announcement: legacy.nav.announcement } : {}),
    },
    footer: legacy.footer,
    analytics: legacy.analytics,
  });
}

/**
 * `nav.links` addressed pages by href. `url:` accepts internal paths as well as
 * external ones, so the conversion is direct and no route is re-derived.
 */
export function toMenu(legacy: LegacyConfig): AuthoredMenuItem[] {
  return legacy.nav.links.map((item) => {
    if (item.mega) {
      return {
        name: item.label,
        wide: true,
        items: item.mega.groups.map((group) => ({
          name: group.heading,
          items: group.items.map(toMenuLeaf),
        })),
      };
    }
    if (item.children) return { name: item.label, items: item.children.map(toMenuLeaf) };
    return { name: item.label, url: item.href! };
  });
}

function toMenuLeaf(leaf: z.infer<typeof legacyNavLeafSchema>): AuthoredMenuItem {
  return {
    name: leaf.label,
    url: leaf.href,
    ...(leaf.description ? { description: leaf.description } : {}),
    ...(leaf.icon ? { icon: leaf.icon } : {}),
  };
}

/* --------------------------------------------------------- the old layout */

export interface LegacyContentSourceOptions {
  /** Absolute path to the content directory. */
  contentDir: string;
  /** Absolute path to zellij.config.yaml. */
  configFile: string;
  onWarning?: (message: string) => void;
}

/**
 * Reads the pre-content-model layout: `zellij.config.yaml` beside the app,
 * `marketing/*.yaml` with sections written inline, and a single `guide/` folder.
 *
 * It produces the current internal model, so the rest of the engine is unaware
 * that two layouts exist. Retained for one release (design §12).
 */
export class LegacyContentSource implements ContentSource {
  private legacyCache?: LegacyConfig;

  constructor(private readonly options: LegacyContentSourceOptions) {}

  private warn(message: string): void {
    this.options.onWarning?.(message);
  }

  async loadLegacyConfig(): Promise<LegacyConfig> {
    if (this.legacyCache) return this.legacyCache;

    const { configFile } = this.options;
    if (!(await exists(configFile))) {
      throw new ZellijContentError(
        `Config file not found: ${configFile}\n` +
          `  A content root with no site.yaml needs at least one guide folder, ` +
          `or a zellij.config.yaml at the app root for the legacy layout.`,
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

    const result = legacyConfigSchema.safeParse(parsed ?? {});
    if (!result.success) throw formatZodError(configFile, result.error);

    this.legacyCache = result.data;
    return result.data;
  }

  async loadSiteConfig(): Promise<SiteConfig> {
    return toSiteConfig(await this.loadLegacyConfig());
  }

  async loadMenu(): Promise<AuthoredMenuItem[] | undefined> {
    const legacy = await this.loadLegacyConfig();
    return legacy.nav.links.length > 0 ? toMenu(legacy) : undefined;
  }

  async listPages(): Promise<SectionsPage[]> {
    const dir = path.join(this.options.contentDir, 'marketing');
    const files = await walk(dir, (name) => /\.ya?ml$/i.test(name));

    const pages: SectionsPage[] = [];
    const errors: ZellijContentError[] = [];

    for (const relFile of files) {
      const label = `marketing/${relFile}`;
      try {
        const raw = await readFile(path.join(dir, relFile), 'utf8');
        let parsed: unknown;
        try {
          parsed = parseYaml(raw);
        } catch (cause) {
          throw new ZellijContentError(
            `Could not parse YAML in ${label}:\n  · ${(cause as Error).message}`,
            label,
          );
        }

        const fm = legacyPageSchema.safeParse(parsed ?? {});
        if (!fm.success) throw formatZodError(label, fm.error);

        const name = relFile.replace(/\.ya?ml$/i, '');
        const route = fm.data.route ? normalizeRoute(fm.data.route) : legacyRoute(name);

        pages.push({
          kind: 'sections',
          name,
          route,
          ...(fm.data.route ? { routeExplicit: true } : {}),
          sourceFile: label,
          title: fm.data.title,
          description: fm.data.description,
          layout: fm.data.layout,
          ...(fm.data.theme ? { theme: fm.data.theme } : {}),
          ...(fm.data.motion ? { motion: fm.data.motion } : {}),
          ...(fm.data.navHighlight ? { navHighlight: fm.data.navHighlight } : {}),
          sections: validateSections(label, fm.data.sections),
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
      throw aggregateErrors(`Zellij could not load ${errors.length} marketing page(s):`, errors);
    }

    return pages;
  }

  /** The legacy layout has at most one guide tree, always mounted at /guide. */
  async listGuides(): Promise<GuideTree[]> {
    const dir = path.join(this.options.contentDir, 'guide');
    if (!(await exists(dir))) return [];

    const authored = (await readSidebarFile(dir)) ?? { nav: [] };
    const tree = await loadGuideFolder({
      dir,
      relDir: 'guide',
      authored,
      routeOverride: '/guide',
      onWarning: (message) => this.warn(message),
    });

    return tree.pages.length > 0 ? [tree] : [];
  }

  async resolveAsset(relPath: string): Promise<ResolvedAsset> {
    const clean = relPath.replace(/^\.?\//, '');
    const absolutePath = path.join(this.options.contentDir, clean);
    if (!(await exists(absolutePath))) {
      throw new ZellijContentError(`Asset not found: ${relPath}\n  Looked in ${absolutePath}`);
    }
    return { source: relPath, url: assetUrl(relPath), absolutePath };
  }
}

/** `landing.yaml` and `index.yaml` were both the "/" page. */
function legacyRoute(name: string): string {
  if (name === 'landing' || name === 'index') return '/';
  return `/${name}`;
}
