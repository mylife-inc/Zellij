// A separate browser loader keeps server-only math imports out of this graph.
export async function loadMermaid() {
  return (await import('mermaid')).default;
}
