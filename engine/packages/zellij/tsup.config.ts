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
      'cli/context-bundle': 'src/cli/context.ts',
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

    /**
     * Bundled means bundled.
     *
     * tsup treats everything in `dependencies` as external, so these entries
     * came out importing `zod`, `yaml` and `gray-matter` from node_modules —
     * fine wherever the package is installed, and fatal in the base image,
     * which carries the engine at `/opt/zellij` and installs nothing. `zel
     * check` runs there with no `npm install`, which is the whole reason it
     * takes a second rather than a minute.
     *
     * React and Next stay external because nothing in this graph should reach
     * them; if one ever does, the failure is loud rather than a silently
     * enormous CLI.
     */
    noExternal: [/.*/],
    external: ['react', 'react-dom', 'next', /^next\//],

    /**
     * Some of what gets bundled is CommonJS that calls `require` at runtime —
     * `yaml` reaches for `process` that way. esbuild's ESM output has no
     * `require` in scope, so it emits a shim that throws "Dynamic require of
     * process is not supported", at the first parse of a YAML file rather than
     * at build time.
     *
     * Defining one from `import.meta.url` is what esbuild's own shim looks for.
     * The alternative — emitting CommonJS — would forbid top-level await in
     * the graph, and the MDX pipeline is one refactor away from wanting it.
     */
    banner: {
      js: "import { createRequire as __createRequire } from 'node:module';\nconst require = __createRequire(import.meta.url);",
    },

    /**
     * No shared chunks either. Splitting emits `dist/chunk-*.js` beside the
     * entries, so a file that reads as self-contained is not — and these are
     * copied into an image by path.
     */
    splitting: false,
  },
]);
