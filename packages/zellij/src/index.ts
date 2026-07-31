/**
 * @shebka/zellij — public API.
 *
 * A wrapper app should need almost nothing beyond `createZellijPages` and
 * `createZellijRootLayout` (spec §9).
 */

export { createZellijPages } from './render/page';
export type { ZellijPageProps } from './render/page';
export { createZellijRootLayout } from './render/root-layout';
export { createZellijSitemap, createZellijRobots } from './render/seo';

export { buildSearchIndex, indexGuidePage, mdxToText } from './search/build-index';
export type { SearchEntry } from './search/build-index';

export { THEMES } from './themes/tokens';
export type { ThemeTokens, Palette } from './themes/tokens';

export * from './config/schema';
export { loadZellijRuntime, resolveContentPaths } from './config/runtime';
export type { ZellijRuntime, ZellijRuntimeOptions } from './config/runtime';

export * from './content/types';
export * from './content/paths';
export { FileSystemContentSource } from './content/filesystem-source';
export { loadSite } from './content/site';
export type { Site } from './content/site';

export {
  registerSection,
  getSection,
  registeredSectionTypes,
  baseSectionSchema,
} from './sections/registry';
export type { SectionDefinition } from './sections/registry';
export * from './sections/schemas';

export { mdxComponents } from './mdx/components';
export { mdxOptions } from './mdx/pipeline';
export { extractToc, slugifyHeading } from './guide/toc';
export type { TocEntry } from './guide/toc';
export { buildBreadcrumbs, findNeighbours, flattenSidebar } from './guide/navigation';

export { ZellijContentError, formatZodError } from './errors';
