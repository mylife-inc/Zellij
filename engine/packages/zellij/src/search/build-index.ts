import GithubSlugger from 'github-slugger';
import type { GuidePage } from '../content/types';

/**
 * This module is deliberately self-contained apart from `github-slugger`: the
 * build-time CLI imports it directly under Node, where every relative import
 * would need an explicit file extension. Keeping the graph to one hop means one
 * such import, and the app still consumes this module normally through the
 * bundler.
 */

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
export function indexGuidePage(page: GuidePage): SearchEntry[] {
  // A fresh slugger per page, matching how rehype-slug numbers duplicates, so
  // a result's anchor resolves to the heading actually rendered.
  const slugger = new GithubSlugger();
  const entries: SearchEntry[] = [];

  // Everything before the first h2/h3 belongs to the page itself.
  const firstHeadingIndex = findFirstHeadingIndex(page.body);
  const intro = mdxToText(
    firstHeadingIndex === -1 ? page.body : page.body.slice(0, firstHeadingIndex),
  );

  entries.push({
    id: page.route,
    route: page.route,
    page: page.title,
    title: page.title,
    text: [page.description ?? '', intro].filter(Boolean).join(' ').trim(),
  });

  for (const section of splitByHeadings(page.body)) {
    // Every heading level is slugged, in document order, because rehype-slug
    // does the same when rendering — skipping h1 would shift the -1, -2
    // suffixes on duplicates and produce anchors that resolve nowhere.
    const anchor = slugger.slug(section.heading);

    // Only h2/h3 become their own search results.
    if (section.level > 3) continue;
    if (section.level < 2) continue;

    const text = mdxToText(section.body);
    if (!text) continue;

    entries.push({
      id: `${page.route}#${anchor}`,
      route: `${page.route}#${anchor}`,
      page: page.title,
      title: section.heading,
      text,
    });
  }

  return entries.filter((entry) => entry.text.length > 0 || entry.title.length > 0);
}

/** Marketing pages are excluded from the index by design (spec §7.3). */
export function buildSearchIndex(guidePages: readonly GuidePage[]): SearchEntry[] {
  return guidePages.flatMap(indexGuidePage);
}

const HEADING = /^(#{1,6})\s+(.+?)\s*#*$/;

interface Section {
  level: number;
  heading: string;
  body: string;
}

/** Offset of the first h2/h3, i.e. where the page's introduction ends. */
function findFirstHeadingIndex(source: string): number {
  let offset = 0;
  let inFence = false;

  for (const line of source.split('\n')) {
    const trimmed = line.trimStart();
    if (trimmed.startsWith('```') || trimmed.startsWith('~~~')) {
      inFence = !inFence;
    } else if (!inFence) {
      const match = HEADING.exec(line);
      if (match && match[1]!.length >= 2) return offset;
    }
    offset += line.length + 1;
  }
  return -1;
}

/** All headings, every level, in document order — fenced code ignored. */
function splitByHeadings(source: string): Section[] {
  const sections: Section[] = [];
  let current: { level: number; heading: string; body: string[] } | undefined;
  let inFence = false;

  const flush = (): void => {
    if (current) {
      sections.push({ level: current.level, heading: current.heading, body: current.body.join('\n') });
    }
  };

  for (const line of source.split('\n')) {
    const trimmed = line.trimStart();
    if (trimmed.startsWith('```') || trimmed.startsWith('~~~')) inFence = !inFence;

    const match = inFence ? null : HEADING.exec(line);
    if (match) {
      flush();
      current = { level: match[1]!.length, heading: match[2]!, body: [] };
    } else if (current) {
      current.body.push(line);
    }
  }

  flush();
  return sections;
}
