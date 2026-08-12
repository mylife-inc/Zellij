/**
 * Next.js settings a Zellij site needs, in one place.
 *
 * These are the engine's requirements rather than the site's, so a wrapper app
 * should not have to know them: when a new one appears — as `mathjax-full` did
 * the day guides learned to typeset mathematics — every site that hand-copied
 * its config would otherwise break at once, with an error naming a package its
 * author never installed.
 *
 * Plain JavaScript, and importing nothing but Node builtins: Node loads
 * `next.config.js` itself, without Next's compiler, so this file cannot be
 * TypeScript and cannot reach into the engine's TypeScript source.
 */

import { networkInterfaces } from 'node:os';

/** @typedef {Record<string, unknown> & { transpilePackages?: string[], serverExternalPackages?: string[], images?: Record<string, unknown>, allowedDevOrigins?: string[] }} ZellijNextConfig */

/** @param {string[]} mine @param {string[] | undefined} theirs @returns {string[]} */
const union = (mine, theirs) => [...new Set([...mine, ...(theirs ?? [])])];

/**
 * Origins the dev server will answer `/_next/*` requests for.
 *
 * Next trusts `localhost` and the address it was told to bind — which for a dev
 * server reachable from anywhere is `0.0.0.0`, an address no browser ever sends.
 * So opening the site from a phone on the same network fails: the HMR socket and
 * the client chunks are refused, React never hydrates, and every entrance reveal
 * stays at `opacity: 0`. The page is *there*, and it is blank. Checking a docs
 * site on a real phone is a normal thing to do, so it should just work.
 *
 * Three sources, all narrow on purpose:
 *
 * - This machine's own addresses. Exactly right for `npm run dev`, and unable
 *   to over-permit — an attacker's page cannot present your LAN IP as its own
 *   origin.
 * - `*.local`, the mDNS name a phone gets from Bonjour. Reserved for link-local
 *   use and unregistrable, so it cannot be claimed by a public host.
 * - `ZELLIJ_DEV_ORIGINS`, comma-separated. Needed because `zel dev` runs the
 *   engine in a container, where the interfaces enumerated below are the
 *   container's and not the host's; `zel` passes the host's addresses in. Also
 *   the way to add a tunnel — ngrok, Tailscale, a Codespaces forward.
 *
 * Deliberately *not* a private-range pattern like `10.*.*.*`: Next matches
 * origins segment by segment, so that would also match `10.anything.example.com`
 * — a subdomain any attacker can create, which is the exact request this check
 * exists to refuse.
 *
 * @returns {string[]}
 */
function devOrigins() {
  const own = Object.values(networkInterfaces())
    .flatMap((addresses) => addresses ?? [])
    .filter((address) => !address.internal)
    .map((address) => address.address);

  const configured = (process.env['ZELLIJ_DEV_ORIGINS'] ?? '')
    .split(',')
    .map((origin) => origin.trim())
    .filter(Boolean);

  return [...new Set(['*.local', ...own, ...configured])];
}

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

      /*
       * Local images may carry a query string.
       *
       * Content assets are served under their author's filename, so `logo.png`
       * stays `logo.png` across a rebrand and no cache has any reason to fetch
       * it again. `zellij prepare` fingerprints them and the engine appends
       * `?v=<hash>`, which makes a changed image a changed URL.
       *
       * Next 16 refuses a local `src` with a search param unless a pattern
       * permits it, and a pattern with no `search` key permits any. `/**` is
       * every local path, which is what was allowed before this key existed —
       * the only thing widened is the query string.
       */
      localPatterns: [{ pathname: '/**' }],

      /*
       * A static export has no image optimizer, so it must not ask for one.
       *
       * `next/image` normally emits `/_next/image?url=…&w=640&q=75`, which is a
       * *server route*. Export the site and that route does not exist: every
       * image 404s while the PNG sits beside it, served fine, at the path the
       * optimizer URL was pointing at. The page renders, the layout is right,
       * and the pictures are missing — which reads as a broken build rather
       * than as a configuration mismatch.
       *
       * Driven by the same variable as `output` below, so the two cannot
       * disagree. A `standalone` build keeps the optimizer, because it ships a
       * server that can run it.
       *
       * Found on the first public deploy of Zellij's own documentation, where
       * `/_next/image?url=…card-guides.png` returned 404 and
       * `/_zellij/assets/card-guides.png` returned 200.
       */
      ...(process.env['ZELLIJ_OUTPUT'] === 'export' ? { unoptimized: true } : {}),

      ...config.images,
    },

    // Ignored outside `next dev`, so there is no production cost to setting it.
    allowedDevOrigins: union(devOrigins(), config.allowedDevOrigins),

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
