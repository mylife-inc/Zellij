import { hierarchy, tree, type HierarchyPointNode } from 'd3-hierarchy';
import { linkHorizontal } from 'd3-shape';
import { parse as parseYaml } from 'yaml';

/**
 * Mindmaps, laid out during the build and shipped as SVG.
 *
 * Mermaid draws diagrams in the browser because it has to measure text to place
 * anything. A mindmap does not need that precision — the boxes are labels, not
 * paragraphs — so estimating the width of a string is enough, and estimating it
 * here means the whole diagram is finished before the page is served: no
 * library reaches the browser, nothing renders after paint, and the colours are
 * CSS variables, so the drawing follows a theme change the way text does rather
 * than needing to be drawn again.
 *
 * The tree runs left to right, which is what makes a mindmap readable at a
 * glance: every branch starts at the same edge and the eye follows one line.
 */

/** A node before layout. */
interface Node {
  label: string;
  children: Node[];
}

/**
 * Reads the YAML shape.
 *
 * A mapping is a node and its children; a list is a set of siblings; a bare
 * string is a leaf. That covers what people actually draw without inventing a
 * vocabulary they would have to learn:
 *
 * ```yaml
 * Zellij:
 *   Content: [site.yaml, menu.yaml]
 *   Output: Static HTML
 * ```
 */
function toNode(label: string, value: unknown): Node {
  if (value === null || value === undefined) {
    return { label, children: [] };
  }

  if (Array.isArray(value)) {
    return {
      label,
      children: value.map((item) =>
        item !== null && typeof item === 'object'
          ? childrenOf(item as Record<string, unknown>)
          : { label: String(item), children: [] },
      ).flat(),
    };
  }

  if (typeof value === 'object') {
    return { label, children: childrenOf(value as Record<string, unknown>) };
  }

  return { label, children: [{ label: String(value), children: [] }] };
}

function childrenOf(value: Record<string, unknown>): Node[] {
  return Object.entries(value).map(([key, child]) => toNode(key, child));
}

export function parseMindmap(source: string): Node {
  const parsed = parseYaml(source) as unknown;

  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('A mindmap starts with a single root, written as one YAML key.');
  }

  const entries = Object.entries(parsed as Record<string, unknown>);
  if (entries.length === 0) {
    throw new Error('A mindmap needs a root.');
  }
  if (entries.length > 1) {
    throw new Error(
      `A mindmap has one root; this has ${entries.length} (${entries
        .map(([key]) => key)
        .join(', ')}). Nest them under a single key.`,
    );
  }

  const [label, value] = entries[0]!;
  return toNode(label, value);
}

/*
 * Geometry. Every number here is in the SVG's own units, and the whole drawing
 * is scaled to its container by the viewBox — so these are proportions rather
 * than pixels and do not need to match any particular font size.
 */
const CHAR_WIDTH = 7.15;
const PADDING_X = 13;
const BOX_HEIGHT = 27;
const ROW_GAP = 36;
const COLUMN_GAP = 46;

/** Estimated width of a label's box. Close enough for a word or two. */
function boxWidth(label: string): number {
  return Math.round(label.length * CHAR_WIDTH + PADDING_X * 2);
}

/** How many colours the branch palette cycles through. */
const BRANCH_COLOURS = 6;

/**
 * The branch a node belongs to, so a whole limb shares one colour.
 *
 * Taken from the ancestor directly below the root: colour is what tells one
 * limb from another, and colouring by depth instead would stripe the diagram
 * into rings that say nothing about its structure.
 */
function branchOf(node: HierarchyPointNode<Node>): number {
  let current = node;
  while (current.parent && current.parent.parent) current = current.parent;

  const siblings = current.parent?.children ?? [];
  const index = siblings.indexOf(current);
  return index < 0 ? 0 : index % BRANCH_COLOURS;
}

/** A hast element, built without importing a builder for four node types. */
type El = {
  type: 'element';
  tagName: string;
  properties: Record<string, string | number | string[]>;
  children: El[] | { type: 'text'; value: string }[];
};

const el = (
  tagName: string,
  properties: Record<string, string | number | string[]>,
  children: unknown[] = [],
): El => ({ type: 'element', tagName, properties, children: children as El[] });

/**
 * Lays the tree out and returns the finished SVG as hast.
 *
 * Hast rather than a string of markup: MDX compiles to JSX, where a raw string
 * is text to be escaped rather than markup to be rendered. Building the nodes
 * means the diagram survives the same pipeline everything else goes through.
 */
export function renderMindmap(source: string): unknown {
  const root = hierarchy(parseMindmap(source));

  // The widest label at each depth decides that column's width, so a long name
  // pushes its own children right instead of overlapping them.
  const widthByDepth = new Map<number, number>();
  root.each((node) => {
    const width = boxWidth(node.data.label);
    widthByDepth.set(node.depth, Math.max(widthByDepth.get(node.depth) ?? 0, width));
  });

  const offsets: number[] = [];
  let running = 0;
  for (let depth = 0; depth <= (root.height ?? 0); depth += 1) {
    offsets[depth] = running;
    running += (widthByDepth.get(depth) ?? 0) + COLUMN_GAP;
  }

  const positioned = tree<Node>().nodeSize([ROW_GAP, 0])(root);

  // d3 lays a tree out top-down; a mindmap reads left to right, so depth
  // becomes the horizontal axis and d3's spread becomes vertical.
  positioned.each((node) => {
    node.y = offsets[node.depth] ?? 0;
  });

  const nodes = positioned.descendants();
  const minX = Math.min(...nodes.map((node) => node.x));
  const maxX = Math.max(...nodes.map((node) => node.x));
  const maxY = Math.max(...nodes.map((node) => node.y + boxWidth(node.data.label)));

  const margin = BOX_HEIGHT;
  const shift = (x: number) => x - minX + BOX_HEIGHT / 2 + margin;
  const width = Math.round(maxY + 8);
  const height = Math.round(maxX - minX + BOX_HEIGHT + margin * 2);

  const link = linkHorizontal<unknown, { x: number; y: number }>()
    .x((node) => node.y)
    .y((node) => node.x);

  const edges = positioned.links().map(({ source, target }) =>
    el('path', {
      className: ['zellij-mindmap-edge'],
      'data-branch': String(branchOf(target)),
      // From the right edge of the parent to the left edge of the child, so a
      // link never runs through the box it starts in.
      d:
        link({
          source: { x: shift(source.x), y: source.y + boxWidth(source.data.label) },
          target: { x: shift(target.x), y: target.y },
        }) ?? '',
    }),
  );

  const boxes = nodes.map((node) => {
    const w = boxWidth(node.data.label);
    const x = node.y;
    const y = shift(node.x) - BOX_HEIGHT / 2;

    return el(
      'g',
      {
        className: ['zellij-mindmap-node'],
        'data-branch': node.depth === 0 ? 'root' : String(branchOf(node)),
        'data-depth': String(node.depth),
      },
      [
        el('rect', { x, y, width: w, height: BOX_HEIGHT, rx: 7 }),
        el(
          'text',
          {
            x: x + w / 2,
            y: y + BOX_HEIGHT / 2,
            'dominant-baseline': 'central',
            'text-anchor': 'middle',
          },
          [{ type: 'text', value: node.data.label }],
        ),
      ],
    );
  });

  return el(
    'svg',
    {
      className: ['zellij-mindmap-svg'],
      viewBox: `0 0 ${width} ${height}`,
      role: 'img',
      xmlns: 'http://www.w3.org/2000/svg',
    },
    [
      el('g', { className: ['zellij-mindmap-edges'] }, edges),
      el('g', { className: ['zellij-mindmap-nodes'] }, boxes),
    ],
  );
}
