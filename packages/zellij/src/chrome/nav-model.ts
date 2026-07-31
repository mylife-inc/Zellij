import type { ReactNode } from 'react';
import type { Cta, NavConfig, SocialConfig } from '../config/schema';

export interface NavLeafModel {
  label: string;
  href: string;
  description?: string;
  icon?: ReactNode;
}

export interface NavItemModel {
  label: string;
  href?: string;
  children?: NavLeafModel[];
  mega?: { groups: Array<{ heading: string; items: NavLeafModel[] }> };
}

export interface AnnouncementModel {
  /** Stable key for remembering dismissal in localStorage (spec §5.5). */
  id: string;
  text: string;
  link?: { label: string; href: string };
  dismissible: boolean;
}

export interface NavModel {
  productName: string;
  logo?: string;
  logoDark?: string;
  sticky: boolean;
  align: NavConfig['align'];
  hideOnScrollDown: boolean;
  transparentOverHero: boolean;
  scrollThreshold: number;
  themeToggle: boolean;
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
