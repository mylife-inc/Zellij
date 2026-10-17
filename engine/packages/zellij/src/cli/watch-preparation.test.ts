import { mkdir, mkdtemp, readFile, rename, rm, stat, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { afterEach, expect, it } from 'vitest';
import { watchPreparation, type PreparationWatch } from './watch-preparation';

const roots: string[] = [];
const watchers: PreparationWatch[] = [];
afterEach(async () => {
  await Promise.all(watchers.splice(0).map((watcher) => watcher.close()));
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});
async function fixture(inAppRoot = false) {
  const appRoot = await mkdtemp(path.join(tmpdir(), 'zellij-watch-'));
  roots.push(appRoot);
  const content = inAppRoot ? appRoot : path.join(appRoot, 'external café');
  const put = async (file: string, text: string) => {
    await mkdir(path.dirname(path.join(content, file)), { recursive: true });
    await writeFile(path.join(content, file), text);
  };
  await put('site.yaml', 'name: Watch\nhome: 01-guide\n');
  await put('01-guide/_sidebar.yaml', 'route: /manual\nnav: [index]\n');
  await put('01-guide/index.mdx', '# Welcome\n\n## Before\n\nOriginal passage.\n');
  await put('assets/logo.svg', '<svg>before</svg>');
  const output = path.join(appRoot, 'public/_zellij');
  const json = async (file: string) => JSON.parse(await readFile(path.join(output, file), 'utf8'));
  const search = async () => JSON.stringify(await json('search-index.json'));
  const errors: unknown[] = [];
  const start = async (legacy = false) => {
    const watcher = await watchPreparation({ appRoot, ...(legacy ? { configFile: 'legacy.yaml' } : { contentDir: content }), onWarning: () => {} }, {
      intervalMs: 20, onError: (error) => errors.push(error),
    });
    watchers.push(watcher);
    return watcher;
  };
  return { appRoot, content, output, put, json, search, start, errors };
}

it('updates edited, added, renamed and deleted assets and canonical search routes', async () => {
  const f = await fixture();
  await f.start();
  const originalHash = (await f.json('asset-manifest.json'))['logo.svg'];
  await f.put('assets/logo.svg', '<svg>after!</svg>');
  await f.put('assets/nested/new.svg', '<svg>added</svg>');
  await f.put('01-guide/index.mdx', '# Welcome\n\n## After\n\nChanged passage.\n');
  await f.put('01-guide/extra.mdx', '---\nroute: /extra\n---\n# Extra\n\nAdded passage.\n');
  await expect.poll(f.search).toContain('/manual#after');
  await expect.poll(f.search).toContain('/extra');
  await expect.poll(async () => (await f.json('asset-manifest.json'))['logo.svg']).not.toBe(originalHash);
  expect(await readFile(path.join(f.output, 'assets/nested/new.svg'), 'utf8')).toBe('<svg>added</svg>');
  await rename(path.join(f.content, 'assets/nested/new.svg'), path.join(f.content, 'assets/moved.svg'));
  await rm(path.join(f.content, '01-guide/extra.mdx'));
  await expect.poll(f.search).not.toContain('/extra');
  await expect.poll(async () => Object.keys(await f.json('asset-manifest.json'))).toEqual(['logo.svg', 'moved.svg']);
  await expect(readFile(path.join(f.output, 'assets/nested/new.svg'))).rejects.toMatchObject({ code: 'ENOENT' });
});

it('keeps previous output through invalid edits and recovers without restarting', async () => {
  const f = await fixture();
  await f.start();
  const original = await f.search();
  await f.put('01-guide/index.mdx', '# Broken\n\n<Callout>\n');
  await f.put('assets/logo.svg', '<svg>after</svg>');
  await expect.poll(() => f.errors.length).toBe(1);
  expect(await f.search()).toBe(original);
  expect(await readFile(path.join(f.output, 'assets/logo.svg'), 'utf8')).toBe('<svg>before</svg>');
  await delay(100);
  expect(f.errors).toHaveLength(1);
  await f.put('01-guide/index.mdx', '# Repaired\n\nRecovered passage.\n');
  await expect.poll(f.search).toContain('Recovered passage');
  await expect.poll(() => readFile(path.join(f.output, 'assets/logo.svg'), 'utf8')).toBe('<svg>after</svg>');
  await f.put('site.yaml', 'name: [');
  await expect.poll(() => f.errors.length).toBe(2);
  await f.put('site.yaml', 'name: Repaired\nhome: 01-guide\n');
  await f.put('01-guide/index.mdx', '# Repaired again\n\nStill watching.\n');
  await expect.poll(f.search).toContain('Still watching');
});

it('handles whole-directory removal/recreation and stops all updates on close', async () => {
  const f = await fixture();
  const watcher = await f.start();
  await rm(path.join(f.content, 'assets'), { recursive: true });
  await expect.poll(() => f.json('asset-manifest.json')).toEqual({});
  await expect(readFile(path.join(f.output, 'assets/logo.svg'))).rejects.toMatchObject({ code: 'ENOENT' });
  await f.put('assets/logo.svg', '<svg>recreated</svg>');
  await expect.poll(async () => Object.keys(await f.json('asset-manifest.json'))).toEqual(['logo.svg']);
  const displaced = `${f.content}-saved`;
  await rename(f.content, displaced);
  await expect.poll(() => f.errors.length).toBe(1);
  await rename(displaced, f.content);
  await f.put('01-guide/index.mdx', '# Returned\n\nThe root returned.\n');
  await expect.poll(f.search).toContain('The root returned');
  await watcher.close();
  await f.put('01-guide/index.mdx', '# Stopped\n\nMust not publish.\n');
  await delay(100);
  expect(await f.search()).not.toContain('Must not publish');
});

it('watches a custom legacy config and follows its changed content root', async () => {
  const f = await fixture();
  const config = path.join(f.appRoot, 'legacy.yaml');
  await writeFile(config, 'product: { name: Watch }\ncontent: external café\n');
  await f.start(true);
  const second = path.join(f.appRoot, 'second');
  await rename(f.content, second);
  await writeFile(path.join(second, '01-guide/index.mdx'), '# Second\n\nNew root passage.\n');
  await writeFile(config, 'product: { name: Watch }\ncontent: second\n');
  await expect.poll(f.search).toContain('New root passage');
});

it('does not loop on its own output when content is the application root', async () => {
  const f = await fixture(true);
  await f.start();
  await delay(200); // allow the first public directory creation to settle
  const before = (await stat(path.join(f.output, 'search-index.json'))).mtimeMs;
  await mkdir(path.join(f.appRoot, '.next'), { recursive: true });
  await writeFile(path.join(f.appRoot, '.next/build'), 'Next output');
  await delay(200);
  expect((await stat(path.join(f.output, 'search-index.json'))).mtimeMs).toBe(before);
  expect(f.errors).toEqual([]);
});

it('does not recopy images for prose edits or rebuild search for asset-only edits', async () => {
  const f = await fixture();
  await f.start();
  const manifest = path.join(f.output, 'asset-manifest.json');
  const before = (await stat(manifest)).mtimeMs;
  await f.put('01-guide/index.mdx', '# Welcome\n\nOnly prose changed.\n');
  await expect.poll(f.search).toContain('Only prose changed');
  expect((await stat(manifest)).mtimeMs).toBe(before);
  const index = path.join(f.output, 'search-index.json');
  const indexBefore = (await stat(index)).mtimeMs;
  await f.put('assets/logo.svg', '<svg>only image changed</svg>');
  await expect.poll(() => readFile(path.join(f.output, 'assets/logo.svg'), 'utf8')).toContain('only image changed');
  expect((await stat(index)).mtimeMs).toBe(indexBefore);
});

it('excludes prepared output when a symlink points at an app-root content layout', async () => {
  const f = await fixture(true);
  const alias = `${f.appRoot}-alias`;
  roots.push(alias);
  await symlink(f.appRoot, alias, 'dir');
  const watcher = await watchPreparation({ appRoot: f.appRoot, contentDir: alias, onWarning: () => {} }, { intervalMs: 20 });
  watchers.push(watcher);
  await delay(200);
  const before = (await stat(path.join(f.output, 'search-index.json'))).mtimeMs;
  await delay(200);
  expect((await stat(path.join(f.output, 'search-index.json'))).mtimeMs).toBe(before);
  await f.put('01-guide/index.mdx', '# Alias\n\nUpdated through alias.\n');
  await expect.poll(f.search).toContain('Updated through alias');
});
