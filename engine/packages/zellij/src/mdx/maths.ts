import type { Pluggable } from 'unified';

/**
 * Mathematics is optional, and costs about fifty megabytes to install.
 *
 * A documentation site with no formulas in it should not carry MathJax and its
 * speech-rule engine around, so the packages are optional dependencies and this
 * is where their absence is handled. Installed, `$…$` is typeset. Not installed,
 * it stays as written — which is what a site that never wanted maths expects
 * anyway, and is a great deal better than failing to build over a package
 * nobody asked for.
 *
 * Resolved once at module load rather than per page: the answer cannot change
 * while the process is running, and a `try` around every compile would pay for
 * asking a question that has one answer.
 */
let maths: Pluggable[][] | undefined;

/**
 * Resolved once and cached. Awaited rather than resolved at module load
 * because the package is built for CommonJS as well as ESM, and top-level
 * await cannot be expressed in CommonJS at all.
 */
export async function loadMaths(): Promise<Pluggable[][]> {
  if (maths) return maths;
  maths = await resolveMaths();
  return maths;
}

async function resolveMaths(): Promise<Pluggable[][]> {
  try {
    const [{ default: remarkMath }, { default: rehypeMathjax }] = await Promise.all([
      import('remark-math'),
      import('rehype-mathjax'),
    ]);

    return [
      [remarkMath as Pluggable],
      // Display maths is left-aligned to sit with the prose rather than centred
      // over it. Width is handled in CSS.
      [[rehypeMathjax, { svg: { displayAlign: 'left' } }] as Pluggable],
    ];
  } catch {
    return [[], []];
  }
}
