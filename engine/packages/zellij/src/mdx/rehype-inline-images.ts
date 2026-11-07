import type { Root, RootContent } from 'hast';

const textContainers = new Set(['p', 'li', 'td', 'th', 'caption', 'dt', 'dd', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6']);
const inlineElements = new Set(['a', 'em', 'strong', 'del', 'code', 'span', 'sub', 'sup', 'b', 'i', 's', 'u', 'small', 'mark', 'abbr', 'cite', 'q', 'kbd', 'samp', 'var', 'time', 'br', 'img']);

/** Images embedded in prose retain inline flow; standalone figures stay blocks. */
export function rehypeInlineImages() {
  return (tree: Root) => {
    function hasText(node: RootContent): boolean {
      if (node.type === 'text') return node.value.trim().length > 0;
      // Text in a nested block does not turn a standalone figure into an icon.
      return node.type === 'element' && inlineElements.has(node.tagName) && node.children.some(hasText);
    }
    function visit(node: Root | RootContent, inline = false): void {
      if (node.type === 'element') {
        if (textContainers.has(node.tagName)) inline = node.children.some(hasText);
        else if (!inlineElements.has(node.tagName)) inline = false;
      }
      if (inline && node.type === 'element' && node.tagName === 'img') {
        const classes = node.properties.className;
        node.properties.className = [...(Array.isArray(classes) ? classes : classes ? [String(classes)] : []), 'zellij-inline-image'];
      }
      if ('children' in node) for (const child of node.children) visit(child, inline);
    }
    visit(tree);
  };
}
