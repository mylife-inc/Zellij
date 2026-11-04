import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { loadZellijRuntime, resetZellijRuntime } from './runtime';

const roots: string[] = [];
afterEach(async () => {
  resetZellijRuntime();
  vi.unstubAllEnvs();
  vi.restoreAllMocks();
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});
async function fixture() {
  const appRoot = await mkdtemp(path.join(tmpdir(), 'zellij-runtime-'));
  roots.push(appRoot);
  for (const [folder, name] of [['content', 'First'], ['other', 'Second']]) {
    await mkdir(path.join(appRoot, folder!));
    await writeFile(path.join(appRoot, folder!, 'site.yaml'), `name: ${name}\n`);
  }
  return appRoot;
}

it('isolates different content roots and runtime options even in production', async () => {
  vi.stubEnv('NODE_ENV', 'production');
  const appRoot = await fixture();
  const first = loadZellijRuntime({ appRoot });
  expect(loadZellijRuntime({ appRoot })).toBe(first);
  const second = await loadZellijRuntime({ appRoot, contentDir: 'other' });
  const prefixed = await loadZellijRuntime({ appRoot, basePath: '/custom' });
  expect((await first).site.config.name).toBe('First');
  expect(second.site.config.name).toBe('Second');
  expect(prefixed.site.config.basePath).toBe('/custom');
  expect((await first).site.config.basePath).toBe('');
});

it('passes the built Next prefix to rendering when content omits it', async () => {
  vi.stubEnv('ZELLIJ_BASE_PATH', '/products/docs');
  const appRoot = await fixture();
  expect((await loadZellijRuntime({ appRoot })).site.config.basePath).toBe('/products/docs');
});

it('rejects runtime or content prefixes that disagree with the compiled Next prefix', async () => {
  vi.stubEnv('ZELLIJ_BASE_PATH', '/products/docs');
  const appRoot = await fixture();
  await expect(loadZellijRuntime({ appRoot, basePath: '/other' })).rejects.toThrow(/basePath.*disagree/i);
  await writeFile(path.join(appRoot, 'content/site.yaml'), 'name: Test\nbasePath: /different\n');
  await expect(loadZellijRuntime({ appRoot })).rejects.toThrow(/basePath.*disagree/i);
});

it('retries a failed production load after the content is repaired', async () => {
  vi.stubEnv('NODE_ENV', 'production');
  const appRoot = await fixture();
  await writeFile(path.join(appRoot, 'content/site.yaml'), 'name: []\n');
  await expect(loadZellijRuntime({ appRoot })).rejects.toThrow();
  await writeFile(path.join(appRoot, 'content/site.yaml'), 'name: Repaired\n');
  expect((await loadZellijRuntime({ appRoot })).site.config.name).toBe('Repaired');
});

it('invalidates the development memo on successful preparation without waiting for the TTL', async () => {
  vi.stubEnv('NODE_ENV', 'development');
  vi.spyOn(Date, 'now').mockReturnValue(1000);
  const appRoot = await fixture();
  const prepared = path.join(appRoot, 'public/_zellij');
  await mkdir(prepared, { recursive: true });
  const revision = (value: string) => writeFile(path.join(prepared, 'dev-revision.json'), JSON.stringify({ version: 1, revision: value }));
  await revision('first');
  const initial = await loadZellijRuntime({ appRoot });
  expect(initial.developmentRevision).toBe('first');
  await writeFile(path.join(appRoot, 'content/site.yaml'), 'name: Updated\n');
  await revision('second');
  const updated = await loadZellijRuntime({ appRoot });
  expect(updated.site.config.name).toBe('Updated');
  expect(updated.developmentRevision).toBe('second');

  vi.stubEnv('NODE_ENV', 'production');
  const production = await loadZellijRuntime({ appRoot });
  expect(production.developmentRevision).toBeUndefined();
  await revision('third');
  expect(await loadZellijRuntime({ appRoot })).toBe(production);
});
