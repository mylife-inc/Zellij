'use client';

import Link from 'next/link';
import { useCallback, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useBodyScrollLock, useFocusTrap } from './hooks';
import type { NavItemModel, NavLeafModel } from './nav-model';

interface MobileDrawerProps {
  open: boolean;
  onClose: () => void;
  items: NavItemModel[];
  cta: Array<{ label: string; href: string; style: string }>;
  isActive: (href: string) => boolean;
}

/**
 * Full-height mobile navigation. Nested items become accordions and the CTA is
 * pinned to the bottom; Escape and backdrop tap close it (spec §5.5).
 */
export function MobileDrawer({ open, onClose, items, cta, isActive }: MobileDrawerProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState<string | null>(null);

  useBodyScrollLock(open);
  useFocusTrap(panelRef, open, onClose);

  const toggle = useCallback(
    (label: string) => setExpanded((current) => (current === label ? null : label)),
    [],
  );

  if (!open) return null;

  const renderLeaf = (leaf: NavLeafModel) => (
    <li key={leaf.href}>
      <Link
        className="zellij-drawer-sublink"
        href={leaf.href}
        onClick={onClose}
        {...(isActive(leaf.href) ? { 'aria-current': 'page' as const } : {})}
      >
        {leaf.icon ? <span className="zellij-drawer-icon">{leaf.icon}</span> : null}
        <span>
          <span className="zellij-drawer-sublabel">{leaf.label}</span>
          {leaf.description ? (
            <span className="zellij-drawer-subdescription">{leaf.description}</span>
          ) : null}
        </span>
      </Link>
    </li>
  );

  /*
   * Rendered into `document.body` rather than where it sits in the tree.
   *
   * The drawer is a child of the header, and the header carries a
   * `backdrop-filter` for its translucent blur. An element with a
   * `backdrop-filter` becomes the containing block for its fixed-position
   * descendants — so `position: fixed` on the root resolved against the
   * header's 48px-tall box instead of the viewport. Opening the drawer part-way
   * down a page gave a 48px sliver at the top of the screen with the CTA
   * stranded under it, and nothing else: the panel was laid out against a box
   * the height of the bar it came from.
   *
   * A portal is the fix rather than moving the blur, because the blur is not
   * the problem — being inside anything is. `transform`, `filter`,
   * `perspective`, `will-change` and `contain: paint` all capture fixed
   * descendants the same way, and a header is exactly where such a property
   * gets added later. Attached to the body, the panel has no ancestor left to
   * be captured by, which is what "fixed to the viewport" was always meant to
   * say.
   */
  return createPortal(
    <div className="zellij-drawer-root">
      <button
        type="button"
        className="zellij-drawer-backdrop"
        aria-label="Close navigation"
        tabIndex={-1}
        onClick={onClose}
      />
      {/* `tabIndex={-1}` so the trap can move focus here on open: not reachable
          by Tab, but focusable programmatically. */}
      <div
        className="zellij-drawer-panel"
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label="Site navigation"
        tabIndex={-1}
      >
        <nav className="zellij-drawer-nav" aria-label="Mobile">
          <ul className="zellij-drawer-list">
            {items.map((item) => {
              const nested = item.children ?? item.mega?.groups.flatMap((g) => g.items);

              if (!nested) {
                return (
                  <li key={item.label}>
                    <Link
                      className="zellij-drawer-link"
                      href={item.href!}
                      onClick={onClose}
                      {...(isActive(item.href!) ? { 'aria-current': 'page' as const } : {})}
                    >
                      {item.label}
                    </Link>
                  </li>
                );
              }

              const isOpen = expanded === item.label;
              const panelId = `zellij-drawer-panel-${slug(item.label)}`;

              return (
                <li key={item.label}>
                  <button
                    type="button"
                    className="zellij-drawer-link zellij-drawer-toggle"
                    onClick={() => toggle(item.label)}
                    aria-expanded={isOpen}
                    aria-controls={panelId}
                  >
                    {item.label}
                    <svg
                      className="zellij-chevron"
                      data-open={isOpen}
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      aria-hidden="true"
                    >
                      <path d="m6 9 6 6 6-6" />
                    </svg>
                  </button>
                  {isOpen ? (
                    <ul className="zellij-drawer-sublist" id={panelId}>
                      {item.mega
                        ? item.mega.groups.map((group) => (
                            <li key={group.heading}>
                              <p className="zellij-drawer-group-heading">{group.heading}</p>
                              <ul className="zellij-drawer-sublist">
                                {group.items.map(renderLeaf)}
                              </ul>
                            </li>
                          ))
                        : nested.map(renderLeaf)}
                    </ul>
                  ) : null}
                </li>
              );
            })}
          </ul>
        </nav>

        {cta.length > 0 ? (
          <div className="zellij-drawer-cta">
            {cta.map((action) => (
              <Link
                key={action.href}
                className="zellij-button"
                data-style={action.style}
                href={action.href}
                onClick={onClose}
              >
                {action.label}
              </Link>
            ))}
          </div>
        ) : null}
      </div>
    </div>,
    document.body,
  );
}

function slug(label: string): string {
  return label.toLowerCase().replace(/[^a-z0-9]+/g, '-');
}
