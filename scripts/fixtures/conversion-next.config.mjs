import { withZellij } from '@shebka/zellij/next-config';

// Must match the site.yaml prefix in check-conversion.mjs.
export default withZellij({ typedRoutes: false, basePath: '/products/docs' });
