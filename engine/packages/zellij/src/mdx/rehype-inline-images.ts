import type { Root, RootContent } from 'hast';

/** Images embedded in prose retain inline flow; standalone figures stay blocks. */
export function rehypeInlineImages() {
  return (tree: Root) => {
    function hasText(node: Root | RootContent): boolean {
      if (node.type === 'text') return node.value.trim().length > 0;
      return 'children' in node && node.children.some(hasText);
    }
    function visit(node: Root | RootContent, inline = false): void {
      if (node.type === 'element' && node.tagName === 'p') inline = hasText(node);
      if (inline && node.type === 'element' && node.tagName === 'img') {
        const classes = node.properties.className;
        node.properties.className = [...(Array.isArray(classes) ? classes : classes ? [String(classes)] : []), 'zellij-inline-image'];
      }
      if ('children' in node) for (const child of node.children) visit(child, inline);
    }
    visit(tree);
  };
}
