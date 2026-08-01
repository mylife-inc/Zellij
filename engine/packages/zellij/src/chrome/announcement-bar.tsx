'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';
import type { AnnouncementModel } from './nav-model';

const STORAGE_PREFIX = 'zellij:announcement:';

/**
 * Optional bar above the nav. Dismissal is remembered in localStorage and keyed
 * by the announcement copy, so new announcements re-appear (spec §5.5).
 */
export function AnnouncementBar({ announcement }: { announcement: AnnouncementModel }) {
  const [dismissed, setDismissed] = useState(false);
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    if (!announcement.dismissible) {
      setHydrated(true);
      return;
    }
    try {
      setDismissed(localStorage.getItem(`${STORAGE_PREFIX}${announcement.id}`) === '1');
    } catch {
      // Storage unavailable — show the bar.
    }
    setHydrated(true);
  }, [announcement.id, announcement.dismissible]);

  const dismiss = (): void => {
    setDismissed(true);
    try {
      localStorage.setItem(`${STORAGE_PREFIX}${announcement.id}`, '1');
    } catch {
      // Non-fatal: the bar stays dismissed for this page view.
    }
  };

  if (dismissed) return null;

  return (
    <div
      className="zellij-announcement"
      // Avoid a flash of the bar before we know it was dismissed.
      data-zellij-hydrated={hydrated ? 'true' : 'false'}
    >
      <p className="zellij-announcement-text">
        {announcement.text}{' '}
        {announcement.link ? (
          <Link className="zellij-announcement-link" href={announcement.link.href}>
            {announcement.link.label}
          </Link>
        ) : null}
      </p>
      {announcement.dismissible ? (
        <button
          type="button"
          className="zellij-announcement-dismiss"
          onClick={dismiss}
          aria-label="Dismiss announcement"
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
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      ) : null}
    </div>
  );
}
