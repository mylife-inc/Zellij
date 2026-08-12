import { describe, expect, it, afterEach } from 'vitest';
// @ts-expect-error — plain JS, loaded by Node without types.
import { withZellij } from '../../next-config.mjs';

/**
 * What `withZellij` must do differently per output mode.
 *
 * These are cheap and they guard a class of bug the e2e suite structurally
 * cannot see: the tests run against `next start`, which is a server, and every
 * difference between a server and a static export is invisible to them.
 */
describe('withZellij', () => {
  const original = process.env['ZELLIJ_OUTPUT'];

  afterEach(() => {
    if (original === undefined) delete process.env['ZELLIJ_OUTPUT'];
    else process.env['ZELLIJ_OUTPUT'] = original;
  });

  const forOutput = (value?: string) => {
    if (value === undefined) delete process.env['ZELLIJ_OUTPUT'];
    else process.env['ZELLIJ_OUTPUT'] = value;
    return withZellij({});
  };

  /**
   * Regression: every image on the deployed documentation site 404'd.
   *
   * `next/image` emits `/_next/image?url=…&w=640&q=75`, which is a *server
   * route*. A static export has no server, so the route does not exist — while
   * the file sits at the path that URL was pointing at, served perfectly. The
   * page renders, the layout is correct, and the pictures are gone.
   *
   * Nothing failed: the build succeeded, the HTML was well-formed, and CI was
   * green. It was visible only to someone looking at the page.
   */
  it('does not ask for an image optimizer in a static export', () => {
    expect(forOutput('export').images.unoptimized).toBe(true);
  });

  /** A standalone build ships a server, so it keeps the optimizer. */
  it('keeps the optimizer for standalone and for dev', () => {
    expect(forOutput('standalone').images.unoptimized).toBeUndefined();
    expect(forOutput(undefined).images.unoptimized).toBeUndefined();
  });

  it('passes the output mode through', () => {
    expect(forOutput('export').output).toBe('export');
    expect(forOutput('standalone').output).toBe('standalone');
    expect(forOutput(undefined).output).toBeUndefined();
  });

  /** A site's own config still wins, including over the export default. */
  it('lets a site override what it sets', () => {
    process.env['ZELLIJ_OUTPUT'] = 'export';
    expect(withZellij({ images: { unoptimized: false } }).images.unoptimized).toBe(false);
  });

  /** Local assets carry a `?v=` fingerprint, which Next 16 rejects by default. */
  it('permits a query string on local images', () => {
    expect(forOutput('export').images.localPatterns).toEqual([{ pathname: '/**' }]);
  });
});
