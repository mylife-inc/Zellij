import { describe, expect, it } from 'vitest';
import { fileHref, setFileLocation, assetProps } from './location';
import { jsonForScript } from './decode';
const location = { route: '/guide/install', basePath: '/product', routes: ['/', '/guide/install', '/café'] };
describe('file export URLs', () => {
  it('keeps file links relocatable, encoded and independent of deployment base paths', () => {
    expect(fileHref('/product', location)).toBe('../../index.html');
    expect(fileHref('/product/guide/install?mode=1#step', location)).toBe('../../guide/install/index.html?mode=1#step');
    expect(fileHref('/caf%C3%A9', location)).toBe('../../caf%C3%A9/index.html');
    expect(fileHref('/product/_zellij/assets/a%20b.pdf?v=abc#page=1', location)).toBe('../../_zellij/assets/a%20b.pdf?v=abc#page=1');
    for (const url of ['#part', 'https://example.com/x', '//example.com/x', 'mailto:help@example.com', 'relative.pdf']) expect(fileHref(url, location)).toBe(url);
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
