import { defineConfig } from 'tsup';

/**
 * Publishing build (spec §13.2).
 *
 * Inside this monorepo the demo consumes `src` directly through Next's
 * `transpilePackages`, so this build is not part of the development loop — it
 * exists to produce the artifact npm consumers install.
 *
 * `bundle: false` is the load-bearing choice. This is a React Server Components
 * library, where `"use client"` marks the server/client boundary per module.
 * Bundling merges modules into shared chunks, which either loses the directive
 * or forces it onto a chunk that also contains server-only code — and marking
 * the main entry `"use client"` would drag the filesystem content loader into
 * the browser. Emitting the module graph 1:1 keeps every boundary exactly where
 * the source put it, and consumers' bundlers tree-shake anyway.
 */
export default defineConfig({
  entry: ['src/**/*.ts', 'src/**/*.tsx', '!src/**/*.test.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  sourcemap: true,
  clean: true,
  bundle: false,
  outDir: 'dist',
  target: 'es2022',
  external: ['react', 'react-dom', 'next', /^next\//],
  esbuildOptions(options) {
    options.jsx = 'automatic';
  },
});
