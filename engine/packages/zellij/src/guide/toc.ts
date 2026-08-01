import GithubSlugger from 'github-slugger';

export interface TocEntry {
  id: string;
  text: string;
  depth: 2 | 3;
}

/**
 * Uses the very library `rehype-slug` uses, rather than reimplementing its
 * rules. Parity has to be exact: a TOC anchor that differs from the rendered
 * heading id by one character is a dead link. (github-slugger does not collapse
 * whitespace — "SSO & SCIM" becomes "sso--scim" — which a hand-rolled slugifier
 * gets wrong.)
 */
export function slugifyHeading(text: string): string {
  return new GithubSlugger().slug(text);
}

const FENCE = /^(```|~~~)/;

/**
 * Extracts h2/h3 headings for the "On this page" rail (spec §7).
 *
 * Parses the raw MDX rather than the compiled output because the compile step
 * happens inside an RSC boundary we cannot read back from. Code fences are
 * skipped so a `# comment` inside a shell block is never mistaken for a heading.
 */
export function extractToc(markdown: string): TocEntry[] {
  const entries: TocEntry[] = [];
  // One slugger per document: it tracks duplicates and appends -1, -2, … in the
  // same order rehype-slug does while walking the same headings.
  const slugger = new GithubSlugger();
  let inFence = false;

  for (const rawLine of markdown.split('\n')) {
    const line = rawLine.trimEnd();

    if (FENCE.test(line.trimStart())) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    // Match every heading level, not just h2/h3: rehype-slug slugs them all, so
    // its duplicate counter would drift from ours if we skipped any.
    const match = /^(#{1,6})\s+(.+?)\s*#*$/.exec(line);
    if (!match) continue;

    const level = match[1]!.length;
    const text = stripInlineMarkdown(match[2]!);
    const id = slugger.slug(text);
    if (!id) continue;

    if (level === 2 || level === 3) {
      entries.push({ id, text, depth: level });
    }
  }

  return entries;
}

/** Removes the inline markdown that would otherwise leak into TOC labels. */
function stripInlineMarkdown(text: string): string {
  return text
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .trim();
}
