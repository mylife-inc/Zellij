import { mkdtemp, mkdir, writeFile, rename, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { expect, it, vi } from 'vitest';
import { ASSET_MANIFEST_FILE } from './paths';
import { assetHref, publicAssetHref } from './asset-href';

it('sees atomically replaced asset fingerprints within a single clock tick', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'zellij-fingerprints-'));
  vi.spyOn(process, 'cwd').mockReturnValue(root);
  vi.spyOn(Date, 'now').mockReturnValue(1000);
  vi.stubEnv('NODE_ENV', 'development');
  const manifest = path.join(root, 'public/_zellij', ASSET_MANIFEST_FILE);
  const replace = async (hash: string) => {
    await writeFile(`${manifest}.new`, JSON.stringify({ 'image.svg': hash, '日本/a b.svg': hash }));
    await rename(`${manifest}.new`, manifest);
  };
  try {
    await mkdir(path.dirname(manifest), { recursive: true });
    await replace('aaaaaaaa');
    expect(assetHref('assets/image.svg', '/docs')).toBe('/docs/_zellij/assets/image.svg?v=aaaaaaaa');
    expect(publicAssetHref('/_zellij/assets/%E6%97%A5%E6%9C%AC/a%20b.svg?view=1&v=old#icon', '/docs'))
      .toBe('/docs/_zellij/assets/%E6%97%A5%E6%9C%AC/a%20b.svg?view=1&v=aaaaaaaa#icon');
    expect(publicAssetHref('https://example.com/image.svg', '/docs')).toBe('https://example.com/image.svg');
    expect(publicAssetHref('/_zellij/assets/bad%zz.svg', '/docs')).toBe('/docs/_zellij/assets/bad%zz.svg');
    await replace('bbbbbbbb');
    expect(assetHref('assets/image.svg', '/docs')).toBe('/docs/_zellij/assets/image.svg?v=bbbbbbbb');
    expect(publicAssetHref('/_zellij/assets/image.svg', '/docs')).toBe('/docs/_zellij/assets/image.svg?v=bbbbbbbb');
    vi.stubEnv('NODE_ENV', 'production');
    await replace('cccccccc');
    expect(assetHref('assets/image.svg', '/docs')).toBe('/docs/_zellij/assets/image.svg?v=bbbbbbbb');
    vi.stubEnv('NODE_ENV', 'development');
    expect(assetHref('assets/image.svg')).toBe('/_zellij/assets/image.svg?v=cccccccc');
    await rm(manifest);
    expect(assetHref('assets/image.svg')).toBe('/_zellij/assets/image.svg');
    await replace('dddddddd');
    expect(assetHref('assets/image.svg')).toBe('/_zellij/assets/image.svg?v=dddddddd');
  } finally {
    vi.restoreAllMocks();
    vi.unstubAllEnvs();
    await rm(root, { recursive: true, force: true });
  }
});
