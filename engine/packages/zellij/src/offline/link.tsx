import { forwardRef, type AnchorHTMLAttributes } from 'react';
import { navigationHref } from './location';
type Props = Omit<AnchorHTMLAttributes<HTMLAnchorElement>, 'href'> & {
  href: string | { pathname?: string; query?: Record<string, string>; hash?: string };
  prefetch?: unknown; replace?: unknown; scroll?: unknown; legacyBehavior?: unknown;
};
const Link = forwardRef<HTMLAnchorElement, Props>(function FileLink({ href, prefetch: _prefetch,
  replace: _replace, scroll: _scroll, legacyBehavior: _legacy, ...props }, ref) {
  const url = typeof href === 'string' ? href : (href.pathname ?? '') +
    (href.query ? `?${new URLSearchParams(href.query)}` : '') + (href.hash ?? '');
  return <a {...props} href={navigationHref(url)} ref={ref} />;
});
export default Link;
