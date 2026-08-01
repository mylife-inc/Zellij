'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';

import type { SidebarNode } from '../content/types';
import { findAncestors } from './navigation';

/**
 * Collapsed state is keyed per tree. A site can have many guide folders, and two
 * of them may well both have a group called "Overview" — sharing one key would
 * collapse both together.
 */
function storageKey(label: string): string {
  return `zellij:guide-nav:${label}`;
}

/** Stable identity for a group: its label path, so siblings never collide. */
function groupId(trail: readonly string[]): string {
  return trail.join(' / ');
}

export function GuideSidebar({
  sidebar,
  label = 'Guide',
}: {
  sidebar: SidebarNode[];
  /** The tree's own title — a site may have several, and "Guide" names none. */
  label?: string;
}) {
  const pathname = usePathname() || '';

  /**
   * Collapsed groups are stored, not expanded ones. A group added to the docs
   * later is then open by default rather than hidden because it was missing
   * from a saved list.
   */
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(() => new Set());

  useEffect(() => {
    try {
      const raw = localStorage.getItem(storageKey(label));
      setCollapsed(raw ? new Set(JSON.parse(raw) as string[]) : new Set());
    } catch {
      // Unreadable or absent — everything stays open.
    }
  }, [label]);

  const toggle = useCallback(
    (id: string) => {
      setCollapsed((current) => {
        const next = new Set(current);
        if (next.has(id)) next.delete(id);
        else next.add(id);

        try {
          localStorage.setItem(storageKey(label), JSON.stringify([...next]));
        } catch {
          // Storage unavailable — the choice still holds for this page view.
        }
        return next;
      });
    },
    [label],
  );

  /**
   * Groups on the path to the current page are always open. Otherwise a reader
   * who collapsed a group and then followed a link into it would land on a page
   * that is invisible in the navigation.
   */
  const openByRoute = useMemo(() => {
    const ancestors = findAncestors(sidebar, pathname);
    const ids = new Set<string>();
    const trail: string[] = [];
    for (const group of ancestors) {
      trail.push(group.label);
      ids.add(groupId(trail));
    }
    return ids;
  }, [sidebar, pathname]);

  const renderNodes = (nodes: readonly SidebarNode[], trail: string[], depth: number) => (
    <ul className="zellij-guide-sidebar-list" data-depth={depth}>
      {nodes.map((node) => {
        if (node.kind === 'link') {
          const active = pathname === node.route;
          return (
            <li key={node.route}>
              <Link
                className="zellij-guide-sidebar-link"
                href={node.route}
                data-active={active}
                {...(active ? { 'aria-current': 'page' as const } : {})}
              >
                {node.label}
              </Link>
            </li>
          );
        }

        const nextTrail = [...trail, node.label];
        const id = groupId(nextTrail);
        const open = openByRoute.has(id) || !collapsed.has(id);
        const panelId = `zellij-nav-${id.replace(/[^a-z0-9]+/gi, '-').toLowerCase()}`;

        return (
          <li key={id} className="zellij-guide-sidebar-group" data-depth={depth}>
            <button
              type="button"
              className="zellij-guide-sidebar-toggle"
              aria-expanded={open}
              aria-controls={panelId}
              onClick={() => toggle(id)}
            >
              <svg
                className="zellij-guide-sidebar-chevron"
                data-open={open}
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <path d="m9 18 6-6-6-6" />
              </svg>
              <span>{node.label}</span>
            </button>

            <div id={panelId} hidden={!open}>
              {renderNodes(node.children, nextTrail, depth + 1)}
            </div>
          </li>
        );
      })}
    </ul>
  );

  return (
    <nav className="zellij-guide-sidebar-nav" aria-label={label}>
      {renderNodes(sidebar, [], 0)}
    </nav>
  );
}
