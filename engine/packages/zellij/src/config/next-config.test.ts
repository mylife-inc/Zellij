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
  it('uses a versioned wrapper selector without treating it as legacy presentation', async () => {
    const appRoot = await wrapper({
      'zellij.config.yaml': 'version: 1\ncontent: external\n',
      'external/site.yaml': 'basePath: /external\n',
    });
    expect(withZellij({}, { appRoot }).basePath).toBe('/external');
  });

  it('refuses a versioned selector for the legacy marketing layout', async () => {
    const appRoot = await wrapper({
      'zellij.config.yaml': 'version: 1\ncontent: external\n',
      'external/marketing/index.yaml': 'title: Old\n',
    });
    expect(() => withZellij({}, { appRoot })).toThrow(/current hosted folder layout/);
  });

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
    expect(config.env).toMatchObject({ CUSTOM: 'kept', ZELLIJ_BASE_PATH: '/wrapper' });
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

describe('explicit feature selection', () => {
  it('disables packages without removing unrelated aliases, env or webpack hooks', async () => {
    const appRoot = await wrapper({ 'package.json': JSON.stringify({ zellij: { features: [] } }) });
    const config = withZellij({
      env: { CUSTOM: 'retained' },
      turbopack: { resolveAlias: { custom: './mine.js', mermaid: './independent-mermaid.js' } },
      webpack: (value: { marker?: boolean }) => ({ ...value, marker: true }),
    }, { appRoot });
    expect(config.env).toMatchObject({ ZELLIJ_MATHS: 'false', NEXT_PUBLIC_ZELLIJ_DIAGRAMS: 'false', CUSTOM: 'retained' });
    expect(config.turbopack.resolveAlias.custom).toBe('./mine.js');
    expect(config.turbopack.resolveAlias.mermaid).toBe('./independent-mermaid.js');
    expect(config.turbopack.resolveAlias['@shebka/zellij/optional-mermaid']).toBe('@shebka/zellij/disabled-feature');
    const webpack = config.webpack({ resolve: { alias: { existing: './kept' } } }, {});
    expect(webpack.marker).toBe(true);
    expect(webpack.resolve.alias.existing).toBe('./kept');
    expect(webpack.resolve.alias['@shebka/zellij/optional-maths']).toBe('@shebka/zellij/disabled-feature');
  });

  it('preserves array-form webpack aliases for independent users of optional libraries', async () => {
    const appRoot = await wrapper({ 'package.json': JSON.stringify({ zellij: { features: [] } }) });
    const config = withZellij({}, { appRoot });
    const result = config.webpack({ resolve: { alias: [
      { name: 'mermaid', alias: './independent-mermaid.js', onlyModule: true },
      { name: '@shebka/zellij/optional-mermaid', alias: './old-loader.js' },
    ] } }, {});
    expect(result.resolve.alias).toEqual([
      { name: '@shebka/zellij/optional-maths', alias: '@shebka/zellij/disabled-feature', onlyModule: true },
      { name: '@shebka/zellij/optional-mermaid', alias: '@shebka/zellij/disabled-feature', onlyModule: true },
      { name: 'mermaid', alias: './independent-mermaid.js', onlyModule: true },
    ]);
  });

  it.each([['maths'], ['diagrams'], ['maths', 'diagrams']])('requires installed packages when explicitly selecting %j', async (...features) => {
    const appRoot = await wrapper({ 'package.json': JSON.stringify({ zellij: { features } }) });
    expect(() => withZellij({}, { appRoot })).toThrow(/packages are missing.*npm install/);
  });

  it.each([null, 'all', ['unknown'], ['maths', 'maths'], [null], [['maths']]])('rejects invalid selections before building: %j', async (features) => {
    const appRoot = await wrapper({ 'package.json': JSON.stringify({ zellij: { features } }) });
    expect(() => withZellij({}, { appRoot })).toThrow(/zellij.features/);
  });

  it('preserves automatic detection for legacy wrappers without feature metadata', async () => {
    const appRoot = await wrapper({ 'package.json': '{}' });
    const config = withZellij({}, { appRoot });
    expect(config.env.ZELLIJ_MATHS).toBe('false');
    expect(config.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS).toBe('false');
  });
});
