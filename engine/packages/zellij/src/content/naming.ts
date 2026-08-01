/**
 * Deriving routes and labels from folder and file names — design §9.
 *
 * Authors order guide folders by prefixing them with a number. The prefix is an
 * ordering instruction, not part of the site's identity, so it is stripped from
 * both the URL and the visible label: `01-getting-started/` is `/getting-started`
 * labelled "Getting started".
 */

/**
 * An ordering prefix: `01-`, `1.`, `02_`, or `10 - `. Anchored and requiring a
 * separator, so a folder genuinely named "2024" or "1984-in-review" keeps its
 * digits.
 */
const ORDER_PREFIX = /^(\d+)\s*[-._]\s*(?=\S)/;

export interface NamePrefix {
  /** The number the prefix carried, if there was one. */
  order?: number;
  /** The name with the prefix removed. */
  rest: string;
}

export function splitOrderPrefix(name: string): NamePrefix {
  const match = ORDER_PREFIX.exec(name);
  if (!match) return { rest: name };
  return { order: Number(match[1]), rest: name.slice(match[0].length) };
}

export function stripOrderPrefix(name: string): string {
  return splitOrderPrefix(name).rest;
}

/**
 * Splits camel and Pascal case into words. `MyFirstChapter` → `My First Chapter`,
 * and `APIReference` → `API Reference` — the second rule keeps acronyms whole by
 * breaking before the final capital of a run rather than between every letter.
 *
 * This exists because folder names like `1.MyFirstChapter` are a natural way to
 * write an ordered set of chapters, and `/myfirstchapter` is not an acceptable
 * route for one. A name that already contains separators is left alone.
 */
function splitCamelCase(name: string): string {
  if (/[\s\-_.]/.test(name)) return name;
  return name
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
    .replace(/([a-z\d])([A-Z])/g, '$1 $2');
}

/**
 * Turns one path segment into a URL slug. Diacritics are folded so that
 * `Café/` routes to `/cafe` rather than to a percent-encoded segment.
 */
export function slugifySegment(name: string): string {
  return splitCamelCase(stripOrderPrefix(name))
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/** Slugifies every segment of a `a/b/c` path. */
export function slugifyPath(relPath: string): string {
  return relPath
    .split('/')
    .map(slugifySegment)
    .filter((segment) => segment.length > 0)
    .join('/');
}

/**
 * A human label for a name with no explicit `title`. Sentence case, not title
 * case: "Getting started", not "Getting Started" — the latter reads as a proper
 * noun and is wrong for most headings.
 */
export function deriveLabel(name: string): string {
  const words = splitCamelCase(stripOrderPrefix(name))
    .replace(/[-_.]+/g, ' ')
    .split(/\s+/)
    .filter((word) => word.length > 0)
    // An all-caps run is an acronym and keeps its case; anything else is just a
    // word that happened to be capitalised by the camel split.
    .map((word) => (word === word.toUpperCase() && word.length > 1 ? word : word.toLowerCase()));

  const first = words[0];
  if (!first) return name;
  return [first.charAt(0).toUpperCase() + first.slice(1), ...words.slice(1)].join(' ');
}

/** Drops a `.md`, `.mdx`, `.yaml` or `.yml` extension. */
export function stripExtension(file: string): string {
  return file.replace(/\.(mdx?|ya?ml)$/i, '');
}

/**
 * The name a section file is referenced by: its path below `sections/` with the
 * extension removed. `sections/home/intro.yaml` is `home/intro`.
 */
export function sectionName(relFile: string): string {
  return stripExtension(relFile);
}
