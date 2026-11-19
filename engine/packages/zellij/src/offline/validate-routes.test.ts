import { describe, it, expect } from 'vitest';
import { validateOfflineRoutes } from './validate-routes';

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
