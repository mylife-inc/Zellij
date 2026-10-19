'use client';

import { loadMermaid } from '@shebka/zellij/optional-mermaid';
import { useEffect, useId, useRef, useState } from 'react';

type Status = 'pending' | 'drawn' | 'failed';

/** The site's mode attribute carries `system`, which only the browser can resolve. */
function prefersDark(): boolean {
  const declared = document.documentElement.getAttribute('data-zellij-mode');
  if (declared === 'dark') return true;
  if (declared === 'light') return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

/** Resolves a CSS colour of any notation to `[r, g, b]` by letting the browser parse it. */
function toRgb(colour: string): [number, number, number] {
  const probe = document.createElement('span');
  probe.style.color = colour;
  probe.style.display = 'none';
  document.body.append(probe);
  const resolved = getComputedStyle(probe).color;
  probe.remove();

  const parts = resolved.match(/-?[\d.]+/g);
  if (!parts || parts.length < 3) return [0, 0, 0];
  return [Number(parts[0]), Number(parts[1]), Number(parts[2])];
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  const [rn, gn, bn] = [r / 255, g / 255, b / 255];
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];

  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h =
    max === rn
      ? ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6
      : max === gn
        ? ((bn - rn) / d + 2) / 6
        : ((rn - gn) / d + 4) / 6;
  return [h * 360, s, l];
}

/**
 * A branch palette, spun out of the theme's own accent.
 *
 * Colour is how a mindmap tells one branch from another, so a single fill makes
 * every limb look alike — but Mermaid's stock palette belongs to no theme in
 * particular. Rotating the accent's hue keeps the family resemblance while
 * still giving each branch its own identity, and it works for any accent the
 * site happens to have rather than a list of colours we picked in advance.
 *
 * The golden-angle step is what keeps neighbouring branches far apart in hue
 * instead of drifting through near-identical shades.
 */
function branchPalette(accent: string, count: number, dark: boolean): string[] {
  const [h, s, l] = rgbToHsl(...toRgb(accent));

  // Saturation and lightness are pinned rather than inherited: a very pale or
  // very dark accent would otherwise produce a set of branches nobody can tell
  // apart, or read against the page.
  const saturation = Math.round(Math.min(Math.max(s, 0.45), 0.7) * 100);
  const lightness = dark ? 68 : 42;

  return Array.from({ length: count }, (_, i) => {
    const hue = Math.round((h + i * 137.5) % 360);
    return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
  });
}

/**
 * Dresses Mermaid in the site's own tokens.
 *
 * Left to itself Mermaid paints in its stock lilac-and-lime palette, which is
 * legible but belongs to no theme in particular — a diagram that plainly came
 * from somewhere else. Reading the live custom properties keeps a diagram in
 * step with whatever theme is showing, including one switched at runtime.
 */
function themeVariables(dark: boolean): Record<string, string> {
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
    /*
     * Sequence diagrams keep their own set of tokens, and the ones left unset
     * fall back to Mermaid's palette rather than to the ones above.
     *
     * `actorTextColor` was the one that showed: with `actorBkg` following the
     * theme's surface and the text left at Mermaid's default, a dark theme drew
     * near-black participant labels on a near-black box. The names were there
     * and could not be read.
     *
     * `sequenceNumberColor` is the digit inside the numbered circle, and it has
     * to oppose the circle rather than match the page — Mermaid fills that
     * circle from `signalColor`, which is the foreground. So the digit takes
     * the background: dark circle and light digit on a light page, and both
     * inverted on a dark one.
     */
    actorBkg: surface,
    actorBorder: accent,
    actorTextColor: fg,
    actorLineColor: token('muted', '#5f6672'),
    signalColor: fg,
    signalTextColor: fg,
    sequenceNumberColor: bg,
    noteBkgColor: surface,
    noteTextColor: fg,
    noteBorderColor: border,

    /*
     * Mindmaps and pie charts colour their branches from a numbered scale
     * rather than from `primaryColor`, so without these they keep Mermaid's own
     * palette while every other diagram type follows the theme — which looked
     * exactly like the bug it was. One fill for every branch: a mindmap's
     * structure is already carried by its shape, so colour has nothing left to
     * say and only competes with the page.
     *
     * `cScale` sets the branch's node outline, its label and the connector
     * running into it, all three at once — Mermaid offers no way to separate
     * them, and styles them through an id selector that CSS cannot outrank
     * afterwards. So the palette has to be right here or not at all.
     */
    ...Object.fromEntries(
      branchPalette(accent, 12, dark)
        .map((colour, i) => {
          // Mermaid numbers sections from -1, so index 0 is the root rather
          // than a branch. It gets the plain foreground: the root is the one
          // node that belongs to no branch, and colouring it made its label
          // unreadable against its own fill in dark mode.
          const ink = i === 0 ? fg : colour;
          return [
            [`cScale${i}`, i === 0 ? border : colour],
            [`cScaleLabel${i}`, ink],
            [`cScaleInv${i}`, ink],
          ];
        })
        .flat(),
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
    if (!code || dark === null || process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS === 'false') return;
    let live = true;

    void (async () => {
      try {
        const mermaid = await loadMermaid();

        mermaid.initialize({
          startOnLoad: false,
          // Escapes text in labels and refuses click handlers and raw HTML, so
          // the markup below is Mermaid's own SVG rather than anything the
          // diagram's author could smuggle through it.
          securityLevel: 'strict',
          /*
           * `neutral` and `neo` together are what make a mindmap's nodes
           * outlines rather than blocks of colour: only that pairing fills a
           * node from `mainBkg` while stroking it with the branch's own colour.
           * `neo` also thins Mermaid's connectors from seventeen pixels to two,
           * which is most of the difference between a diagram that looks drawn
           * and one that looks extruded.
           */
          theme: 'neutral',
          look: 'neo',
          themeVariables: {
            ...themeVariables(dark),
            darkMode: dark ? 'true' : 'false',
          },
          // Straight-line elbows read as a wiring diagram; curves read as a
          // map, which is what these mostly are.
          flowchart: { curve: 'basis' },
          /*
           * `mirrorActors: false` drops the copy of every participant that
           * Mermaid repeats along the bottom edge.
           *
           * That repetition earns its place in a diagram long enough that the
           * lifelines outrun the labels — a printed protocol trace. On a web
           * page the whole diagram is one glance, and the second row reads as a
           * second set of participants until you work out that it is not.
           *
           * `useMaxWidth` lets a wide diagram shrink to its column instead of
           * pushing a scrollbar onto the page, which matters most on a phone
           * where a five-participant trace is wider than the screen.
           */
          sequence: { mirrorActors: false, useMaxWidth: true },
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
    <figure className="zellij-mermaid" data-status={process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS === 'false' ? 'disabled' : status}>
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
