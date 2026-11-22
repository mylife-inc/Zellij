import { describe, it, expect } from 'vitest';
import { validateOfflineRoutes, validateHttpRoutes } from './validate-routes';

describe('portable rich output routes', () => {
  it('accepts nested routes, Unicode and spaces without decoding filesystem paths', () => {
    expect(() => validateOfflineRoutes(['/', '/guide', '/guide/日本', '/hello world'])).not.toThrow();
  });
  it('rejects traversal, URL ambiguity and Windows aliases before writing pages', () => {
    for (const route of ['/../outside', '/a/./b', '//host', '/a\\b', '/a%2fb', '/a?b', '/a#b',
      '/a:b', '/AUX.txt', '/a/LPT1', '/a.', '/a ', '/a\0b', '/a/']) {
      expect(() => validateOfflineRoutes([route]), route).toThrow(/offline route/);
    }
  });
  it('rejects case and Unicode-normalization output collisions', () => {
    for (const routes of [['/guide', '/Guide'], ['/café', '/cafe\u0301']]) {
      expect(() => validateOfflineRoutes(routes)).toThrow(/collide/);
    }
  });
});

describe('flat HTTP output routes', () => {
  it('accepts flat/nested and encoded-in-URL Unicode destinations', () => {
    expect(() => validateHttpRoutes(['/', '/guide', '/guide/日本', '/café'])).not.toThrow();
  });
  it('rejects reserved files and file/directory output collisions', () => {
    for (const routes of [['/', '/index'], ['/404'], ['/_zellij/static/runtime'], ['/guide', '/guide.html/child'], ['/GUIDE', '/guide.HTML/child']]) expect(() => validateHttpRoutes(routes)).toThrow(/colli/);
  });
});
