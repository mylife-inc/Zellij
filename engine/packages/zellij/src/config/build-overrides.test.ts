import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { readBuildOverrides } from '../../deployment-config.cjs';
// @ts-expect-error — plain JS bootstrap, loaded directly by Next.
import { withZellij } from '../../next-config.mjs';
import { loadZellijRuntime, resetZellijRuntime } from './runtime';
import { createZellijSitemap, createZellijRobots } from '../render/seo';

const roots: string[] = [];
const encode = (value: unknown) => Buffer.from(JSON.stringify(value)).toString('hex');
function overrides(value: Record<string, unknown>) {
  vi.stubEnv('ZELLIJ_BUILD_OVERRIDES', encode({ version: 1, noCheck: false, ...value }));
}
async function fixture(files: Record<string, string>) {
  const appRoot = await mkdtemp(path.join(tmpdir(), 'zellij-overrides-'));
  roots.push(appRoot);
  for (const [name, text] of Object.entries(files)) {
    const file = path.join(appRoot, name);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, text);
  }
  return appRoot;
}
afterEach(async () => {
  resetZellijRuntime();
  vi.unstubAllEnvs();
  await Promise.all(roots.splice(0).map(root => rm(root, { recursive: true, force: true })));
});

it.each(['', '0', 'zz', 'ff', '7b', '00'.repeat(16385)])('rejects corrupt protocol data', encoded => {
  vi.stubEnv('ZELLIJ_BUILD_OVERRIDES', encoded);
  expect(() => readBuildOverrides()).toThrow(/build overrides/i);
});
it.each([
  null, [], { version: 2, noCheck: false }, { version: 1 },
  { version: 1, noCheck: 'false' }, { version: 1, noCheck: false, extra: true },
])('rejects unsupported protocol shapes: %j', value => {
  vi.stubEnv('ZELLIJ_BUILD_OVERRIDES', encode(value));
  expect(() => readBuildOverrides()).toThrow(/build overrides/i);
});
it.each(['/../docs', '/a//b', '//host', '/a%20b', '/a?x', '/a#x', '/a\\b', '/a\nb', 1, null])('rejects an invalid prefix: %j', basePath => {
  overrides({ basePath });
  expect(() => readBuildOverrides()).toThrow(/basePath/i);
});
it.each(['https:example.com', 'https:///example.com', 'https://example.com/a/../', 'https://example.com/docs', 'file:///site', 'https://u:p@example.com', 'https://@example.com', 'https://example.com/?', 'https://example.com/#', ' https://example.com', 'https://example.com\\', 1, null])('rejects a non-origin site URL: %j', siteUrl => {
  overrides({ siteUrl });
  expect(() => readBuildOverrides()).toThrow(/siteUrl/i);
});

it('normalizes root and origins and treats shell-looking prefixes only as data', () => {
  overrides({ basePath: '/', siteUrl: 'https://EXAMPLE.com:443/', noCheck: true });
  expect(readBuildOverrides()).toEqual({ basePath: '', siteUrl: 'https://example.com' });
  overrides({ basePath: "/日本語/'$(literal)`value`/", siteUrl: 'http://[::1]:3000' });
  expect(readBuildOverrides()).toEqual({ basePath: "/日本語/'$(literal)`value`", siteUrl: 'http://[::1]:3000' });
});

it('overrides authored and wrapper prefixes, metadata and production cache without changing files', async () => {
  vi.stubEnv('NODE_ENV', 'production');
  const authored = 'name: Test\nbasePath: /authored\nsiteUrl: https://old.example\n';
  const appRoot = await fixture({
    'content/site.yaml': authored,
    'content/manual/_sidebar.yaml': 'title: Manual\nnav: [index]\n',
    'content/manual/index.mdx': '# Guide\n',
  });
  const initial = await loadZellijRuntime({ appRoot });
  overrides({ basePath: '/invocation', siteUrl: 'https://new.example' });
  const next = withZellij({ basePath: '/wrapper', env: { CUSTOM: 'kept', ZELLIJ_BASE_PATH: '/wrapper' } }, { appRoot });
  expect(next.basePath).toBe('/invocation');
  expect(next.env).toMatchObject({ CUSTOM: 'kept', ZELLIJ_BASE_PATH: '/invocation', ZELLIJ_BUILD_OVERRIDES: process.env.ZELLIJ_BUILD_OVERRIDES });
  vi.stubEnv('ZELLIJ_BASE_PATH', next.basePath);
  const updated = await loadZellijRuntime({ appRoot, basePath: '/wrapper' });
  expect(updated.site.config).toMatchObject({ basePath: '/invocation', siteUrl: 'https://new.example' });
  expect(updated).not.toBe(initial);
  expect(await createZellijSitemap({ appRoot })()).toContainEqual(expect.objectContaining({ url: 'https://new.example/invocation/manual' }));
  expect(await createZellijRobots({ appRoot })()).toMatchObject({ sitemap: 'https://new.example/invocation/sitemap.xml' });
  overrides({ basePath: '/', siteUrl: 'https://root.example' });
  vi.stubEnv('ZELLIJ_BASE_PATH', '');
  expect((await loadZellijRuntime({ appRoot })).site.config).toMatchObject({ basePath: '', siteUrl: 'https://root.example' });
  expect(await readFile(path.join(appRoot, 'content/site.yaml'), 'utf8')).toBe(authored);
  vi.stubEnv('ZELLIJ_BUILD_OVERRIDES', undefined);
  vi.stubEnv('ZELLIJ_BASE_PATH', undefined);
  expect(await loadZellijRuntime({ appRoot })).toBe(initial);
});

it('applies the same settings to legacy content and keeps compiled prefix checks', async () => {
  const appRoot = await fixture({
    'zellij.config.yaml': 'product: { name: Old }\nbasePath: /old\nsiteUrl: https://old.example\n',
    'content/marketing/index.yaml': 'title: Home\ndescription: Legacy home\nsections: []\n',
  });
  overrides({ basePath: '/new', siteUrl: 'https://new.example' });
  expect(withZellij({}, { appRoot }).basePath).toBe('/new');
  expect((await loadZellijRuntime({ appRoot, onWarning: () => {} })).site.config).toMatchObject({ basePath: '/new', siteUrl: 'https://new.example' });
  vi.stubEnv('ZELLIJ_BASE_PATH', '/compiled-elsewhere');
  await expect(loadZellijRuntime({ appRoot })).rejects.toThrow(/basePath.*disagree/i);
});

it.each(['name: []\n', 'basePath: /a/../b\n', 'siteUrl: not-a-url\n'])('does not hide invalid authored settings or disable preparation validation: %s', async authored => {
  const appRoot = await fixture({ 'content/site.yaml': authored });
  overrides({ basePath: '/new', siteUrl: 'https://new.example', noCheck: true });
  await expect(loadZellijRuntime({ appRoot })).rejects.toThrow();
});
