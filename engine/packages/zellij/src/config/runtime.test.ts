import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { loadZellijRuntime, resetZellijRuntime } from './runtime';

const roots: string[] = [];
afterEach(async () => {
  resetZellijRuntime();
  vi.unstubAllEnvs();
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
