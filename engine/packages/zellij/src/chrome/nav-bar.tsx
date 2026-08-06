import type { Site } from '../content/site';
import { Icon } from '../icons';
import { NavBarClient } from './nav-bar-client';
import { THEMES } from '../themes/tokens';
import { THEME_NAMES } from '../config/schema';
import { assetUrl, isExternal, withBasePath } from '../content/paths';
import type { MenuLink } from '../content/types';
import type { BrandModel, NavItemModel, NavLeafModel, NavModel, ThemeOption } from './nav-model';

/**
 * Server half of the nav. Resolves icons and derives the route metadata the
 * client needs, so lucide and the content model never reach the browser bundle.
 */
export function NavBar({ site }: { site: Site }) {
  const { config } = site;

  const toLeaf = (leaf: MenuLink, context: string): NavLeafModel => ({
    label: leaf.label,
    href: leaf.href,
    ...(leaf.external ? { external: true } : {}),
    ...(leaf.newTab ? { newTab: true } : {}),
    ...(leaf.description ? { description: leaf.description } : {}),
    ...(leaf.icon ? { icon: <Icon name={leaf.icon} context={context} size={18} /> } : {}),
  });

  const items: NavItemModel[] = site.menu.map((item) => {
    if (item.groups) {
      return {
        label: item.label,
        mega: {
          groups: item.groups.map((group) => ({
            heading: group.heading,
            items: group.items.map((c) => toLeaf(c, `menu › ${item.label} › ${group.heading}`)),
          })),
        },
      };
    }
    if (item.children) {
      return {
        label: item.label,
        children: item.children.map((c) => toLeaf(c, `menu › ${item.label}`)),
      };
    }
    return {
      label: item.label,
      href: item.href!,
      ...(item.external ? { external: true } : {}),
      ...(item.newTab ? { newTab: true } : {}),
    };
  });

  const landingRoutes = site.pages.filter((p) => p.layout === 'landing').map((p) => p.route);

  const navHighlights: Record<string, string> = {};
  for (const page of site.pages) {
    if (page.navHighlight) navHighlights[page.route] = page.navHighlight;
  }

  const announcement = config.nav.announcement;

  const model: NavModel = {
    brand: buildBrand(site),
    sticky: config.nav.sticky,
    align: config.nav.align,
    hideOnScrollDown: config.nav.hideOnScrollDown,
    transparentOverHero: config.nav.transparentOverHero,
    scrollThreshold: config.nav.scrollThreshold,
    themeToggle: config.nav.themeToggle,
    themeSwitcher: config.nav.themeSwitcher,
    // Sent as data rather than imported client-side, so the theme token tables
    // stay out of the browser bundle.
    themes: config.nav.themeSwitcher
      ? THEME_NAMES.map<ThemeOption>((name) => ({
          name,
          label: THEMES[name].label,
          description: THEMES[name].description,
        }))
      : [],
    search: config.nav.search,
    items,
    cta: config.nav.cta,
    ...(config.nav.social ? { social: config.nav.social } : {}),
    ...(announcement
      ? {
          announcement: {
            // Dismissal is remembered per announcement text, so changing the
            // copy re-shows the bar to everyone.
            id: hashText(announcement.text),
            text: announcement.text,
            ...(announcement.link ? { link: announcement.link } : {}),
            dismissible: announcement.dismissible,
          },
        }
      : {}),
    landingRoutes,
    navHighlights,
    basePath: config.basePath,
  };

  return <NavBarClient model={model} />;
}

/**
 * The brand block — design §4.1.
 *
 * Asset URLs are resolved here rather than in the client half because the
 * client has no access to the content root. They go through `withBasePath`
 * because these are plain `<img>` elements: `next/image` applies the prefix
 * itself, but nothing else does.
 */
function buildBrand(site: Site): BrandModel {
  const { name, brand, basePath } = site.config;
  const logo = brand.logo;

  if (!logo) {
    return { siteName: name, ...(brand.title ? { title: { label: brand.title, href: '/' } } : {}) };
  }

  const href = logo.href ?? '/';
  const external = isExternal(href);

  return {
    siteName: name,
    logo: {
      src: withBasePath(assetUrl(logo.src), basePath),
      ...(logo.dark ? { dark: withBasePath(assetUrl(logo.dark), basePath) } : {}),
      alt: logo.alt ?? name,
      href,
      external,
      ...(logo.height ? { height: logo.height } : {}),
    },
    // A logo that leaves the site would otherwise take the only way home with
    // it, so the site's own name becomes that link.
    ...(brand.title ? { title: { label: brand.title, href: '/' } } : {}),
  };
}

/** Small stable hash — only needs to change when the announcement copy does. */
function hashText(text: string): string {
  let hash = 0;
  for (let i = 0; i < text.length; i += 1) {
    hash = (hash << 5) - hash + text.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash).toString(36);
}
