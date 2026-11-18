/** Cross-field authoring rules that Zod's JSON Schema cannot express. Keep the
 * engine validator, generated starters and native authoring on the same data. */
export type AuthoringRules = {
  composeConstraints?: boolean;
  exactlyOne?: string[];
  boundedTree?: { children: string; maxNodes: number; maxPrimaryHeadings: number };
  alignedRows?: { columns: string; rows: string; cells: string };
};
export const sectionAuthoringRules: Record<string, AuthoringRules> = {
  compose: { composeConstraints: true, boundedTree: { children: 'children', maxNodes: 128, maxPrimaryHeadings: 1 } },
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
  if (rules?.composeConstraints && !validComposition(section)) return false;
  return true;
}

/** Cross-field rules are also enforced by native section authoring. */
function validComposition(node: Record<string, unknown>, insideStage = false, insideLayers = false): boolean {
  const scroll = node.scroll as Record<string, number> | undefined;
  if (scroll && (insideStage || (scroll.start ?? 0) >= (scroll.end ?? 1))) return false;
  if (node.type === 'story') {
    if (insideStage || insideLayers) return false;
    if (!(node.scenes as {media: {kind: string; start?: number; end?: number}}[]).every(scene => scene.media.kind !== 'video' || (scene.media.start ?? 0) < scene.media.end!)) return false;
  }
  if (node.type === 'selector' || node.type === 'explorer') {
    const items = node.items as {id: string; label: string; swatch?: string}[];
    if (new Set(items.map(item => item.id)).size !== items.length || new Set(items.map(item => item.label)).size !== items.length) return false;
    if (node.initial !== undefined && !items.some(item => item.id === node.initial)) return false;
    if (node.type === 'selector' && node.presentation === 'swatches' && items.some(item => !item.swatch)) return false;
  }
  if (node.type === 'table') {
    const columns = node.columns as string[];
    if (new Set(columns).size !== columns.length || !(node.rows as {cells: unknown[]}[]).every(row => row.cells.length === columns.length)) return false;
  }
  const children = node.children as Record<string, unknown>[] | undefined;
  if (!children) return true;
  const layers = (node.layout as {kind?: string} | undefined)?.kind === 'layers';
  const places = new Set<string>();
  for (const child of children) {
    if (layers) {
      const place = String(child.place ?? 'center');
      if (places.has(place)) return false;
      places.add(place);
    } else if (child.place !== undefined) return false;
    if (!validComposition(child, insideStage || Boolean(scroll), insideLayers || layers)) return false;
  }
  return true;
}
