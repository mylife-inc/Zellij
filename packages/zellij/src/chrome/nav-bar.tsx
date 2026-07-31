import type { Site } from '../content/site';
import { Icon } from '../icons';
import { NavBarClient } from './nav-bar-client';
import { THEMES } from '../themes/tokens';
import { THEME_NAMES } from '../config/schema';
import type { NavItemModel, NavLeafModel, NavModel, ThemeOption } from './nav-model';

/**
 * Server half of the nav. Resolves icons and derives the route metadata the
 * client needs, so lucide and the content model never reach the browser bundle.
 */
export function NavBar({ site }: { site: Site }) {
  const { config } = site;

  const toLeaf = (
    leaf: { label: string; href: string; description?: string; icon?: string },
    context: string,
  ): NavLeafModel => ({
    label: leaf.label,
    href: leaf.href,
    ...(leaf.description ? { description: leaf.description } : {}),
    ...(leaf.icon ? { icon: <Icon name={leaf.icon} context={context} size={18} /> } : {}),
  });

  const items: NavItemModel[] = config.nav.links.map((item) => {
    if (item.children) {
      return {
        label: item.label,
        children: item.children.map((c) => toLeaf(c, `nav.links › ${item.label}`)),
      };
    }
    if (item.mega) {
      return {
        label: item.label,
        mega: {
          groups: item.mega.groups.map((group) => ({
            heading: group.heading,
            items: group.items.map((c) => toLeaf(c, `nav.links › ${item.label} › ${group.heading}`)),
          })),
        },
      };
    }
    return { label: item.label, href: item.href! };
  });

  const landingRoutes = site.marketingPages
    .filter((p) => p.layout === 'landing')
    .map((p) => p.route);

  const navHighlights: Record<string, string> = {};
  for (const page of site.marketingPages) {
    if (page.navHighlight) navHighlights[page.route] = page.navHighlight;
  }

  const announcement = config.nav.announcement;

  const model: NavModel = {
    productName: config.product.name,
    ...(config.product.logo ? { logo: config.product.logo } : {}),
    ...(config.product.logoDark ? { logoDark: config.product.logoDark } : {}),
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

/** Small stable hash — only needs to change when the announcement copy does. */
function hashText(text: string): string {
  let hash = 0;
  for (let i = 0; i < text.length; i += 1) {
    hash = (hash << 5) - hash + text.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash).toString(36);
}
