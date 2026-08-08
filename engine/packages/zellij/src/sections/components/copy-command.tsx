'use client';

import { useRef, useState } from 'react';

/**
 * A one-line command, with a button that copies it.
 *
 * The whole reason an install line appears on a landing page is that someone is
 * about to paste it into a terminal, and selecting a long URL by dragging is the
 * one step of that where they can go wrong — a missed leading character produces
 * an error message about something else entirely.
 *
 * The text is real text, not an image and not a `<pre>` full of spans, so it
 * survives copying with no clipboard support at all: select and ⌘C still work,
 * and the button is an accelerator rather than the only route.
 *
 * On failure the button stays idle. Clipboard access is refused over plain HTTP
 * and by permission policy, and a button that says "Copied" when nothing reached
 * the clipboard sends someone to a terminal to paste whatever was there before.
 */
export function CopyCommand({ command, label = 'terminal' }: { command: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const copy = async (): Promise<void> => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      clearTimeout(timer.current);
      timer.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // Left idle deliberately — see above.
    }
  };

  return (
    <div className="zellij-command">
      <code className="zellij-command-text">{command}</code>
      <button
        type="button"
        className="zellij-command-copy"
        onClick={copy}
        aria-label={copied ? `Copied the ${label} command` : `Copy the ${label} command`}
      >
        {copied ? (
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.25" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <path d="M20 6 9 17l-5-5" />
          </svg>
        ) : (
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <rect x="9" y="9" width="12" height="12" rx="2" />
            <path d="M5 15V5a2 2 0 0 1 2-2h10" />
          </svg>
        )}
        <span aria-hidden="true">{copied ? 'Copied' : 'Copy'}</span>
      </button>
    </div>
  );
}
