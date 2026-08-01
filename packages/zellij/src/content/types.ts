import type { PageLayout, MotionPreset, ThemeName, ZellijConfig } from '../config/schema';
import type { AnySection } from '../sections/schemas';
import type { SidebarEntry } from './page-schema';

export interface MarketingPage {
  /** Resolved site route, always leading-slash, never trailing (except "/"). */
  route: string;
  /** Source file, relative to the content dir — used in error messages. */
  sourceFile: string;
  title: string;
  description: string;
  layout: PageLayout;
  theme?: ThemeName;
  motion?: MotionPreset;
  navHighlight?: string;
  sections: AnySection[];
}

export interface GuidePage {
  /** Resolved site route, e.g. "/guide/quickstart". */
  route: string;
  sourceFile: string;
  title: string;
  description?: string;
  /** Raw MDX source; compiled in the render layer (M2), not the loader. */
  body: string;
}

/**
 * A resolved guide navigation tree.
 *
 * Nesting is arbitrary depth because `_sidebar.yaml` allows it — a group may
 * contain groups. Modelling it as a flat section-plus-items pair silently threw
 * that structure away.
 */
export type SidebarNode = SidebarLink | SidebarGroup;

export interface SidebarLink {
  kind: 'link';
  label: string;
  route: string;
}

export interface SidebarGroup {
  kind: 'group';
  label: string;
  children: SidebarNode[];
}

export interface ResolvedAsset {
  /** Content-relative source path as written by the author. */
  source: string;
  /** Public URL to render, already basePath-aware. */
  url: string;
  /** Absolute path on disk (filesystem source only). */
  absolutePath?: string;
}

/**
 * The single seam through which all content is read (spec §10, §13.3).
 * v1 ships only FileSystemContentSource; remote/DB sources implement this later.
 */
export interface ContentSource {
  loadConfig(): Promise<ZellijConfig>;
  listMarketingPages(): Promise<MarketingPage[]>;
  loadGuideSidebar(): Promise<SidebarEntry[]>;
  listGuidePages(): Promise<GuidePage[]>;
  resolveAsset(relPath: string): Promise<ResolvedAsset>;
}
