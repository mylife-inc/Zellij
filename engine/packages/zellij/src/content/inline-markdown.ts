/** Shared with InlineMarkdown: emphasis recurses, code remains literal. */
export function inlineParts(text: string): string[] {
  const pattern = /((?:\*\*(?:[^*]|\*(?!\*))+\*\*)|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  return text.split(pattern).filter((part) => part !== '');
}

export function inlineLinks(text: string): string[] {
  return inlineParts(text).flatMap((part) => {
    if (part.startsWith('**') && part.endsWith('**')) return inlineLinks(part.slice(2, -2));
    if (part.startsWith('*') && part.endsWith('*')) return inlineLinks(part.slice(1, -1));
    if (part.startsWith('`') && part.endsWith('`')) return [];
    const link = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(part);
    return link ? [link[2]!] : [];
  });
}
