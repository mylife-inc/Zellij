import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
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


const roots: string[] = [];
afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

async function wrapper(files: Record<string, string>) {
  const appRoot = await mkdtemp(path.join(tmpdir(), 'zellij-prefix-'));
  roots.push(appRoot);
  for (const [name, value] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(appRoot, name)), { recursive: true });
    await writeFile(path.join(appRoot, name), value);
  }
  return appRoot;
}

describe('deployment prefix', () => {
  it('derives Next and rendering prefixes from content without editing Next config', async () => {
    const appRoot = await wrapper({ 'content/site.yaml': 'basePath: /products/docs/\n' });
    const config = withZellij({}, { appRoot });
    expect(config.basePath).toBe('/products/docs');
    expect(config.env.ZELLIJ_BASE_PATH).toBe('/products/docs');
  });

  it('uses the same explicit root and legacy configuration paths as preparation', async () => {
    const appRoot = await wrapper({
      'external/site.yaml': 'basePath: /external\n',
      'legacy.yaml': 'product: { name: Old }\ncontent: external\n',
    });
    expect(withZellij({}, { appRoot, contentDir: 'external' }).basePath).toBe('/external');
    expect(withZellij({}, { appRoot, configFile: 'legacy.yaml' }).basePath).toBe('/external');
  });

  it('reads the legacy prefix when marketing layout is selected', async () => {
    const appRoot = await wrapper({
      'zellij.config.yaml': 'product: { name: Old }\nbasePath: /old\n',
      'content/marketing/index.yaml': 'title: Old\n',
    });
    expect(withZellij({}, { appRoot }).basePath).toBe('/old');
  });

  it('accepts a wrapper prefix when content does not set one and retains other env entries', async () => {
    const appRoot = await wrapper({ 'content/site.yaml': 'name: Test\n' });
    const config = withZellij({ basePath: '/wrapper', env: { CUSTOM: 'kept' } }, { appRoot });
    expect(config.basePath).toBe('/wrapper');
    expect(config.env).toEqual({ CUSTOM: 'kept', ZELLIJ_BASE_PATH: '/wrapper' });
  });

  it('rejects contradictory explicit prefixes, including an explicit empty prefix', async () => {
    const appRoot = await wrapper({ 'content/site.yaml': 'basePath: /content\n' });
    expect(() => withZellij({ basePath: '/wrapper' }, { appRoot })).toThrow(/basePath.*disagree/i);
    expect(() => withZellij({ basePath: '' }, { appRoot })).toThrow(/basePath.*disagree/i);
    expect(withZellij({ basePath: '/content' }, { appRoot }).basePath).toBe('/content');
  });

  it.each(['/a/../b', '//host/path', '/a?b', '/a#b', '/a\\b', '/a//b', '/a%2fb'])('rejects an invalid prefix: %s', async (basePath) => {
    const appRoot = await wrapper({});
    expect(() => withZellij({ basePath }, { appRoot })).toThrow(/basePath/);
  });
});
