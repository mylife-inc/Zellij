import { describe, expect, it } from 'vitest';
import { buildSearchIndex, indexGuidePage, mdxToText } from './build-index';
import type { GuidePage } from '../content/types';
import remarkMath from 'remark-math';

function guide(body: string, overrides: Partial<GuidePage> = {}): GuidePage {
  return {
    route: '/guide/test',
    sourceFile: 'guide/test.mdx',
    title: 'Test',
    body,
    guide: '/guide',
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

  it.each([
    '````text\n```\nfenced_only_sentinel\n````',
    '~~~~text\n~~~\nfenced_only_sentinel\n~~~~',
    '`````text\n# literal heading\n````mdx\n[fenced_only_sentinel](../missing.md)\n````\n`````',
    '```text\nfenced_only_sentinel\n`````',
    '~~~text\nfenced_only_sentinel\n~~~~~',
    '```text\n~~~\nfenced_only_sentinel\n~~~\n```',
    '~~~text\n```\nfenced_only_sentinel\n```\n~~~',
    '```text\n```not-a-closing-fence\nfenced_only_sentinel\n```',
    '   ````text\n   ```\n   fenced_only_sentinel\n   ````',
    '> ````text\n> ```\n> fenced_only_sentinel\n> ````',
    '- ````text\n  ```\n  fenced_only_sentinel\n  ````',
    '<Callout>\n\n````text\n```\nfenced_only_sentinel\n````\n\n</Callout>',
  ])('drops the whole parsed code block: %s', (block) => {
    expect(mdxToText(`Before\n\n${block}\n\nAfter`)).toBe('Before After');
  });

  it.each(['```text', '~~~~text'])('drops an unclosed fence through the end of the document: %s', (fence) => {
    expect(mdxToText(`Before\n\n${fence}\nfenced_only_sentinel\n## Still code`)).toBe('Before');
  });

  it.each([
    '`` inline ` sentinel ``',
    '``` inline `` sentinel ```',
    '`` inline\nsentinel ``',
  ])('drops complete inline code spans: %s', (code) => {
    expect(mdxToText(`Before ${code} After`)).toBe('Before After');
  });

  it('keeps unmatched and escaped backticks as prose', () => {
    expect(mdxToText('Before `unmatched After')).toBe('Before `unmatched After');
    expect(mdxToText('Before \\`escaped sentinel\\` After')).toContain('escaped sentinel');
  });

  it('does not treat backticks in a JSX attribute as Markdown code', () => {
    expect(mdxToText('<Callout title="`">Keep this prose.</Callout>\n\nAfter `hidden` code.')).toBe('Keep this prose. After code.');
  });

  it('uses the configured parser extensions when identifying code', () => {
    const source = 'Before $a `mathtoken` b$ after and `codetoken`.';
    const text = mdxToText(source, { remarkPlugins: [remarkMath] });
    expect(text).toContain('mathtoken');
    expect(text).not.toContain('codetoken');
    expect(indexGuidePage(guide(source), { remarkPlugins: [remarkMath] })[0]?.text).toBe(text);
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

  it('keeps deeper subsections searchable within their parent passage', () => {
    const entries = indexGuidePage(guide('## Setup\n\nOverview.\n\n#### Detail\n\nUnique detail.\n\n## Detail\n\nNext passage.'));
    expect(entries[1]?.text).toContain('Unique detail.');
    expect(entries[1]?.text).not.toContain('Next passage.');
    expect(entries[2]?.route).toBe('/guide/test#detail-1');
  });

  it('retains section offsets with CRLF, Unicode and nested fenced examples', () => {
    const source = [
      '# Résumé 🚀', '', 'Before 🚀.', '', '````md', '```', 'hiddenintro', '````', '',
      '## First', '', 'First prose.', '', '> ~~~~text', '> ~~~', '> hiddenquote', '> ~~~~', '',
      '## Empty', '', '`````text', '```', 'hiddensection', '`````', '',
      '## Last', '', 'Last prose.', '', '- ````text', '  ```', '  hiddenlist', '  ````', '',
    ].join('\r\n');
    expect(indexGuidePage(guide(source))).toEqual([
      {id:'/guide/test', route:'/guide/test', page:'Test', title:'Test', text:'Résumé 🚀 Before 🚀.'},
      {id:'/guide/test#first', route:'/guide/test#first', page:'Test', title:'First', text:'First prose.'},
      {id:'/guide/test#last', route:'/guide/test#last', page:'Test', title:'Last', text:'Last prose.'},
    ]);
  });

  it('parses complete MDX before splitting sections inside components', () => {
    const source = [
      '<Callout>', '', 'Introduction.', '', '## Nested', '',
      'Nested prose.', '', '````md', '```', 'hiddennested', '````', '',
      '</Callout>', '', '## Outside', '', 'Outside prose.',
    ].join('\n');
    expect(indexGuidePage(guide(source)).map(({route, text}) => ({route, text}))).toEqual([
      {route:'/guide/test', text:'Introduction.'},
      {route:'/guide/test#nested', text:'Nested prose.'},
      {route:'/guide/test#outside', text:'Outside prose.'},
    ]);
  });

  it('retains code in heading labels and never evaluates authored JavaScript', () => {
    const source = 'export const value = (() => { throw new Error("must not execute"); })();\n\n## Use `zel`\n\nRead this prose.\n\n````text\n```\nhiddencode\n````';
    const entries = indexGuidePage(guide(source));
    expect(entries[1]).toMatchObject({route:'/guide/test#use-zel', title:'Use zel', text:'Read this prose.'});
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
