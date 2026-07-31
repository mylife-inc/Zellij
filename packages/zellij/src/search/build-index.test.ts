import { describe, expect, it } from 'vitest';
import { buildSearchIndex, indexGuidePage, mdxToText } from './build-index';
import type { GuidePage } from '../content/types';

function guide(body: string, overrides: Partial<GuidePage> = {}): GuidePage {
  return {
    route: '/guide/test',
    sourceFile: 'guide/test.mdx',
    title: 'Test',
    body,
    ...overrides,
  };
}

describe('mdxToText', () => {
  it('drops fenced code blocks', () => {
    const text = mdxToText('Before\n\n```bash\ndocker run acme\n```\n\nAfter');
    expect(text).toBe('Before After');
    // A search for a term should find the prose explaining it, not the snippet.
    expect(text).not.toContain('docker run');
  });

  it('drops inline code but keeps surrounding prose', () => {
    expect(mdxToText('Run `acme login` to sign in.')).toBe('Run to sign in.');
  });

  it('strips JSX components while keeping their text', () => {
    expect(mdxToText('<Callout type="info">Needs Node 20.</Callout>')).toBe('Needs Node 20.');
  });

  it('keeps link text and drops the target', () => {
    expect(mdxToText('See the [quickstart](/guide/quickstart) guide.')).toBe(
      'See the quickstart guide.',
    );
  });

  it('drops images entirely', () => {
    expect(mdxToText('![a screenshot](assets/shot.png) Body.')).toBe('Body.');
  });

  it('removes heading, list and emphasis syntax', () => {
    expect(mdxToText('## Setup\n\n- **bold** item\n- _italic_ item')).toBe(
      'Setup bold item italic item',
    );
  });
});

describe('indexGuidePage', () => {
  const page = guide(
    [
      '# Installation',
      '',
      'Acme runs hosted or self-managed.',
      '',
      '## Hosted',
      '',
      'Nothing to install.',
      '',
      '## Self-hosted',
      '',
      'Requires Postgres 15.',
    ].join('\n'),
    { route: '/guide/installation', title: 'Installation' },
  );

  it('creates one entry for the page intro and one per section', () => {
    const entries = indexGuidePage(page);
    expect(entries.map((e) => e.title)).toEqual(['Installation', 'Hosted', 'Self-hosted']);
  });

  it('anchors section entries so a hit lands on the heading', () => {
    const entries = indexGuidePage(page);
    expect(entries[1]?.route).toBe('/guide/installation#hosted');
    expect(entries[2]?.route).toBe('/guide/installation#self-hosted');
  });

  it('keeps the page title as context on every entry', () => {
    expect(indexGuidePage(page).every((e) => e.page === 'Installation')).toBe(true);
  });

  it('stops the intro at the first subheading', () => {
    const [intro] = indexGuidePage(page);
    expect(intro?.text).toContain('hosted or self-managed');
    expect(intro?.text).not.toContain('Nothing to install');
  });

  it('includes the frontmatter description in the page entry', () => {
    const [intro] = indexGuidePage(guide('# T\n\nBody.', { description: 'A summary.' }));
    expect(intro?.text).toContain('A summary.');
  });

  it('ignores headings inside fenced code', () => {
    const entries = indexGuidePage(
      guide('# T\n\n```md\n## Not a heading\n```\n\n## Real\n\nActual prose.'),
    );
    expect(entries.map((e) => e.title)).toEqual(['Test', 'Real']);
  });

  /**
   * rehype-slug slugs every heading level when rendering, so the indexer must
   * too — otherwise duplicate headings get different suffixes here than in the
   * document, and results link to anchors that do not exist.
   */
  it('numbers duplicate headings the way the rendered page does', () => {
    const entries = indexGuidePage(guide('# Setup\n\n## Setup\n\nBody.\n\n## Setup\n\nMore.'));
    expect(entries.map((e) => e.route)).toEqual([
      '/guide/test',
      '/guide/test#setup-1',
      '/guide/test#setup-2',
    ]);
  });

  it('skips sections with no prose', () => {
    const entries = indexGuidePage(guide('# T\n\n## Empty\n\n```bash\nls\n```'));
    expect(entries.map((e) => e.title)).toEqual(['Test']);
  });
});

describe('buildSearchIndex', () => {
  it('indexes every guide page', () => {
    const entries = buildSearchIndex([
      guide('# One\n\nAlpha.', { route: '/guide/one', title: 'One' }),
      guide('# Two\n\nBeta.', { route: '/guide/two', title: 'Two' }),
    ]);
    expect(entries.map((e) => e.route)).toEqual(['/guide/one', '/guide/two']);
  });

  it('returns nothing for an empty guide', () => {
    expect(buildSearchIndex([])).toEqual([]);
  });
});
