'use client';

import { useEffect } from 'react';

/** Serial, bounded polling also works through a container's published port. */
export function watchDevelopmentRevision(url: string, revision: string, reload: () => void): () => void {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let timeout: ReturnType<typeof setTimeout> | undefined;
  let request: AbortController | undefined;
  const poll = async () => {
    request = new AbortController();
    timeout = setTimeout(() => request?.abort(), 5000);
    try {
      const response = await fetch(url, { cache: 'no-store', signal: request.signal });
      if (response.ok) {
        const value: unknown = await response.json();
        if (!stopped && value && typeof value === 'object' && 'version' in value && value.version === 1
          && 'revision' in value && typeof value.revision === 'string' && value.revision
          && value.revision !== revision) {
          stopped = true;
          reload();
        }
      }
    } catch {
      // An unavailable watcher/server is temporary; leave the current page up.
    } finally {
      clearTimeout(timeout);
      if (!stopped) timer = setTimeout(() => { void poll(); }, 1000);
    }
  };
  void poll();
  return () => { stopped = true; clearTimeout(timer); clearTimeout(timeout); request?.abort(); };
}

/** Mounted only in development when prepared revision metadata is available. */
export function DevelopmentReload({ url, revision }: { url: string; revision: string }) {
  useEffect(() => watchDevelopmentRevision(url, revision, () => window.location.reload()), [url, revision]);
  return null;
}
