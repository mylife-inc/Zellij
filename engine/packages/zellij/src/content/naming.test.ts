import { describe, expect, it } from 'vitest';
import { deriveLabel, sectionName, slugifyPath, slugifySegment, splitOrderPrefix } from './naming';

describe('splitOrderPrefix', () => {
  it('reads the separators authors actually use', () => {
    expect(splitOrderPrefix('01-getting-started')).toEqual({ order: 1, rest: 'getting-started' });
    expect(splitOrderPrefix('1.MyFirstChapter')).toEqual({ order: 1, rest: 'MyFirstChapter' });
    expect(splitOrderPrefix('02_advanced')).toEqual({ order: 2, rest: 'advanced' });
    expect(splitOrderPrefix('10 - appendix')).toEqual({ order: 10, rest: 'appendix' });
  });

  it('leaves a name that merely starts with digits alone', () => {
    expect(splitOrderPrefix('2024')).toEqual({ rest: '2024' });
    expect(splitOrderPrefix('1984')).toEqual({ rest: '1984' });
  });

  /** A prefix with nothing after it is a folder name, not an ordering. */
  it('does not strip a prefix that would leave an empty name', () => {
    expect(splitOrderPrefix('01-')).toEqual({ rest: '01-' });
  });
});

describe('slugifySegment', () => {
  it('drops the ordering prefix from the route', () => {
    expect(slugifySegment('01-getting-started')).toBe('getting-started');
  });

  it('splits camel case so a chapter folder gets a readable route', () => {
    expect(slugifySegment('1.MyFirstChapter')).toBe('my-first-chapter');
  });

  /** Breaking between every capital would give "a-p-i-reference". */
  it('keeps acronyms whole', () => {
    expect(slugifySegment('APIReference')).toBe('api-reference');
  });

  it('folds diacritics rather than percent-encoding them', () => {
    expect(slugifySegment('Café')).toBe('cafe');
  });

  it('leaves a name that already has separators alone', () => {
    expect(slugifySegment('using-payos')).toBe('using-payos');
  });
});

describe('slugifyPath', () => {
  it('slugifies each segment independently', () => {
    expect(slugifyPath('01-guide/config/Database Setup')).toBe('guide/config/database-setup');
  });

  it('drops empty segments', () => {
    expect(slugifyPath('guide//intro')).toBe('guide/intro');
  });
});

describe('deriveLabel', () => {
  it('produces sentence case, not title case', () => {
    expect(deriveLabel('01-getting-started')).toBe('Getting started');
    expect(deriveLabel('MyFirstChapter')).toBe('My first chapter');
    expect(deriveLabel('usage_platform_admin')).toBe('Usage platform admin');
  });
});

describe('sectionName', () => {
  it('is the path below sections/ with the extension dropped', () => {
    expect(sectionName('home/hero.yaml')).toBe('home/hero');
    expect(sectionName('why.mdx')).toBe('why');
  });
});
