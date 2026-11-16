'use client';

import Link from 'next/link';
import { useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useBodyScrollLock, useFocusTrap } from './hooks';
import type { NavItemModel, NavLeafModel } from './nav-model';

interface MobileDrawerProps {
  open: boolean;
  onClose: () => void;
  items: NavItemModel[];
  cta: Array<{ label: string; href: string; style: string }>;
  github?: string;
  isActive: (href: string) => boolean;
}

/** Each opening starts at the main menu; closing unmounts its navigation state. */
export function MobileDrawer(props: MobileDrawerProps) {
  return props.open ? <OpenMobileDrawer {...props} /> : null;
}

function OpenMobileDrawer({ onClose, items, cta, github, isActive }: MobileDrawerProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const navRef = useRef<HTMLElement>(null);
  const titleRef = useRef<HTMLHeadingElement>(null);
  const rootScroll = useRef(0);
  const pendingFocus = useRef<'heading' | number | null>(null);
  const navId = useId();
  const [view, setView] = useState<{ index: number | null; direction: 'forward' | 'back' }>({ index: null, direction: 'forward' });
  const section = view.index === null ? undefined : items[view.index];

  useBodyScrollLock(true);
  useFocusTrap(panelRef, true, onClose);

  useLayoutEffect(() => {
    const target = pendingFocus.current;
    if (target === null) return;
    pendingFocus.current = null;
    if (navRef.current) navRef.current.scrollTop = target === 'heading' ? 0 : rootScroll.current;
    if (target === 'heading') titleRef.current?.focus({ preventScroll: true });
    else {
      const trigger = navRef.current?.querySelector<HTMLElement>(`[data-zellij-drawer-item="${target}"]`);
      (trigger ?? panelRef.current)?.focus({ preventScroll: true });
    }
  }, [view.index]);

  const enter = (index: number) => {
    rootScroll.current = navRef.current?.scrollTop ?? 0;
    pendingFocus.current = 'heading';
    setView({ index, direction: 'forward' });
  };
  const back = () => {
    pendingFocus.current = view.index;
    setView({ index: null, direction: 'back' });
  };

  const renderLeaf = (leaf: NavLeafModel, index: number) => (
    <li key={index}>
      <Link
        className="zellij-drawer-sublink"
        href={leaf.href}
        onClick={onClose}
        {...(leaf.newTab ? { target: '_blank', rel: 'noopener noreferrer' } : {})}
        {...(isActive(leaf.href) ? { 'aria-current': 'page' as const } : {})}
      >
        {leaf.icon ? <span className="zellij-drawer-icon">{leaf.icon}</span> : null}
        <span>
          <span className="zellij-drawer-sublabel">{leaf.label}</span>
          {leaf.description ? <span className="zellij-drawer-subdescription">{leaf.description}</span> : null}
        </span>
        {leaf.external ? <DrawerIcon kind="external" /> : null}
      </Link>
    </li>
  );

  // Portal outside the sticky header: its backdrop-filter would otherwise
  // constrain fixed descendants and leave a scrolled drawer only header-high.
  return createPortal(
    <div className="zellij-drawer-root">
      <button type="button" className="zellij-drawer-backdrop" aria-hidden="true" tabIndex={-1} onClick={onClose} />
      <div className="zellij-drawer-panel" ref={panelRef} role="dialog" aria-modal="true" aria-label="Site navigation" tabIndex={-1}>
        <div className="zellij-drawer-toolbar">
          {section ? (
            <button type="button" className="zellij-drawer-control" aria-label="Back to main menu" onClick={back}>
              <DrawerIcon kind="back" />
            </button>
          ) : <span aria-hidden="true" />}
          <h2 className="zellij-drawer-title" ref={titleRef} tabIndex={-1}>{section?.label ?? 'Menu'}</h2>
          <button type="button" className="zellij-drawer-control" aria-label="Close navigation" onClick={onClose}>
            <DrawerIcon kind="close" />
          </button>
        </div>
        <nav className="zellij-drawer-nav" ref={navRef} id={navId} aria-label={section ? `${section.label} navigation` : 'Mobile'}>
          <div key={view.index ?? 'root'} className="zellij-drawer-view" data-direction={view.direction} data-level={section ? 'section' : 'root'}>
            {section ? (
              <ul className="zellij-drawer-sublist">
                {section.mega ? section.mega.groups.map((group, index) => (
                  <li className="zellij-drawer-group" key={index}>
                    {group.heading ? <h3 className="zellij-drawer-group-heading">{group.heading}</h3> : null}
                    <ul className="zellij-drawer-sublist">{group.items.map(renderLeaf)}</ul>
                  </li>
                )) : section.children?.map(renderLeaf)}
              </ul>
            ) : (
              <ul className="zellij-drawer-list">
                {items.map((item, index) => {
                  const nested = item.children ?? item.mega?.groups.flatMap(group => group.items);
                  return (
                    <li key={index}>
                      {nested ? (
                        <button type="button" className="zellij-drawer-link zellij-drawer-toggle"
                          data-zellij-drawer-item={index} data-active={nested.some(leaf => isActive(leaf.href))}
                          aria-controls={navId} onClick={() => enter(index)}>
                          <span>{item.label}</span><DrawerIcon kind="forward" />
                        </button>
                      ) : (
                        <Link className="zellij-drawer-link" href={item.href!} onClick={onClose}
                          {...(item.newTab ? { target: '_blank', rel: 'noopener noreferrer' } : {})}
                          {...(isActive(item.href!) ? { 'aria-current': 'page' as const } : {})}>
                          <span>{item.label}</span>{item.external ? <DrawerIcon kind="external" /> : null}
                        </Link>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}
          </div>
        </nav>
        {!section && (cta.length > 0 || github) ? (
          <div className="zellij-drawer-cta">
            {github ? <a className="zellij-drawer-link" href={github} target="_blank" rel="noopener noreferrer" onClick={onClose}>GitHub repository</a> : null}
            {cta.map(action => (
              <Link key={action.href} className="zellij-button" data-style={action.style} href={action.href} onClick={onClose}>{action.label}</Link>
            ))}
          </div>
        ) : null}
      </div>
    </div>,
    document.body,
  );
}

function DrawerIcon({ kind }: { kind: 'back' | 'forward' | 'close' | 'external' }) {
  const path = { back: 'm14 6-6 6 6 6', forward: 'm9 6 6 6-6 6', close: 'm6 6 12 12M6 18 18 6', external: 'M7 17 17 7M7 7h10v10' }[kind];
  return <svg className="zellij-drawer-glyph" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={path} /></svg>;
}
