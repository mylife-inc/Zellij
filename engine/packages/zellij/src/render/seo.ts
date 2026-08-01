import type { MetadataRoute } from 'next';
import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';
import { withBasePath } from '../content/paths';

/**
 * `app/sitemap.ts` for a wrapper app:
 *
 * ```ts
 * import { createZellijSitemap } from '@shebka/zellij';
 * export default createZellijSitemap();
 * ```
 *
 * Emits every generated route. Requires `siteUrl` in config — without an
 * origin a sitemap cannot contain the absolute URLs the format requires.
 */
export function createZellijSitemap(options: ZellijRuntimeOptions = {}) {
  return async function sitemap(): Promise<MetadataRoute.Sitemap> {
    const { site } = await loadZellijRuntime(options);
    if (!site.config.siteUrl) return [];

    const origin = site.config.siteUrl.replace(/\/+$/, '');

    return [...site.pages, ...site.guidePages].map((page) => ({
      url: `${origin}${withBasePath(page.route, site.config.basePath)}`,
      changeFrequency: 'weekly' as const,
      // Landing page first, then marketing, then guide.
      priority: page.route === '/' ? 1 : page.route.startsWith('/guide') ? 0.5 : 0.8,
    }));
  };
}

/**
 * `app/robots.ts` for a wrapper app:
 *
 * ```ts
 * import { createZellijRobots } from '@shebka/zellij';
 * export default createZellijRobots();
 * ```
 */
export function createZellijRobots(options: ZellijRuntimeOptions = {}) {
  return async function robots(): Promise<MetadataRoute.Robots> {
    const { site } = await loadZellijRuntime(options);
    const origin = site.config.siteUrl?.replace(/\/+$/, '');

    return {
      rules: { userAgent: '*', allow: '/' },
      ...(origin ? { sitemap: `${origin}${withBasePath('/sitemap.xml', site.config.basePath)}` } : {}),
    };
  };
}
