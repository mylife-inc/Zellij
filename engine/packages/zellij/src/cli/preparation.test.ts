import { mkdtemp, mkdir, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it } from 'vitest';
import { writeSearchIndex } from './write-search-index';
import { syncAssets } from './sync-assets';
import type { SearchEntry } from '../search/build-index';

const roots: string[] = [];
afterEach(async () => { await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true }))); });

async function fixture(folder = 'content') {
  const appRoot = await mkdtemp(path.join(tmpdir(), 'zellij-prepare-'));
  roots.push(appRoot);
  const content = path.join(appRoot, folder);
  const files = {
    'site.yaml': 'name: Test\nhome: 01-guide\nbasePath: /products/docs\n',
    '01-guide/_sidebar.yaml': 'route: /manual\nnav: [index, setup]\n',
    '01-guide/index.mdx': '# Install\n\n## Install\n\nFirst paragraph.\n\n## Install\n\nSecond paragraph.\n',
    '01-guide/setup.mdx': '---\ntitle: Setup\nroute: /custom/setup\n---\n# Setup\n\nSetup instructions.\n',
    '01-guide/unlisted.mdx': '# Unlisted\n\nStill rendered and searchable.\n',
    'assets/logo.svg': '<svg>fixture</svg>',
  };
  for (const [name, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(content, name)), { recursive: true });
    await writeFile(path.join(content, name), text);
  }
  return { appRoot, content };
}

async function entries(appRoot: string): Promise<SearchEntry[]> {
  return JSON.parse(await readFile(path.join(appRoot, 'public/_zellij/search-index.json'), 'utf8'));
}

it('indexes the canonical guide routes and the body with its leading title removed', async () => {
  const { appRoot } = await fixture();
  await writeSearchIndex(appRoot);
  const routes = (await entries(appRoot)).map((entry) => entry.route);
  expect(routes).toEqual(['/manual', '/manual#install', '/manual#install-1', '/custom/setup', '/manual/unlisted']);
  // Next router applies the deployment prefix; search stores canonical routes.
  expect(routes.every((route) => !route.startsWith('/products/docs'))).toBe(true);
});

it('prepares search and assets from the same explicit content root as rendering', async () => {
  const { appRoot, content } = await fixture('external content');
  await writeSearchIndex({ appRoot, contentDir: content });
  await syncAssets({ appRoot, contentDir: 'external content' });
  expect((await entries(appRoot)).some((entry) => entry.route === '/manual#install')).toBe(true);
  expect(await readFile(path.join(appRoot, 'public/_zellij/assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});

it('uses the legacy configuration path consistently for preparation', async () => {
  const { appRoot } = await fixture('external');
  await writeFile(path.join(appRoot, 'zellij.config.yaml'), 'product: { name: Test }\ncontent: ./external\n');
  await writeSearchIndex(appRoot);
  await syncAssets(appRoot);
  expect((await entries(appRoot))[0]?.route).toBe('/manual');
  expect(await readFile(path.join(appRoot, 'public/_zellij/assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});

it('refreshes changed content instead of reusing the rendering cache', async () => {
  const { appRoot, content } = await fixture();
  await writeSearchIndex(appRoot);
  await writeFile(path.join(content, '01-guide/index.mdx'), '# Install\n\n## Changed\n\nNew passage.\n');
  await writeSearchIndex(appRoot);
  const routes = (await entries(appRoot)).map((entry) => entry.route);
  expect(routes).toContain('/manual#changed');
  expect(routes).not.toContain('/manual#install');
});

it('removes stale prepared assets and hashes when the source assets folder disappears', async () => {
  const { appRoot, content } = await fixture();
  await syncAssets(appRoot);
  await rm(path.join(content, 'assets'), { recursive: true });
  await syncAssets(appRoot);
  await expect(readFile(path.join(appRoot, 'public/_zellij/assets/logo.svg'))).rejects.toMatchObject({ code: 'ENOENT' });
  expect(JSON.parse(await readFile(path.join(appRoot, 'public/_zellij/asset-manifest.json'), 'utf8'))).toEqual({});
});

it('rejects an overlapping source before deleting any authored assets', async () => {
  const { appRoot, content } = await fixture('public/_zellij');
  await expect(syncAssets({ appRoot, contentDir: content })).rejects.toThrow(/must not overlap/);
  expect(await readFile(path.join(content, 'assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});

it('rejects an assets file without clearing previously prepared assets', async () => {
  const { appRoot, content } = await fixture();
  await syncAssets(appRoot);
  await rm(path.join(content, 'assets'), { recursive: true });
  await writeFile(path.join(content, 'assets'), 'Not a directory');
  await expect(syncAssets(appRoot)).rejects.toThrow(/assets must be a directory/);
  expect(await readFile(path.join(appRoot, 'public/_zellij/assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});

it('detects overlap through a public-directory symlink', async () => {
  const { appRoot, content } = await fixture();
  await mkdir(path.join(appRoot, 'public'));
  await symlink(content, path.join(appRoot, 'public/_zellij'), 'dir');
  await expect(syncAssets(appRoot)).rejects.toThrow(/must not overlap/);
  expect(await readFile(path.join(content, 'assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});

it('fails on a missing explicit content root without clearing previous assets or search', async () => {
  const { appRoot } = await fixture();
  await syncAssets(appRoot);
  await writeSearchIndex(appRoot);
  const previous = await entries(appRoot);
  const options = { appRoot, contentDir: 'missing' };
  await expect(syncAssets(options)).rejects.toThrow(/No content directory/);
  await expect(writeSearchIndex(options)).rejects.toThrow(/No content directory/);
  expect(await entries(appRoot)).toEqual(previous);
  expect(await readFile(path.join(appRoot, 'public/_zellij/assets/logo.svg'), 'utf8')).toBe('<svg>fixture</svg>');
});
