import GithubSlugger from 'github-slugger';
import { readHeadings, type HeadingOptions } from '../mdx/headings';

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

/** Use the same parsed heading IDs as search and rendered Markdown. */
export function extractToc(markdown: string, options: HeadingOptions = {}): TocEntry[] {
  return readHeadings(markdown, options).flatMap(({ id, text, depth }) =>
    id && (depth === 2 || depth === 3) ? [{ id, text, depth }] : [],
  );
}
