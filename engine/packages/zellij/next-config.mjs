/**
 * Next.js settings a Zellij site needs, in one place.
 *
 * These are the engine's requirements rather than the site's, so a wrapper app
 * should not have to know them: when a new one appears — as `mathjax-full` did
 * the day guides learned to typeset mathematics — every site that hand-copied
 * its config would otherwise break at once, with an error naming a package its
 * author never installed.
 *
 * Plain JavaScript, and deliberately importing nothing: Node loads
 * `next.config.js` itself, without Next's compiler, so this file cannot be
 * TypeScript and cannot reach into the engine's TypeScript source.
 */

/** @typedef {Record<string, unknown> & { transpilePackages?: string[], serverExternalPackages?: string[], images?: Record<string, unknown> }} ZellijNextConfig */

/** @param {string[]} mine @param {string[] | undefined} theirs @returns {string[]} */
const union = (mine, theirs) => [...new Set([...mine, ...(theirs ?? [])])];

/**
 * Merges the engine's requirements into a site's own Next config.
 *
 * The site wins on every key it sets. Arrays are unioned rather than replaced,
 * so adding a package of your own to `serverExternalPackages` does not silently
 * drop the engine's.
 *
 * @param {ZellijNextConfig} [config]
 * @returns {ZellijNextConfig}
 */
export function withZellij(config = {}) {
  return {
    ...config,

    // Zellij ships TypeScript source alongside its build output. In a monorepo
    // the source is what resolves, and Next has to compile it.
    transpilePackages: union(['@shebka/zellij'], config.transpilePackages),

    // MathJax resolves its own component files at runtime, relative to its
    // package directory. Bundled into the server output those paths no longer
    // exist, and the build fails looking for a package.json inside `.next`.
    serverExternalPackages: union(['mathjax-full'], config.serverExternalPackages),

    images: {
      formats: ['image/avif', 'image/webp'],
      ...config.images,
    },

    /*
     * `standalone` traces the server and its dependencies into one directory,
     * which is what lets a container ship the built site without `node_modules`
     * — the difference between a runtime image of tens of megabytes and one of
     * several hundred.
     *
     * Driven by the environment because the same content folder is built both
     * ways: locally by an author, and in a container by CI. Neither should have
     * to keep a second config around to say so.
     */
    ...(process.env['ZELLIJ_OUTPUT'] ? { output: process.env['ZELLIJ_OUTPUT'] } : {}),
  };
}

export default withZellij;
