import { describe, expect, it } from 'vitest';
import { extractToc, slugifyHeading } from './toc';
import { buildBreadcrumbs, findNeighbours, flattenSidebar } from './navigation';
import type { SidebarNode } from '../content/types';

describe('slugifyHeading', () => {
  it('matches the slugs rehype-slug generates', () => {
    expect(slugifyHeading('Core settings')).toBe('core-settings');
    expect(slugifyHeading('Read replicas!')).toBe('read-replicas');
    expect(slugifyHeading('SSO & SCIM')).toBe('sso--scim');
  });
});

describe('extractToc', () => {
  it('collects h2 and h3 headings, ignoring h1', () => {
    const toc = extractToc(['# Title', '## First', '### Nested', '## Second'].join('\n'));
    expect(toc).toEqual([
      { id: 'first', text: 'First', depth: 2 },
      { id: 'nested', text: 'Nested', depth: 3 },
      { id: 'second', text: 'Second', depth: 2 },
    ]);
  });

  it('ignores headings inside fenced code blocks', () => {
    const toc = extractToc(
      ['## Real', '', '```bash', '# not a heading', '## also not', '```', '', '## Also real'].join(
        '\n',
      ),
    );
    expect(toc.map((entry) => entry.text)).toEqual(['Real', 'Also real']);
  });

  it('strips inline markdown from labels', () => {
    const toc = extractToc('## Using `acme login` and **bold** and [a link](/x)');
    expect(toc[0]?.text).toBe('Using acme login and bold and a link');
  });

  it('disambiguates duplicate headings the way rehype-slug does', () => {
    const toc = extractToc(['## Setup', '## Setup', '## Setup'].join('\n'));
    expect(toc.map((entry) => entry.id)).toEqual(['setup', 'setup-1', 'setup-2']);
  });

  it('counts skipped heading levels so ids stay aligned with rehype-slug', () => {
    // rehype-slug slugs the h1 too, so the h2 below it must be "setup-1".
    const toc = extractToc(['# Setup', '## Setup'].join('\n'));
    expect(toc.map((entry) => entry.id)).toEqual(['setup-1']);
  });

  it('returns nothing for a document with no subheadings', () => {
    expect(extractToc('# Only a title\n\nSome prose.')).toEqual([]);
  });
});

const SIDEBAR: SidebarNode[] = [
  {
    kind: 'group',
    label: 'Getting Started',
    children: [
      { kind: 'link', label: 'Quickstart', route: '/guide/quickstart' },
      { kind: 'link', label: 'Installation', route: '/guide/installation' },
    ],
  },
  {
    kind: 'group',
    label: 'Configuration',
    children: [
      { kind: 'link', label: 'Configuration overview', route: '/guide/config/overview' },
      { kind: 'link', label: 'Database', route: '/guide/config/database' },
    ],
  },
];

/** A group inside a group — the shape the resolver used to discard. */
const NESTED: SidebarNode[] = [
  {
    kind: 'group',
    label: 'Guide',
    children: [
      { kind: 'link', label: 'Intro', route: '/guide/intro' },
      {
        kind: 'group',
        label: 'Advanced',
        children: [
          {
            kind: 'group',
            label: 'Tuning',
            children: [{ kind: 'link', label: 'Cache', route: '/guide/adv/cache' }],
          },
          { kind: 'link', label: 'Scaling', route: '/guide/adv/scaling' },
        ],
      },
    ],
  },
];

describe('guide navigation', () => {
  it('flattens the sidebar into reading order', () => {
    expect(flattenSidebar(SIDEBAR).map((entry) => entry.route)).toEqual([
      '/guide/quickstart',
      '/guide/installation',
      '/guide/config/overview',
      '/guide/config/database',
    ]);
  });

  it('finds neighbours across section boundaries', () => {
    const { previous, next } = findNeighbours(SIDEBAR, '/guide/installation');
    expect(previous?.route).toBe('/guide/quickstart');
    expect(next?.route).toBe('/guide/config/overview');
  });

  it('omits previous on the first page and next on the last', () => {
    expect(findNeighbours(SIDEBAR, '/guide/quickstart').previous).toBeUndefined();
    expect(findNeighbours(SIDEBAR, '/guide/config/database').next).toBeUndefined();
  });

  it('returns no neighbours for a page absent from the sidebar', () => {
    expect(findNeighbours(SIDEBAR, '/guide/orphan')).toEqual({});
  });

  it('walks nested groups when flattening reading order', () => {
    expect(flattenSidebar(NESTED).map((entry) => entry.route)).toEqual([
      '/guide/intro',
      '/guide/adv/cache',
      '/guide/adv/scaling',
    ]);
  });

  it('finds neighbours across nesting levels', () => {
    const { previous, next } = findNeighbours(NESTED, '/guide/adv/cache');
    expect(previous?.route).toBe('/guide/intro');
    expect(next?.route).toBe('/guide/adv/scaling');
  });

  it('includes every ancestor group in breadcrumbs, not just the first', () => {
    expect(buildBreadcrumbs(NESTED, '/guide/adv/cache', 'Cache')).toEqual([
      { label: 'Guide', route: '/guide' },
      { label: 'Guide' },
      { label: 'Advanced' },
      { label: 'Tuning' },
      { label: 'Cache' },
    ]);
  });

  it('builds breadcrumbs from the sidebar section, not the file path', () => {
    expect(buildBreadcrumbs(SIDEBAR, '/guide/config/database', 'Database')).toEqual([
      { label: 'Guide', route: '/guide' },
      { label: 'Configuration' },
      { label: 'Database' },
    ]);
  });

  it('keeps the guide index breadcrumb to a single crumb', () => {
    expect(buildBreadcrumbs(SIDEBAR, '/guide', 'Guide')).toEqual([
      { label: 'Guide', route: '/guide' },
    ]);
  });
});
