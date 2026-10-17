import { describe, expect, it } from 'vitest';
import { parseFrontmatter } from './frontmatter';

describe('YAML front matter', () => {
  it('preserves a document without front matter, including later horizontal rules', () => {
    const content = '# Guide\n\n---\n\nBody';
    expect(parseFrontmatter(content)).toEqual({ data: {}, content });
    expect(parseFrontmatter('----\nBody').content).toBe('----\nBody');
  });

  it('reads nested values and multiline text without consuming body delimiters', () => {
    expect(parseFrontmatter('---\ntitle: "Guide: start"\ndescription: |\n  First line\n  Second line\nnested: { enabled: true }\n---\n# Body\n---\n')).toEqual({
      data: { title: 'Guide: start', description: 'First line\nSecond line\n', nested: { enabled: true } },
      content: '# Body\n---\n',
    });
  });

  it('accepts BOM, Windows newlines, and explicit YAML language', () => {
    expect(parseFrontmatter('\uFEFF---yaml\r\ntitle: Guide\r\n---\r\n# Body\r\n'))
      .toEqual({ data: { title: 'Guide' }, content: '# Body\r\n' });
    expect(parseFrontmatter('---\n# empty\n---').data).toEqual({});
    expect(parseFrontmatter('---\ntitle: Guide\n...\nBody').content).toBe('Body');
  });

  it('rejects malformed, unclosed, or non-mapping metadata', () => {
    for (const raw of ['---\ntitle: [\n---\n', '---\ntitle: Guide', '---\n- item\n---\n', '---\n42\n---\n']) {
      expect(() => parseFrontmatter(raw)).toThrow();
    }
  });

  it('never evaluates JavaScript front matter', () => {
    expect(() => parseFrontmatter('---javascript\n({ title: "executable" })\n---\nBody'))
      .toThrow('Unsupported front matter language');
  });
});
