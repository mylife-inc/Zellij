import { describe, expect, it } from 'vitest';
import { fileHref, navigationHref, setFileLocation, assetProps } from './location';
import { jsonForScript } from './decode';
import { usePathname } from './navigation';
const location = { route: '/guide/install', basePath: '/product', routes: ['/', '/guide/install', '/café'] };
describe('file export URLs', () => {
  it.each(['file', 'static'] as const)('keeps %s current-page identity independent of the deployment prefix', target => {
    for (const route of ['/', '/guide/install', '/café']) {
      setFileLocation({ ...location, route, basePath: '/products/manual', target });
      expect(usePathname()).toBe(route);
    }
  });
  it('keeps file links relocatable, encoded and independent of deployment base paths', () => {
    expect(fileHref('/product', location)).toBe('../../index.html');
    expect(fileHref('/product/guide/install?mode=1#step', location)).toBe('../../guide/install/index.html?mode=1#step');
    expect(fileHref('/caf%C3%A9', location)).toBe('../../caf%C3%A9/index.html');
    expect(fileHref('/product/_zellij/assets/a%20b.pdf?v=abc#page=1', location)).toBe('../../_zellij/assets/a%20b.pdf?v=abc#page=1');
    for (const url of ['#part', 'https://example.com/x', '//example.com/x', 'mailto:help@example.com', 'relative.pdf']) expect(fileHref(url, location)).toBe(url);
  });
  it('preserves hosted route and asset URLs, including prefix, query and fragment', () => {
    const hosted = { ...location, target: 'static' as const };
    for (const url of ['/product/guide/install?mode=1#step', '/product/_zellij/assets/a%20b.pdf?v=x', '/caf%C3%A9', '#part', 'https://example.com/x']) expect(fileHref(url, hosted)).toBe(url);
  });
  it('applies the hosted prefix to router and Link destinations once', () => {
    const hosted = { ...location, target: 'static' as const };
    expect(navigationHref('/guide/install?mode=1#step', hosted)).toBe('/product/guide/install?mode=1#step');
    expect(navigationHref('/productivity', hosted)).toBe('/product/productivity');
    for (const url of ['/product', '/product?x=1', '/product#top', '/product/café', '#part', 'https://example.com', '//example.com', '../relative']) expect(navigationHref(url, hosted)).toBe(url);
  });
  it('rejects encoded traversal and Windows separators', () => {
    for (const url of ['/../secret', '/%2e%2e/secret', '/a/./secret', '/a%5csecret']) expect(() => fileHref(url, location)).toThrow('Unsafe');
  });
  it('rewrites asset props and background URLs without rewriting logical navigation routes', () => {
    setFileLocation(location);
    expect(assetProps({ src: '/product/_zellij/assets/a.png', href: '/guide/install',
      style: { backgroundImage: 'url("/product/_zellij/assets/b.png")' } })).toEqual({
      src: '../../_zellij/assets/a.png', href: '/guide/install', style: { backgroundImage: 'url("../../_zellij/assets/b.png")' } });
  });
  it('cannot end its inert page-data script with authored text', () => {
    const input = { text: '</script><script>alert(1)</script>\u2028\u2029' };
    const json = jsonForScript(input);
    expect(json).not.toContain('<');
    expect(JSON.parse(json)).toEqual(input);
  });
});
