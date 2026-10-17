import { readHeadings, type HeadingOptions } from '../mdx/headings';
import { ZellijContentError } from '../errors';
import type { GuidePage } from '../content/types';

export interface SearchEntry {
  /** Stable id: route plus optional heading anchor. */
  id: string;
  route: string;
  /** Page title, shown as the result's context line. */
  page: string;
  /** Heading this passage sits under, or the page title for the intro. */
  title: string;
  /** Plain-text body used for matching and for the result snippet. */
  text: string;
}

/**
 * Strips MDX down to prose.
 *
 * Code blocks are dropped rather than indexed: a search for "docker" should
 * find the paragraph explaining Docker, not every fenced snippet that happens
 * to mention it.
 */
export function mdxToText(source: string): string {
  return (
    source
      // Fenced code, then inline code.
      .replace(/```[\s\S]*?```/g, ' ')
      .replace(/~~~[\s\S]*?~~~/g, ' ')
      .replace(/`[^`]*`/g, ' ')
      // JSX/HTML tags, keeping any text between them.
      .replace(/<\/?[A-Za-z][^>]*>/g, ' ')
      // Images before links, so alt text does not leak in as a label.
      .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
      .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
      // Emphasis, headings, list markers, blockquotes, table pipes.
      .replace(/[*_]{1,3}([^*_]+)[*_]{1,3}/g, '$1')
      .replace(/^\s{0,3}#{1,6}\s+/gm, '')
      .replace(/^\s*[-*+]\s+/gm, '')
      .replace(/^\s*>\s?/gm, '')
      .replace(/\|/g, ' ')
      .replace(/\s+/g, ' ')
      .trim()
  );
}

/**
 * Splits a guide page into one entry per h2/h3 section, so a hit lands on the
 * relevant heading rather than the top of a long page.
 */
export function indexGuidePage(page: GuidePage, options: HeadingOptions = {}): SearchEntry[] {
  const headings = readHeadings(page.body, options).filter((heading) => heading.depth === 2 || heading.depth === 3);
  const entries: SearchEntry[] = [];
  const intro = mdxToText(page.body.slice(0, headings[0]?.start));

  entries.push({
    id: page.route,
    route: page.route,
    page: page.title,
    title: page.title,
    text: [page.description ?? '', intro].filter(Boolean).join(' ').trim(),
  });

  for (const [index, heading] of headings.entries()) {
    const text = mdxToText(page.body.slice(heading.end, headings[index + 1]?.start));
    if (!text) continue;
    entries.push({
      id: `${page.route}#${heading.id}`,
      route: `${page.route}#${heading.id}`,
      page: page.title,
      title: heading.text,
      text,
    });
  }

  return entries.filter((entry) => entry.text.length > 0 || entry.title.length > 0);
}

/** Marketing pages are excluded from the index by design (spec §7.3). */
export function buildSearchIndex(guidePages: readonly GuidePage[], options: HeadingOptions = {}): SearchEntry[] {
  return guidePages.flatMap((page) => {
    try {
      return indexGuidePage(page, options);
    } catch (error) {
      throw new ZellijContentError(`Cannot index ${page.sourceFile}: ${(error as Error).message}`, page.sourceFile);
    }
  });
}
