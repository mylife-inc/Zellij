import { defineConfig } from 'vitest/config';

/**
 * The engine's tests, and one setting they need.
 *
 * `tsconfig.base.json` sets `jsx: preserve`, which is right for the build —
 * Next.js wants the JSX handed to it untransformed. Vitest does not go through
 * Next: it transforms with esbuild, and esbuild's default for JSX is the
 * classic runtime, which compiles `<p/>` to `React.createElement` and expects
 * `React` to be a global. Nothing imports it that way in this codebase, so any
 * test that renders a component fails with "React is not defined" — in the
 * component, not in the test.
 *
 * `automatic` compiles to `jsx()` from `react/jsx-runtime` instead, which is
 * imported rather than assumed, and matches what Next emits in the real build.
 */
export default defineConfig({
  test: { dir: 'src' },
  esbuild: { jsx: 'automatic' },
});
