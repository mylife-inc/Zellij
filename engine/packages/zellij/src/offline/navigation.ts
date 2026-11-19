import { fileHref, fileLocation } from './location';
export function usePathname() { const page = fileLocation(); return page.basePath + page.route; }
const router = {
  push: (url: string) => { window.location.assign(fileHref(url)); },
  replace: (url: string) => { window.location.replace(fileHref(url)); },
  back: () => window.history.back(), forward: () => window.history.forward(),
  refresh: () => window.location.reload(), prefetch: () => {},
};
export function useRouter() { return router; }
export function redirect(to: string): never { throw new FileRedirect(to); }
export function notFound(): never { throw new Error('Offline route was not found'); }
export class FileRedirect extends Error { constructor(public to: string) { super(`Redirect to ${to}`); } }
