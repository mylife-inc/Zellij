/** @type {import('next').NextConfig} */
const nextConfig = {
  // Zellij ships TypeScript source in the monorepo; Turbopack compiles it
  // directly, so there is no package build step during development (spec §13.2).
  transpilePackages: ['@shebka/zellij'],

  // Uncomment to verify reverse-proxied deployments (spec §11.5).
  // Keep in sync with `basePath` in zellij.config.yaml.
  // basePath: '/products/acme/docs',

  images: {
    // Content assets are synced into public/_zellij/assets and served locally,
    // so no remote patterns are needed in v1.
    formats: ['image/avif', 'image/webp'],
  },

  typedRoutes: false,
};

export default nextConfig;
