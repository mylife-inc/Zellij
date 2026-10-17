'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from 'react';

import { overflowItem } from './nav-overflow';
import { useNavOverflow } from './use-nav-overflow';
import { AnnouncementBar } from './announcement-bar';
import { MobileDrawer } from './mobile-drawer';
import { ThemeToggle } from './theme-toggle';
import { ThemeSwitcher } from './theme-switcher';
import { SearchModal } from '../search/search-modal';
import {
  useDismissOnOutside,
  useFocusTrap,
  useHideOnScrollDown,
  useHoverCapable,
  useScrollSpy,
  useScrolledPast,
} from './hooks';
import type { BrandModel, NavItemModel, NavLeafModel, NavModel } from './nav-model';

/**
 * Logo and site name — design §4.1.
 *
 * Both colour variants are rendered and one is hidden by CSS rather than being
 * chosen in JavaScript: picking here would need the resolved colour scheme,
 * which is not known until after hydration, and the logo would visibly swap.
 */
function NavBrand({ brand }: { brand: BrandModel }) {
  const { logo, title } = brand;

  /*
   * Set as the variable the stylesheet already falls back on, rather than as a
   * height on the element: the rule that sizes the mark also caps its width and
   * swaps light for dark, and an inline height would win over all of it in one
   * of the two variants only.
   */
  const sizing = logo?.height ? ({ '--zellij-brand-height': logo.height } as CSSProperties) : undefined;

  const mark = logo ? (
    <>
      {/* eslint-disable-next-line @next/next/no-img-element -- next/image emits
          the src verbatim for unoptimized assets, which drops basePath. */}
      <img
        className="zellij-brand-image"
        data-variant="light"
        src={logo.src}
        alt={logo.alt}
        decoding="async"
      />
      {logo.dark ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img
          className="zellij-brand-image"
          data-variant="dark"
          src={logo.dark}
          alt=""
          aria-hidden="true"
          decoding="async"
        />
      ) : null}
    </>
  ) : (
    <span className="zellij-logo-text">{brand.siteName}</span>
  );

  return (
    <div
      className="zellij-brand"
      data-has-title={title ? 'true' : 'false'}
      data-has-dark={logo?.dark ? 'true' : 'false'}
      style={sizing}
    >
      {logo?.external ? (
        <a className="zellij-logo" href={logo.href} aria-label={logo.alt}>
          {mark}
        </a>
      ) : (
        <Link
          className="zellij-logo"
          href={logo?.href ?? '/'}
          aria-label={logo ? logo.alt : `${brand.siteName} home`}
        >
          {mark}
        </Link>
      )}

      {title ? (
        <Link className="zellij-brand-title" href={title.href}>
          {title.label}
        </Link>
      ) : null}
    </div>
  );
}

/** Reserved key for the theme picker in the bar's open-panel state. */
const THEME_MENU = '\u0000theme';
const OVERFLOW_MENU = '\u0000overflow';

export function NavBarClient({ model }: { model: NavModel }) {
  const pathname = usePathname() || '/';
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const menuBarRef = useRef<HTMLUListElement>(null);
  const headerRef = useRef<HTMLElement>(null);
  const navRef = useRef<HTMLElement>(null);
  const measureRef = useRef<HTMLUListElement>(null);
  const focusAfterResize = useRef<number | 'more' | null>(null);

  const isLanding = model.landingRoutes.includes(pathname);
  const transparentEligible = model.transparentOverHero && isLanding;

  const scrolled = useScrolledPast(model.scrollThreshold);
  const hidden = useHideOnScrollDown(model.hideOnScrollDown, model.scrollThreshold);
  const activeAnchor = useScrollSpy(isLanding);
  const hoverCapable = useHoverCapable();

  /*
   * Opening is immediate; closing is deferred by a beat. Pointers travel in
   * arcs, not straight lines, so a cursor heading from a trigger into its panel
   * can clip a neighbouring item or the panel's edge on the way. Closing
   * instantly on every pointer-leave makes the menu feel like it is dodging the
   * user. Any re-entry within the grace period cancels the close.
   */
  const closeTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const cancelClose = useCallback(() => {
    clearTimeout(closeTimer.current);
    closeTimer.current = undefined;
  }, []);

  const closeMenu = useCallback(() => {
    cancelClose();
    setOpenMenu(null);
  }, [cancelClose]);

  /*
   * Hover intent, for the "menu aim" problem.
   *
   * A full-width panel's links sit far from the trigger that opened them, so a
   * pointer travelling to one crosses neighbouring triggers along the bar. With
   * a naive hover-to-open, each of those hijacks the menu and the panel the
   * user was aiming at disappears from under them.
   *
   * Opening from rest is instant, because that is what makes a menu bar feel
   * responsive. Switching away from an already-open menu requires a brief
   * dwell — imperceptible when deliberate, but long enough that merely passing
   * through a trigger does not steal the panel.
   */
  const switchTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const pendingLabel = useRef<string | undefined>(undefined);

  const cancelSwitch = useCallback(() => {
    clearTimeout(switchTimer.current);
    switchTimer.current = undefined;
    pendingLabel.current = undefined;
  }, []);

  /*
   * Cancels a pending switch only if it is the one this item scheduled.
   *
   * Leaving one item and entering the next do not arrive in a guaranteed order —
   * the browser may deliver the leave *after* the enter. An unconditional cancel
   * then kills the switch the new item just scheduled, and the panel never
   * opens at all.
   */
  const cancelSwitchFor = useCallback((label: string) => {
    if (pendingLabel.current !== label) return;
    clearTimeout(switchTimer.current);
    switchTimer.current = undefined;
    pendingLabel.current = undefined;
  }, []);

  const openMenuNow = useCallback(
    (label: string) => {
      cancelClose();
      cancelSwitch();
      setOpenMenu(label);
    },
    [cancelClose, cancelSwitch],
  );

  const requestOpen = useCallback(
    (label: string) => {
      cancelClose();
      cancelSwitch();

      setOpenMenu((current) => {
        if (current === null || current === label) return label;
        /*
         * Another panel is open: only switch if the pointer settles here.
         *
         * The dwell is matched to the close grace rather than tuned short. A
         * pointer crossing this trigger on its way to the open panel can appear
         * stationary for longer than you would guess — a busy main thread
         * stretches the gap between move events — and every one of those
         * moments would otherwise steal the panel out from under the user.
         * Switching still feels immediate when it is what was meant.
         */
        pendingLabel.current = label;
        switchTimer.current = setTimeout(() => {
          pendingLabel.current = undefined;
          setOpenMenu(label);
        }, 260);
        return current;
      });
    },
    [cancelClose, cancelSwitch],
  );

  const closeMenuSoon = useCallback(() => {
    cancelClose();
    cancelSwitch();
    closeTimer.current = setTimeout(() => setOpenMenu(null), 260);
  }, [cancelClose, cancelSwitch]);

  useEffect(
    () => () => {
      cancelClose();
      cancelSwitch();
    },
    [cancelClose, cancelSwitch],
  );

  const onOverflowChange = useCallback((previous: number, next: number) => {
    const focused = document.activeElement;
    if (focused && menuBarRef.current?.contains(focused)) {
      const index = focused.closest<HTMLElement>('[data-nav-index]')?.dataset.navIndex;
      if (index === 'more' || (index !== undefined && Number(index) >= next)) {
        focusAfterResize.current = next < model.items.length ? 'more' : Math.min(previous, next - 1);
      }
    }
    closeMenu();
    cancelSwitch();
  }, [closeMenu, cancelSwitch, model.items.length]);

  const { count: visibleCount, ready: measured } = useNavOverflow(navRef, measureRef, model.items.length, onOverflowChange);
  const overflow = useMemo(() => overflowItem(model.items.slice(visibleCount)), [model.items, visibleCount]);

  useEffect(() => {
    const target = focusAfterResize.current;
    if (target === null) return;
    // Panel cleanup may restore the old trigger first; then move to a visible one.
    const frame = requestAnimationFrame(() => {
      menuBarRef.current?.querySelector<HTMLElement>(`[data-nav-index="${target}"] [data-zellij-navitem]`)?.focus();
      focusAfterResize.current = null;
    });
    return () => cancelAnimationFrame(frame);
  }, [visibleCount]);

  useEffect(() => {
    const media = window.matchMedia('(max-width: 900px)');
    let frame = 0;
    const changed = () => {
      const focused = document.activeElement;
      const restore = focused && (headerRef.current?.contains(focused) || focused.closest('.zellij-drawer-root'));
      closeMenu();
      cancelSwitch();
      setDrawerOpen(false);
      cancelAnimationFrame(frame);
      if (restore) frame = requestAnimationFrame(() => {
        const target = media.matches
          ? headerRef.current?.querySelector<HTMLElement>('.zellij-hamburger')
          : menuBarRef.current?.querySelector<HTMLElement>('[data-zellij-navitem]') ?? headerRef.current?.querySelector<HTMLElement>('.zellij-logo');
        target?.focus();
      });
    };
    media.addEventListener('change', changed);
    return () => {
      cancelAnimationFrame(frame);
      media.removeEventListener('change', changed);
    };
  }, [closeMenu, cancelSwitch]);

  /*
   * Dismissal is scoped to the header, not the menu bar list. The theme picker
   * sits in the actions area, outside that list — scoping to the list counted a
   * click on one of its options as "outside" and tore the panel down before the
   * click could land.
   */
  useDismissOnOutside(headerRef, openMenu !== null, closeMenu);

  /*
   * Escape closes whatever is open, wherever focus happens to be. The per-panel
   * focus traps only see the key once focus is inside them, which leaves a
   * hover-opened panel with no keyboard dismissal at all.
   */
  useEffect(() => {
    if (openMenu === null) return;

    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') closeMenu();
    };

    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [openMenu, closeMenu]);

  // Transparent only while over the hero; solid everywhere else and once scrolled.
  const solid = !transparentEligible || scrolled || openMenu !== null;

  const navHighlight = model.navHighlights[pathname];

  const isActive = useCallback(
    (href: string): boolean => {
      const [routePart, anchor] = splitHref(href);

      if (anchor && routePart === pathname) {
        return activeAnchor === anchor;
      }
      if (routePart === '' && anchor) {
        return activeAnchor === anchor;
      }
      if (routePart === '/') return pathname === '/';
      // Guide sub-pages keep the top-level "Docs" item active.
      return pathname === routePart || pathname.startsWith(`${routePart}/`);
    },
    [pathname, activeAnchor],
  );

  const isItemActive = useCallback(
    (item: NavItemModel): boolean => {
      if (navHighlight) return slug(item.label) === navHighlight || item.href === navHighlight;
      if (item.href) return isActive(item.href);
      const nested = item.children ?? item.mega?.groups.flatMap((g) => g.items) ?? [];
      return nested.some((leaf) => isActive(leaf.href));
    },
    [isActive, navHighlight],
  );

  /** Arrow-key movement between top-level items (spec §5.5 keyboard access). */
  const onMenuBarKeyDown = (event: React.KeyboardEvent<HTMLUListElement>): void => {
    const key = event.key;
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End', 'Escape'].includes(key)) return;

    if (key === 'Escape') {
      closeMenu();
      return;
    }

    const triggers = [
      ...(menuBarRef.current?.querySelectorAll<HTMLElement>('[data-zellij-navitem]') ?? []),
    ];
    if (triggers.length === 0) return;

    const index = triggers.indexOf(document.activeElement as HTMLElement);
    if (index === -1) return;

    event.preventDefault();
    const next =
      key === 'ArrowRight'
        ? (index + 1) % triggers.length
        : key === 'ArrowLeft'
          ? (index - 1 + triggers.length) % triggers.length
          : key === 'Home'
            ? 0
            : triggers.length - 1;

    triggers[next]?.focus();
  };

  const headerClasses = useMemo(
    () => ['zellij-header', model.sticky ? 'zellij-header-sticky' : ''].filter(Boolean).join(' '),
    [model.sticky],
  );

  return (
    <header
      ref={headerRef}
      className={headerClasses}
      data-solid={solid}
      data-hidden={hidden && !drawerOpen && openMenu === null}
      data-transparent-capable={transparentEligible}
      /*
       * The open/close region is the whole header — bar and panel together.
       * A mega panel spans the full width, so its links sit far from the
       * trigger that opened them; a pointer travelling to one crosses other
       * nav items on the way. Closing per item would drop the panel mid-
       * journey, which is exactly the dead-zone problem. Hovering a different
       * trigger still switches panels, because each one opens on its own enter.
       */
      {...(hoverCapable
        ? { onPointerEnter: cancelClose, onPointerLeave: closeMenuSoon }
        : {})}
    >
      {model.announcement ? <AnnouncementBar announcement={model.announcement} /> : null}

      <div className="zellij-nav-inner">
        <NavBrand brand={model.brand} />

        <nav ref={navRef} className="zellij-nav-desktop" aria-label="Main" data-align={model.align} data-measured={measured}>
          <ul className="zellij-nav-list" ref={menuBarRef} onKeyDown={onMenuBarKeyDown}>
            {model.items.slice(0, visibleCount).map((item, index) => (
              <NavItem
                key={index}
                itemKey={String(index)}
                item={item}
                active={isItemActive(item)}
                open={openMenu === String(index)}
                onOpen={() => openMenuNow(String(index))}
                onHover={() => requestOpen(String(index))}
                onHoverEnd={() => cancelSwitchFor(String(index))}
                onClose={closeMenu}
                onToggle={() =>
                  openMenu === String(index) ? closeMenu() : openMenuNow(String(index))
                }
                isActive={isActive}
              />
            ))}
            {visibleCount < model.items.length ? (
              <NavItem
                itemKey="more"
                item={overflow}
                active={model.items.slice(visibleCount).some(isItemActive)}
                open={openMenu === OVERFLOW_MENU}
                onOpen={() => openMenuNow(OVERFLOW_MENU)}
                onHover={() => requestOpen(OVERFLOW_MENU)}
                onHoverEnd={() => cancelSwitchFor(OVERFLOW_MENU)}
                onClose={closeMenu}
                onToggle={() => openMenu === OVERFLOW_MENU ? closeMenu() : openMenuNow(OVERFLOW_MENU)}
                isActive={isActive}
              />
            ) : null}
          </ul>
          <ul className="zellij-nav-list zellij-nav-measure" ref={measureRef} aria-hidden="true" inert>
            {[...model.items, { label: 'More' }].map((item, index) => (
              <li className="zellij-nav-probe" key={index}>
                <span className="zellij-nav-probe-label" data-active={index < model.items.length ? isItemActive(item) : true}>
                  {item.label}
                  {item.href ? (item.external ? <ExternalIcon /> : null) : <Chevron />}
                </span>
              </li>
            ))}
          </ul>
        </nav>

        <div className="zellij-nav-actions">
          {model.search ? (
            <button
              type="button"
              className="zellij-search-trigger"
              // Opens the ⌘K modal (SearchModal listens for this attribute).
              data-zellij-search-trigger
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                aria-hidden="true"
              >
                <circle cx="11" cy="11" r="7" />
                <path d="m20 20-3.5-3.5" />
              </svg>
              <span className="zellij-search-label">Search</span>
              {/* Decorative: the shortcut is not part of the button's name, and
                  including it would mismatch the accessible label. */}
              <kbd className="zellij-kbd" aria-hidden="true">
                ⌘K
              </kbd>
            </button>
          ) : null}

          {model.themeSwitcher ? (
            <ThemeSwitcher
              themes={model.themes}
              open={openMenu === THEME_MENU}
              onOpen={() => openMenuNow(THEME_MENU)}
              onToggle={() =>
                openMenu === THEME_MENU ? closeMenu() : openMenuNow(THEME_MENU)
              }
              onClose={closeMenu}
              {...(hoverCapable ? { onHover: () => requestOpen(THEME_MENU) } : {})}
            />
          ) : null}

          {model.themeToggle ? <ThemeToggle /> : null}

          {model.social?.github ? (
            <a
              className="zellij-icon-button"
              href={model.social.github}
              target="_blank"
              rel="noreferrer noopener"
              aria-label="GitHub repository"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M12 .5a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1.1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.8-1.6-2.7-.3-5.5-1.3-5.5-5.9 0-1.3.5-2.4 1.2-3.2 0-.4-.5-1.6.2-3.2 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.7 1.6.2 2.8.1 3.2.8.8 1.2 1.9 1.2 3.2 0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 .5Z" />
              </svg>
            </a>
          ) : null}

          {model.cta.map((action) => (
            <Link
              key={action.href}
              className="zellij-button zellij-nav-cta"
              data-style={action.style}
              href={action.href}
            >
              {action.label}
            </Link>
          ))}

          <button
            type="button"
            className="zellij-hamburger"
            aria-label="Open navigation"
            aria-expanded={drawerOpen}
            onClick={() => setDrawerOpen(true)}
          >
            <svg
              width="22"
              height="22"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              aria-hidden="true"
            >
              <path d="M3 6h18M3 12h18M3 18h18" />
            </svg>
          </button>
        </div>
      </div>

      <MobileDrawer
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        items={model.items}
        cta={model.cta}
        isActive={isActive}
      />

      {model.search ? <SearchModal basePath={model.basePath} /> : null}
    </header>
  );
}

interface NavItemProps {
  itemKey: string;
  item: NavItemModel;
  active: boolean;
  open: boolean;
  onOpen: () => void;
  /** Pointer-driven open, subject to hover intent. */
  onHover: () => void;
  /** Pointer left this item before the switch dwell elapsed. */
  onHoverEnd: () => void;
  onClose: () => void;
  onToggle: () => void;
  isActive: (href: string) => boolean;
}

function NavItem({
  itemKey,
  item,
  active,
  open,
  onOpen,
  onHover,
  onHoverEnd,
  onClose,
  onToggle,
  isActive,
}: NavItemProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const hoverCapable = useHoverCapable();
  // Set when the panel is opened from the keyboard, so focus follows it in —
  // but stays put when the panel opens from hover.
  const focusPanelOnOpen = useRef(false);
  useFocusTrap(panelRef, open, onClose);

  useLayoutEffect(() => {
    const panel = panelRef.current;
    if (!open || itemKey !== 'more' || item.mega || !panel) return;
    const position = () => {
      panel.style.right = '0px';
      const bounds = panel.getBoundingClientRect();
      const inset = 16;
      const offset = Math.min(0, bounds.left - inset)
        + Math.max(0, bounds.right - document.documentElement.clientWidth + inset);
      panel.style.right = `${offset}px`;
    };
    position();
    const observer = new ResizeObserver(position);
    observer.observe(panel);
    const list = panel.parentElement?.parentElement;
    if (list) observer.observe(list);
    window.addEventListener('resize', position);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', position);
    };
  }, [open, itemKey, item.mega]);

  useEffect(() => {
    if (!open || !focusPanelOnOpen.current) return;
    focusPanelOnOpen.current = false;
    panelRef.current?.querySelector<HTMLElement>('a')?.focus();
  }, [open]);

  if (item.href) {
    return (
      <li className="zellij-nav-item" data-nav-index={itemKey}>
        <MenuAnchor className="zellij-nav-link" target={item} active={active} navItem>
          {item.label}
        </MenuAnchor>
      </li>
    );
  }

  const panelId = `zellij-menu-${itemKey}`;
  const isMega = Boolean(item.mega);

  return (
    <li
      className="zellij-nav-item"
      data-nav-index={itemKey}
      data-mega={isMega}
      // Hover opens on desktop only; touch devices use tap-to-expand, and
      // keyboard uses ArrowDown/Escape (spec §5.5). Closing is the header's
      // job, so moving between items swaps panels instead of dismissing them.
      {...(hoverCapable ? { onPointerEnter: onHover, onPointerLeave: onHoverEnd } : {})}
    >
      <button
        type="button"
        className="zellij-nav-link zellij-nav-trigger"
        data-zellij-navitem
        data-active={active}
        aria-expanded={open}
        aria-controls={panelId}
        aria-haspopup="true"
        // On hover devices the menu is already open by the time a click lands,
        // so a toggle would close it on every click. Click opens (idempotent);
        // pointer-leave, Escape, or an outside click closes. Touch devices have
        // no hover, so there the click is a true toggle (spec §5.5).
        onClick={hoverCapable ? onOpen : onToggle}
        onKeyDown={(event) => {
          if (event.key === 'ArrowDown') {
            event.preventDefault();
            focusPanelOnOpen.current = true;
            onOpen();
          }
        }}
      >
        {item.label}
        <Chevron open={open} />
      </button>

      {open ? (
        <div
          className={isMega ? 'zellij-mega-panel' : 'zellij-dropdown'}
          id={panelId}
          ref={panelRef}
        >
          {isMega ? (
            <div className="zellij-mega-groups">
              {item.mega!.groups.map((group, index) => (
                <div className="zellij-mega-group" key={index}>
                  {group.heading ? <p className="zellij-mega-heading">{group.heading}</p> : null}
                  <ul>
                    {group.items.map((leaf, leafIndex) => (
                      <MegaLeaf key={`${leafIndex}:${leaf.href}`} leaf={leaf} active={isActive(leaf.href)} onNavigate={onClose} />
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          ) : (
            <ul>
              {item.children!.map((leaf, leafIndex) => (
                <li key={`${leafIndex}:${leaf.href}`}>
                  <MenuAnchor
                    className="zellij-dropdown-link"
                    target={leaf}
                    active={isActive(leaf.href)}
                    onNavigate={onClose}
                  >
                    {leaf.label}
                  </MenuAnchor>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </li>
  );
}

/**
 * One menu destination. A link that leaves the site is a plain anchor — routing
 * it through `next/link` would prefetch a foreign origin — and opens in a new
 * tab so the visitor keeps their place here (design §5).
 */
function MenuAnchor({
  className,
  target,
  active,
  onNavigate,
  navItem = false,
  children,
}: {
  className: string;
  target: NavLeafModel | NavItemModel;
  active: boolean;
  onNavigate?: () => void;
  /** Marks a top-level bar entry, which the keyboard and hover logic query for. */
  navItem?: boolean;
  children: ReactNode;
}) {
  const shared = {
    className,
    'data-active': active,
    ...(navItem ? { 'data-zellij-navitem': true } : {}),
    ...(onNavigate ? { onClick: onNavigate } : {}),
    ...(active ? { 'aria-current': 'page' as const } : {}),
  };

  if (target.external) {
    return (
      <a
        {...shared}
        href={target.href!}
        {...(target.newTab ? { target: '_blank', rel: 'noreferrer noopener' } : {})}
      >
        {children}
        <ExternalIcon />
      </a>
    );
  }

  return (
    <Link {...shared} href={target.href!}>
      {children}
    </Link>
  );
}

function MegaLeaf({
  leaf,
  active,
  onNavigate,
}: {
  leaf: NavLeafModel;
  active: boolean;
  onNavigate: () => void;
}) {
  return (
    <li>
      <MenuAnchor
        className="zellij-mega-link"
        target={leaf}
        active={active}
        onNavigate={onNavigate}
      >
        {leaf.icon ? <span className="zellij-mega-icon">{leaf.icon}</span> : null}
        <span className="zellij-mega-copy">
          <span className="zellij-mega-label">{leaf.label}</span>
          {leaf.description ? (
            <span className="zellij-mega-description">{leaf.description}</span>
          ) : null}
        </span>
      </MenuAnchor>
    </li>
  );
}

function splitHref(href: string): [string, string | undefined] {
  const index = href.indexOf('#');
  if (index === -1) return [href, undefined];
  return [href.slice(0, index), href.slice(index + 1) || undefined];
}

function slug(label: string): string {
  return label.toLowerCase().replace(/[^a-z0-9]+/g, '-');
}

function Chevron({ open = false }: { open?: boolean }) {
  return (
    <svg
      className="zellij-chevron"
      data-open={open}
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}

function ExternalIcon() {
  return (
    <svg
      className="zellij-external-icon"
      width="12"
      height="12"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.5"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M7 17 17 7M9 7h8v8" />
    </svg>
  );
}
