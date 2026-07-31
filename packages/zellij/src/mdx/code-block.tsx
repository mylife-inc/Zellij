'use client';

import { useRef, useState, type ReactNode } from 'react';

/**
 * Wraps Shiki's `<pre>` output with a filename label and copy button (spec §7.2).
 *
 * Highlighting itself happens at build time via rehype, so this client shell
 * only adds the interactive affordances — no highlighter ships to the browser.
 */
export function CodeBlock({
  children,
  ...props
}: {
  children?: ReactNode;
  'data-title'?: string;
  'data-lang'?: string;
  className?: string;
}) {
  const title = props['data-title'];
  const lang = props['data-lang'];
  const preRef = useRef<HTMLDivElement>(null);
  const [copied, setCopied] = useState(false);
  const resetTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const copy = async (): Promise<void> => {
    const text = preRef.current?.querySelector('code')?.textContent ?? '';
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      clearTimeout(resetTimer.current);
      resetTimer.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard blocked (insecure context or denied permission) — leave the
      // button idle rather than claiming a copy that did not happen.
    }
  };

  return (
    <div className="zellij-code" ref={preRef}>
      <div className="zellij-code-bar">
        {title ? (
          <span className="zellij-code-title">{title}</span>
        ) : (
          <span className="zellij-code-lang">{lang}</span>
        )}
        <button
          type="button"
          className="zellij-code-copy"
          onClick={copy}
          aria-label={copied ? 'Copied to clipboard' : 'Copy code to clipboard'}
        >
          {copied ? (
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <path d="M20 6 9 17l-5-5" />
            </svg>
          ) : (
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <rect x="9" y="9" width="11" height="11" rx="2" />
              <path d="M5 15V5a2 2 0 0 1 2-2h10" />
            </svg>
          )}
          <span className="zellij-code-copy-label">{copied ? 'Copied' : 'Copy'}</span>
        </button>
      </div>
      <pre className={props.className} tabIndex={0}>
        {children}
      </pre>
    </div>
  );
}
