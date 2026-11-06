/** Cross-field authoring rules that Zod's JSON Schema cannot express. Keep the
 * engine validator, generated starters and native authoring on the same data. */
export type AuthoringRules = {
  exactlyOne?: string[];
  alignedRows?: { columns: string; rows: string; cells: string };
};
export const sectionAuthoringRules: Record<string, AuthoringRules> = {
  'hero-split': { exactlyOne: ['visual', 'terminal'] },
  'steps-panel': { exactlyOne: ['visual', 'terminal', 'diagram'] },
  'media-panel': { exactlyOne: ['image', 'video'] },
  matrix: { alignedRows: { columns: 'columns', rows: 'rows', cells: 'cells' } },
};

export function satisfiesAuthoringRules(type: string, section: Record<string, unknown>): boolean {
  const rules = sectionAuthoringRules[type];
  if (rules?.exactlyOne && rules.exactlyOne.filter((key) => Boolean(section[key])).length !== 1) return false;
  if (rules?.alignedRows) {
    const { columns, rows, cells } = rules.alignedRows;
    const count = (section[columns] as unknown[]).length;
    return (section[rows] as Record<string, unknown>[]).every((row) => (row[cells] as unknown[]).length === count);
  }
  return true;
}
