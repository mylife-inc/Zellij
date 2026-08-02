import type { MotionPreset, PageLayout, SiteConfig, ThemeName } from '../config/schema';
import type { AnySection } from '../sections/schemas';
import type { AuthoredMenuItem } from './model-schema';

/**
 * A page built by stacking named sections — design §2. Declared in `pages.yaml`,
 * or (legacy) by a file under `marketing/` with its sections written inline.
 */
export interface SectionsPage {
  kind: 'sections';
  /** The name `menu.yaml` and `pages.yaml` refer to it by. */
  name: string;
  /** Resolved site route, always leading-slash, never trailing (except "/"). */
  route: string;
  /** True when the author wrote `route:`, which then survives becoming home. */
  routeExplicit?: boolean;
  /** Source file, relative to the content root — used in error messages. */
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
  /** Resolved site route, e.g. "/getting-started/quickstart". */
  route: string;
  sourceFile: string;
  title: string;
  description?: string;
  /** Raw MDX source; compiled in the render layer, not the loader. */
  body: string;
  /** Route of the guide tree this page belongs to — how it finds its sidebar. */
  guide: string;
}

/**
 * A folder containing `_sidebar.yaml`, rendered as a documentation tree —
 * design §8. Any name, any number of them per site. A guide tree is a page in
 * the sense that matters here: something a menu item can point at.
 */
export interface GuideTree {
  kind: 'guide';
  /** The name `menu.yaml` refers to it by — its path in the content root. */
  name: string;
  /** Where the tree mounts, e.g. "/getting-started". */
  route: string;
  title: string;
  description?: string;
  /** From `order:` in `_sidebar.yaml`, or a numeric folder prefix (design §9). */
  order?: number;
  sourceDir: string;
  sidebar: SidebarNode[];
  pages: GuidePage[];
}

/** Anything a menu item can target. */
export type Destination = SectionsPage | GuideTree;

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

/* ------------------------------------------------------------ resolved menu */

/** A menu destination after `page:`/`url:` has been resolved to an href. */
export interface MenuLink {
  label: string;
  href: string;
  external: boolean;
  newTab: boolean;
  description?: string;
  /** Icon name; resolved to a component in the chrome layer. */
  icon?: string;
}

export interface MenuGroup {
  heading: string;
  items: MenuLink[];
}

/**
 * One entry in the top bar. A leaf carries `href`; a drawer carries `children`
 * (flat) or `groups` (headed columns, rendered as a full-width panel).
 */
export interface MenuItem {
  label: string;
  href?: string;
  external?: boolean;
  newTab?: boolean;
  description?: string;
  icon?: string;
  wide?: boolean;
  children?: MenuLink[];
  groups?: MenuGroup[];
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
 *
 * Two implementations ship: `FolderContentSource` for the current layout and
 * `LegacyContentSource` for `marketing/` + `zellij.config.yaml`. Both produce
 * this same model, so nothing downstream knows which layout it came from.
 */
/**
 * A section written to be read as well as rendered — a documentation example.
 *
 * Both halves come from one file under `examples/`: the source is shown
 * verbatim, and the parsed section is handed to the ordinary renderer. Nothing
 * transcribes one into the other, so a documented example cannot drift from
 * what it claims to produce, and a broken one fails the build like any other
 * content rather than shipping a wrong screenshot.
 */
export interface SectionExample {
  /** The name `<SectionPreview example="...">` refers to it by. */
  name: string;
  sourceFile: string;
  /** Verbatim file contents, for the source tab. */
  source: string;
  section: AnySection;
}

export interface ContentSource {
  loadSiteConfig(): Promise<SiteConfig>;
  /** `undefined` when no menu is declared — the caller then derives one (design §10). */
  loadMenu(): Promise<AuthoredMenuItem[] | undefined>;
  listPages(): Promise<SectionsPage[]>;
  listGuides(): Promise<GuideTree[]>;
  listExamples(): Promise<SectionExample[]>;
  resolveAsset(relPath: string): Promise<ResolvedAsset>;
}
