import { navigationHref, fileLocation } from './location';
// Components compare this identity with logical content routes, as Next does.
// Deployment prefixes belong on generated links, not the current route.
export function usePathname() { return fileLocation().route; }
const router = {
  push: (url: string) => { window.location.assign(navigationHref(url)); },
  replace: (url: string) => { window.location.replace(navigationHref(url)); },
  back: () => window.history.back(), forward: () => window.history.forward(),
  refresh: () => window.location.reload(), prefetch: () => {},
};
export function useRouter() { return router; }
export function redirect(to: string): never { throw new FileRedirect(to); }
export function notFound(): never { throw new Error('Offline route was not found'); }
export class FileRedirect extends Error { constructor(public to: string) { super(`Redirect to ${to}`); } }
