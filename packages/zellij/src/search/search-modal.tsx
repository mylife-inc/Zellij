'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';

import { useFocusTrap } from '../chrome/hooks';
import { SEARCH_INDEX_PATH } from '../content/paths';
import type { SearchEntry } from './build-index';

interface Loaded {
  entries: SearchEntry[];
  search: (query: string) => number[];
}

interface FlexIndexInstance {
  add: (id: number, text: string) => void;
  search: (query: string, options?: { limit?: number }) => number[];
}

type FlexIndexConstructor = new (options: { tokenize?: string }) => FlexIndexInstance;

/**
 * ⌘K search over guide content (spec §7.3).
 *
 * FlexSearch and the index are imported on first open, not at page load, so a
 * visitor who never searches downloads neither. Marketing pages are absent from
 * the index by construction.
 */
export function SearchModal({ basePath }: { basePath: string }) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [loaded, setLoaded] = useState<Loaded | undefined>(undefined);
  const [status, setStatus] = useState<'idle' | 'loading' | 'error'>('idle');
  const [active, setActive] = useState(0);

  const panelRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const router = useRouter();

  const close = useCallback(() => setOpen(false), []);
  useFocusTrap(panelRef, open, close);

  // ⌘K / Ctrl-K anywhere, and the nav's search button.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setOpen((current) => !current);
      }
    };

    const onTrigger = (event: Event): void => {
      const target = event.target as HTMLElement | null;
      if (target?.closest('[data-zellij-search-trigger]')) setOpen(true);
    };

    document.addEventListener('keydown', onKeyDown);
    document.addEventListener('click', onTrigger);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      document.removeEventListener('click', onTrigger);
    };
  }, []);

  // Fetch the index and the search library the first time the modal opens.
  //
  // `started` is a ref, not state, on purpose: setting state here would
  // re-run this effect, and its cleanup would cancel the very request it just
  // kicked off — leaving the panel stuck on "Loading…" forever.
  const started = useRef(false);

  useEffect(() => {
    if (!open || started.current) return;
    started.current = true;

    let cancelled = false;
    setStatus('loading');

    void (async () => {
      try {
        const [module, response] = await Promise.all([
          import('flexsearch'),
          fetch(`${basePath}${SEARCH_INDEX_PATH}`),
        ]);
        if (!response.ok) throw new Error(`index request failed: ${response.status}`);

        // FlexSearch ships both a named `Index` and a default namespace, and
        // which one survives depends on the bundler's ESM interop. Accept
        // either rather than assuming.
        const namespace = module as unknown as {
          Index?: FlexIndexConstructor;
          default?: { Index?: FlexIndexConstructor };
        };
        const FlexIndex = namespace.Index ?? namespace.default?.Index;
        if (!FlexIndex) throw new Error('flexsearch: no Index export');

        const entries = (await response.json()) as SearchEntry[];
        if (cancelled) return;

        const index = new FlexIndex({ tokenize: 'forward' });
        entries.forEach((entry, position) => {
          // Titles are repeated so heading matches outrank body matches.
          index.add(position, `${entry.title} ${entry.title} ${entry.page} ${entry.text}`);
        });

        setLoaded({
          entries,
          search: (value: string) => index.search(value, { limit: 8 }) as number[],
        });
        setStatus('idle');
      } catch {
        if (!cancelled) {
          // Allow a retry the next time the modal is opened.
          started.current = false;
          setStatus('error');
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [open, basePath]);

  useEffect(() => {
    if (open) inputRef.current?.focus();
    else {
      setQuery('');
      setActive(0);
    }
  }, [open]);

  const results = query.trim() && loaded ? loaded.search(query.trim()).map((i) => loaded.entries[i]!) : [];

  const go = useCallback(
    (entry: SearchEntry) => {
      setOpen(false);
      router.push(entry.route);
    },
    [router],
  );

  const onKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === 'Escape') {
      setOpen(false);
      return;
    }
    if (results.length === 0) return;

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActive((i) => (i + 1) % results.length);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((i) => (i - 1 + results.length) % results.length);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const entry = results[active];
      if (entry) go(entry);
    }
  };

  if (!open) return null;

  return (
    <div className="zellij-search-root" onKeyDown={onKeyDown}>
      <button
        type="button"
        className="zellij-search-backdrop"
        aria-label="Close search"
        tabIndex={-1}
        onClick={close}
      />
      <div
        className="zellij-search-panel"
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label="Search the guide"
      >
        <div className="zellij-search-field">
          <svg
            width="18"
            height="18"
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
          <input
            ref={inputRef}
            className="zellij-search-input"
            type="search"
            placeholder="Search the guide…"
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setActive(0);
            }}
            aria-label="Search the guide"
            aria-controls="zellij-search-results"
            autoComplete="off"
          />
          <kbd className="zellij-kbd">Esc</kbd>
        </div>

        <div className="zellij-search-results" id="zellij-search-results" role="listbox">
          {status === 'error' ? (
            <p className="zellij-search-empty">Search is unavailable right now.</p>
          ) : !loaded ? (
            // Keyed on `loaded`, not on `status`: the fetch starts in an effect
            // after the first paint, so a status-only check reports "no results"
            // for anything typed before the index arrives.
            <p className="zellij-search-empty">Loading…</p>
          ) : !query.trim() ? (
            <p className="zellij-search-empty">Start typing to search the guide.</p>
          ) : results.length === 0 ? (
            <p className="zellij-search-empty">No results for “{query.trim()}”.</p>
          ) : (
            <ul>
              {results.map((entry, index) => (
                <li key={entry.id}>
                  <button
                    type="button"
                    role="option"
                    aria-selected={index === active}
                    className="zellij-search-result"
                    data-active={index === active}
                    onMouseEnter={() => setActive(index)}
                    onClick={() => go(entry)}
                  >
                    <span className="zellij-search-result-title">{entry.title}</span>
                    <span className="zellij-search-result-page">{entry.page}</span>
                    {entry.text ? (
                      <span className="zellij-search-result-text">{entry.text.slice(0, 120)}</span>
                    ) : null}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}
