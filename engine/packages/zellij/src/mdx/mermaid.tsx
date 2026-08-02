'use client';

import { useEffect, useId, useRef, useState } from 'react';

type Status = 'pending' | 'drawn' | 'failed';

/** The site's mode attribute carries `system`, which only the browser can resolve. */
function prefersDark(): boolean {
  const declared = document.documentElement.getAttribute('data-zellij-mode');
  if (declared === 'dark') return true;
  if (declared === 'light') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

/**
 * Dresses Mermaid in the site's own tokens.
 *
 * Left to itself Mermaid paints in its stock lilac-and-lime palette, which is
 * legible but belongs to no theme in particular — a diagram that plainly came
 * from somewhere else. Reading the live custom properties keeps a diagram in
 * step with whatever theme is showing, including one switched at runtime.
 */
function themeVariables(): Record<string, string> {
  const styles = getComputedStyle(document.documentElement);
  const token = (name: string, fallback: string) =>
    styles.getPropertyValue(`--zellij-${name}`).trim() || fallback;

  const fg = token('fg', '#14161a');
  const bg = token('bg', '#ffffff');
  const surface = token('surface', '#f7f8fa');
  const border = token('border', '#e4e7ec');
  const accent = token('accent', '#E86A33');

  return {
    background: bg,
    mainBkg: surface,
    primaryColor: surface,
    primaryTextColor: fg,
    primaryBorderColor: border,
    secondaryColor: bg,
    secondaryTextColor: fg,
    secondaryBorderColor: border,
    tertiaryColor: surface,
    tertiaryTextColor: fg,
    tertiaryBorderColor: border,
    lineColor: token('muted', '#5f6672'),
    textColor: fg,
    nodeBorder: border,
    clusterBkg: bg,
    clusterBorder: border,
    // The one place the accent earns its keep: the arrows and highlights that
    // carry the diagram's meaning, rather than every box on it.
    edgeLabelBackground: bg,
    actorBkg: surface,
    actorBorder: accent,
    signalColor: fg,
    signalTextColor: fg,

    /*
     * Mindmaps and pie charts colour their branches from a numbered scale
     * rather than from `primaryColor`, so without these they keep Mermaid's own
     * palette while every other diagram type follows the theme — which looked
     * exactly like the bug it was. One fill for every branch: a mindmap's
     * structure is already carried by its shape, so colour has nothing left to
     * say and only competes with the page.
     *
     * The border token rather than the surface one because a mindmap paints its
     * connectors in the branch colour too — surface-on-surface drew the lines
     * in the page's own background and lost them. Mermaid styles those lines
     * through an id selector, so CSS cannot take them back afterwards.
     */
    ...Object.fromEntries(
      Array.from({ length: 12 }, (_, i) => [
        [`cScale${i}`, border],
        [`cScaleLabel${i}`, fg],
        [`cScaleInv${i}`, fg],
      ]).flat(),
    ),
  };
}

/**
 * Draws a Mermaid diagram — flowcharts, sequences, state machines, mindmaps,
 * Gantt charts, everything the grammar covers.
 *
 * Mermaid measures text to lay a diagram out, so it needs a real DOM and cannot
 * run during the build the way the maths does. It is imported dynamically
 * instead: the library is a few hundred kilobytes, and a page with no diagram
 * on it must not pay for one.
 *
 * The source stays in the markup as a `<pre>` until the drawing replaces it, so
 * a reader without JavaScript — or with a diagram that fails to parse — still
 * gets the definition rather than an empty space.
 */
export function Mermaid({ code }: { code?: string }) {
  const host = useRef<HTMLDivElement>(null);
  const [status, setStatus] = useState<Status>('pending');
  const [error, setError] = useState<string>('');
  const [dark, setDark] = useState<boolean | null>(null);

  // Mermaid needs a DOM id it can key its own definitions off; `useId` gives a
  // stable one, but its colons are not valid in a CSS selector.
  const id = `zellij-mermaid-${useId().replace(/:/g, '')}`;

  // A diagram is drawn with baked-in colours, so unlike everything else on the
  // page it cannot follow a theme change on its own — it has to be redrawn.
  useEffect(() => {
    const sync = () => setDark(prefersDark());
    sync();

    const observer = new MutationObserver(sync);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-zellij-mode'],
    });

    const query = window.matchMedia('(prefers-color-scheme: dark)');
    query.addEventListener('change', sync);

    return () => {
      observer.disconnect();
      query.removeEventListener('change', sync);
    };
  }, []);

  useEffect(() => {
    if (!code || dark === null) return;
    let live = true;

    void (async () => {
      try {
        const { default: mermaid } = await import('mermaid');

        mermaid.initialize({
          startOnLoad: false,
          // Escapes text in labels and refuses click handlers and raw HTML, so
          // the markup below is Mermaid's own SVG rather than anything the
          // diagram's author could smuggle through it.
          securityLevel: 'strict',
          theme: 'base',
          themeVariables: {
            ...themeVariables(),
            darkMode: dark ? 'true' : 'false',
          },
          fontFamily: getComputedStyle(document.body).fontFamily,
        });

        const { svg } = await mermaid.render(id, code);
        if (!live || !host.current) return;

        host.current.innerHTML = svg;
        setStatus('drawn');
      } catch (err) {
        if (!live) return;
        setError(err instanceof Error ? err.message : String(err));
        setStatus('failed');
      }
    })();

    return () => {
      live = false;
    };
  }, [code, dark, id]);

  if (!code) return null;

  return (
    <figure className="zellij-mermaid" data-status={status}>
      {status === 'drawn' ? null : (
        <pre className="zellij-mermaid-source">
          <code>{code}</code>
        </pre>
      )}
      <div ref={host} className="zellij-mermaid-canvas" aria-hidden={status !== 'drawn'} />
      {status === 'failed' ? (
        <figcaption className="zellij-mermaid-error" role="status">
          This diagram could not be drawn: {error}
        </figcaption>
      ) : null}
    </figure>
  );
}
