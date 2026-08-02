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
export default defineConfig([
  {
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
  },
  /**
   * The CLI, bundled.
   *
   * Node's ESM resolver requires file extensions on relative imports and the
   * source does not carry them, so the unbundled build above cannot be executed
   * directly — only compiled by a bundler, which is what a consuming app does
   * with everything else in the package.
   *
   * `bin/zellij.mjs` runs from source inside this monorepo, where the files sit
   * outside `node_modules` and Node will strip their types. Once the package is
   * installed as a dependency that stops working, so these are what actually
   * run for a consumer. Bundling is correct here for the same reason it is
   * wrong above: nothing in this graph is a React Server Component.
   */
  {
    entry: {
      'cli/check-bundle': 'src/cli/check.ts',
      'cli/init-bundle': 'src/cli/init.ts',
      'cli/sync-assets-bundle': 'src/cli/sync-assets.ts',
      'cli/write-search-index-bundle': 'src/cli/write-search-index.ts',
    },
    format: ['esm'],
    dts: false,
    sourcemap: false,
    clean: false,
    bundle: true,
    outDir: 'dist',
    target: 'es2022',
    platform: 'node',
  },
]);
