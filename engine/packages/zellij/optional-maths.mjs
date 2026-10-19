// Optional peers remain absent from disabled wrappers.
export async function loadMathLibraries() {
  return Promise.all([import('remark-math'), import('rehype-mathjax')]);
}
