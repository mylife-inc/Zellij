import { parse } from 'yaml';

/** YAML front matter shared by prose sections, guides, and the search index. */
export function parseFrontmatter(raw: string): { data: Record<string, unknown>; content: string } {
  const source = raw.replace(/^\uFEFF/, '');
  const opening = /^---([^\r\n]*)(?:\r?\n|$)/.exec(source);
  if (!opening || opening[1]!.startsWith('-')) return { data: {}, content: source };

  // Content is data. Never dispatch language tags to executable JS parsers.
  const language = opening[1]!.trim();
  if (language && language !== 'yaml' && language !== 'yml') {
    throw new Error(`Unsupported front matter language: ${language}. Use YAML.`);
  }
  const rest = source.slice(opening[0].length);
  const closing = /^(?:---|\.\.\.)[\t ]*(?:\r?\n|$)/m.exec(rest);
  if (!closing) throw new Error('YAML front matter needs a closing --- delimiter.');

  const data: unknown = parse(rest.slice(0, closing.index));
  if (data != null && (typeof data !== 'object' || Array.isArray(data))) {
    throw new Error('YAML front matter must be a mapping of field names to values.');
  }
  return {
    data: (data ?? {}) as Record<string, unknown>,
    content: rest.slice(closing.index + closing[0].length),
  };
}
