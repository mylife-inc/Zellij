/**
 * @shebka/zellij — public API.
 *
 * A wrapper app should need almost nothing beyond `createZellijPages` and
 * `createZellijRootLayout` (spec §9).
 */

export { createZellijPages } from './render/page';
export type { ZellijPageProps } from './render/page';
export { createZellijRootLayout } from './render/root-layout';

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

export { ZellijContentError, formatZodError } from './errors';
