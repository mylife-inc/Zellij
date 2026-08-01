'use client';

import { useEffect, useState } from 'react';
import type { TocEntry } from './toc';

/**
 * "On this page" rail. Highlights the heading currently in view so long pages
 * always show the reader where they are (spec §7).
 */
export function TableOfContents({ entries }: { entries: TocEntry[] }) {
  const [activeId, setActiveId] = useState<string | undefined>(entries[0]?.id);

  useEffect(() => {
    if (entries.length === 0) return;

    const headings = entries
      .map((entry) => document.getElementById(entry.id))
      .filter((el): el is HTMLElement => el !== null);
    if (headings.length === 0) return;

    const observer = new IntersectionObserver(
      (observed) => {
        const visible = observed
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);

        if (visible.length > 0) {
          setActiveId(visible[0]!.target.id);
          return;
        }

        // Nothing intersecting (between headings): keep the last one above the fold.
        const above = headings.filter((el) => el.getBoundingClientRect().top < 120);
        const last = above[above.length - 1];
        if (last) setActiveId(last.id);
      },
      { rootMargin: '-80px 0px -70% 0px', threshold: [0, 1] },
    );

    for (const heading of headings) observer.observe(heading);
    return () => observer.disconnect();
  }, [entries]);

  if (entries.length === 0) return null;

  return (
    <aside className="zellij-guide-toc" aria-label="On this page">
      <p className="zellij-guide-toc-heading">On this page</p>
      <ul>
        {entries.map((entry) => (
          <li key={entry.id} data-depth={entry.depth}>
            <a
              className="zellij-guide-toc-link"
              href={`#${entry.id}`}
              data-active={activeId === entry.id}
            >
              {entry.text}
            </a>
          </li>
        ))}
      </ul>
    </aside>
  );
}
