/** Cross-field authoring rules that Zod's JSON Schema cannot express. Keep the
 * engine validator, generated starters and native authoring on the same data. */
export type AuthoringRules = {
  exactlyOne?: string[];
  boundedTree?: { children: string; maxNodes: number; maxPrimaryHeadings: number };
  alignedRows?: { columns: string; rows: string; cells: string };
};
export const sectionAuthoringRules: Record<string, AuthoringRules> = {
  compose: { boundedTree: { children: 'children', maxNodes: 128, maxPrimaryHeadings: 1 } },
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
  if (rules?.boundedTree) {
    const rule = rules.boundedTree;
    const pending = [...section[rule.children] as Record<string, unknown>[]];
    let count = 0, headings = 0;
    while (pending.length) {
      const node = pending.pop()!;
      if (++count > rule.maxNodes) return false;
      if (node.type === 'heading' && node.level === 1 && ++headings > rule.maxPrimaryHeadings) return false;
      if (Array.isArray(node[rule.children])) pending.push(...node[rule.children] as Record<string, unknown>[]);
    }
  }
  return true;
}
