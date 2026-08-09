import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { createElement } from 'react';
import { InlineMarkdown } from './shared';

const render = (text: string) => renderToStaticMarkup(createElement(InlineMarkdown, { text }));

describe('InlineMarkdown', () => {
  it('renders the four inline forms', () => {
    expect(render('**bold**')).toBe('<strong>bold</strong>');
    expect(render('*emphasis*')).toBe('<em>emphasis</em>');
    expect(render('`code`')).toBe('<code>code</code>');
    expect(render('[label](/somewhere)')).toContain('href="/somewhere"');
  });

  /**
   * Regression: one flat pass read ``**a `b` c**`` as bold containing three
   * literal backticks, because the bold alternative matched first and nothing
   * looked inside it again. Naming a command inside a bold clause is ordinary
   * writing, and it reached the page with its punctuation showing.
   */
  it('parses emphasis inside emphasis', () => {
    expect(render('**In `recipient` mode**')).toBe(
      '<strong>In <code>recipient</code> mode</strong>',
    );
    expect(render('*the `--full` flag*')).toBe('<em>the <code>--full</code> flag</em>');
    expect(render('**bold with *emphasis* inside**')).toContain('<em>emphasis</em>');
  });

  /**
   * The exception, and deliberate: backticks mean "this is literal". An
   * asterisk in a shell glob is an asterisk, not the start of emphasis.
   */
  it('does not parse markdown inside code', () => {
    expect(render('`rm *.tmp`')).toBe('<code>rm *.tmp</code>');
    expect(render('`**not bold**`')).toBe('<code>**not bold**</code>');
  });

  it('leaves plain text alone', () => {
    expect(render('nothing to see here')).toBe('nothing to see here');
  });
});
