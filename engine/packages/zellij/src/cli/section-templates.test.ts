import { describe, expect, it } from 'vitest';
import { parse } from 'yaml';
import { buildSectionTemplates } from './section-templates';
import { buildSchemaJson } from './context';
import { getSection, registeredSectionTypes } from '../sections/registry';
import { sectionAuthoringRules } from '../sections/authoring-rules';

describe('embedded section templates', () => {
  const templates = buildSectionTemplates();

  it.each(registeredSectionTypes())('%s passes the actual engine schema', (type) => {
    expect(() => getSection(type)!.schema.parse(parse(templates[type]!))).not.toThrow();
  });

  it('embeds exactly the templates validated by the engine for Rust to consume', () => {
    expect(JSON.parse(buildSchemaJson()).sectionTemplates).toEqual(templates);
    expect(JSON.parse(buildSchemaJson()).sectionAuthoringRules).toEqual(sectionAuthoringRules);
    expect(Object.keys(templates)).toEqual(registeredSectionTypes());
  });

  it('keeps optional fields discoverable without making them live', () => {
    const hero = templates.hero!;
    expect(parse(hero)).not.toHaveProperty('badge');
    expect(hero).toContain('# badge:');
    expect(hero).toContain('floating | framed | full-bleed');
    const optional = hero.split('\n').filter((line) => line.startsWith('# '))
      .filter((line) => !line.includes('Uncomment') && !line.includes(' | '))
      .map((line) => line.slice(2)).join('\n');
    expect(parse(optional).cta).toEqual([{ label: 'TODO', href: '#' }]);
  });

  it('preserves nested object shapes and matrix dimensions', () => {
    const cards = parse(templates['code-cards']!).cards;
    expect(cards.length).toBeGreaterThanOrEqual(2);
    expect(cards[0].code).toEqual({ body: 'TODO' });
    const matrix = parse(templates.matrix!);
    expect(matrix.rows.every((row: { cells: string[] }) => row.cells.length === matrix.columns.length)).toBe(true);
    expect(parse(templates['hero-split']!).terminal).toEqual({ body: '$ zel --help' });
  });

  it('keeps text-or-number starters editable when schemas use primitive type arrays', () => {
    expect(parse(templates['stats-band']!).stats[0].value).toBe('TODO');
    expect(parse(templates.timeline!).items[0].date).toBe('TODO');
  });
});


it('exports the same cross-field constraints enforced by engine validation', () => {
  const templates = buildSectionTemplates();
  for (const [type, rules] of Object.entries(sectionAuthoringRules)) {
    const value = parse(templates[type]!);
    if (rules.exactlyOne) {
      for (const field of rules.exactlyOne) delete value[field];
      expect(getSection(type)!.schema.safeParse(value).success).toBe(false);
      Object.assign(value, parse(templates[type]!));
      const missing = rules.exactlyOne.find((field) => !(field in value))!;
      value[missing] = missing === 'terminal' ? { body: 'hello' } : 'assets/example.svg';
      expect(getSection(type)!.schema.safeParse(value).success).toBe(false);
    }
    if (rules.alignedRows) {
      const { rows, cells } = rules.alignedRows;
      value[rows][0][cells].push('extra cell');
      expect(getSection(type)!.schema.safeParse(value).success).toBe(false);
    }
  }
});
