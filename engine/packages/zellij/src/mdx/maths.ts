import type { Pluggable } from 'unified';
import { featureEnabled } from '../../feature-config.cjs';
import { loadMathLibraries } from '@shebka/zellij/optional-maths';

/** Optional math peers are loaded only when selected. Legacy wrappers without
 * selection metadata retain automatic availability-based rendering. */
let maths: Pluggable[][] | undefined;

/**
 * Resolved once and cached. Awaited rather than resolved at module load
 * because the package is built for CommonJS as well as ESM, and top-level
 * await cannot be expressed in CommonJS at all.
 */
export async function loadMaths(): Promise<Pluggable[][]> {
  if (!featureEnabled('maths')) return [[], []];
  if (maths) return maths;
  maths = await resolveMaths();
  return maths;
}

async function resolveMaths(): Promise<Pluggable[][]> {
  try {
    const [{ default: remarkMath }, { default: rehypeMathjax }] = await loadMathLibraries();

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
