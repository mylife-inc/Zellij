import type { ReactNode } from 'react';
import type { Cta, NavConfig, SocialConfig } from '../config/schema';

export interface NavLeafModel {
  label: string;
  href: string;
  external?: boolean;
  newTab?: boolean;
  description?: string;
  icon?: ReactNode;
}

export interface NavItemModel {
  label: string;
  href?: string;
  external?: boolean;
  newTab?: boolean;
  children?: NavLeafModel[];
  mega?: { groups: Array<{ heading: string; items: NavLeafModel[] }> };
}

/**
 * The left of the bar — design §4.1. `logo.href` pointing away from this site
 * is how a family of sites shares an owner: the mark returns to the hub, and
 * `title` becomes the way home within this site.
 */
export interface BrandModel {
  /** Fallback wordmark and the accessible name when no logo image is set. */
  siteName: string;
  logo?: { src: string; dark?: string; alt: string; href: string; external: boolean };
  title?: { label: string; href: string };
}

export interface AnnouncementModel {
  /** Stable key for remembering dismissal in localStorage (spec §5.5). */
  id: string;
  text: string;
  link?: { label: string; href: string };
  dismissible: boolean;
}

export interface ThemeOption {
  name: string;
  label: string;
  description: string;
}

export interface NavModel {
  brand: BrandModel;
  sticky: boolean;
  align: NavConfig['align'];
  hideOnScrollDown: boolean;
  transparentOverHero: boolean;
  scrollThreshold: number;
  themeToggle: boolean;
  themeSwitcher: boolean;
  /** Populated only when the switcher is enabled. */
  themes: ThemeOption[];
  search: boolean;
  items: NavItemModel[];
  cta: Cta[];
  social?: SocialConfig;
  announcement?: AnnouncementModel;
  /** Routes using `layout: landing`, where the bar starts transparent (spec §5.5). */
  landingRoutes: string[];
  /** route → navHighlight, for pages that mark a nav item active explicitly. */
  navHighlights: Record<string, string>;
  basePath: string;
}
