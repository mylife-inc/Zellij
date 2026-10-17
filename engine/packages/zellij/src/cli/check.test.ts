import { mkdtemp, mkdir, writeFile, rm, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { check } from './check';

const roots: string[] = [];
afterEach(async () => {
  vi.restoreAllMocks();
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});
async function fixture(files: Record<string, string>) {
  const root = await mkdtemp(path.join(tmpdir(), 'zellij-deep-'));
  roots.push(root);
  for (const [name, body] of Object.entries(files)) {
    const target = path.join(root, name);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, body);
  }
  vi.spyOn(console, 'log').mockImplementation(() => {});
  vi.spyOn(console, 'error').mockImplementation(() => {});
  vi.spyOn(console, 'warn').mockImplementation(() => {});
  return root;
}
const HOME = { 'pages.yaml': '- name: home\n  sections: [hero]\n', 'sections/hero.yaml': 'type: hero\nheadline: Home\n' };
const GUIDE = { '01-guide/_sidebar.yaml': 'route: /manual\nnav: [intro, nested/detail]\n', '01-guide/intro.mdx': '# Introduction\n\n## Start\n\n## Start\n', '01-guide/nested/detail.mdx': '# Detail\n' };

it('keeps fast checks but deep mode rejects section links, missing assets and malformed MDX', async () => {
  const contentDir = await fixture({ ...HOME, ...GUIDE,
    'sections/hero.yaml': 'type: hero\nheadline: Home\nvisual: assets/missing.png\ncta: [{label: Missing, href: /not-a-page}]\n',
    '01-guide/nested/detail.mdx': '# Detail\n\n<Callout>\n',
  });
  expect((await check({ contentDir })).ok).toBe(true);
  const result = await check({ contentDir, deep: true });
  expect(result.ok).toBe(false);
  expect(result.errors?.join('\n')).toMatch(/sections\/hero.yaml.*no page resolves/s);
  expect(result.errors?.join('\n')).toMatch(/missing.png.*cannot read asset/);
  expect(result.errors?.join('\n')).toMatch(/nested\/detail.mdx.*invalid MDX syntax/);
});

it('strict mode fails on warnings without changing the default result', async () => {
  const contentDir = await fixture({ ...HOME, 'site.yaml': 'name: Test\nfutureOption: true\n' });
  expect((await check({ contentDir })).ok).toBe(true);
  expect((await check({ contentDir, strict: true })).ok).toBe(false);
});

it('accepts canonical/relative routes, queries, redirected anchors, reference images and literal JSX', async () => {
  const contentDir = await fixture({ ...GUIDE,
    'site.yaml': 'name: Test\nnav: {cta: [{label: Read, href: "/manual/intro?mode=full#start"}]}\n',
    'assets/café image.svg': '<svg/>',
    '01-guide/nested/detail.mdx': '# Detail\n\n[Back](../intro?mode=full#start-1)\n\n[Root](/manual#start)\n\n![Asset][picture]\n\n[picture]: /_zellij/assets/caf%C3%A9%20image.svg?v=1\n\n<Screenshot src={"assets/café image.svg"} alt="Test" width={1200} height={630} />\n\n<Card href={"/manual/intro#start"}>Read</Card>\n\n<div id="custom" />\n\n[Custom](#custom)\n',
  });
  const result = await check({ contentDir, deep: true, strict: true });
  expect(result.errors).toEqual([]);
  expect(result.warnings).toEqual([]);
  expect(result.ok).toBe(true);
});

it('reports broken prose routes and anchors after title stripping and preserves literal menu URLs', async () => {
  const contentDir = await fixture({ ...GUIDE,
    'menu.yaml': '- name: Elsewhere\n  url: /unmanaged\n',
    '01-guide/nested/detail.mdx': '# Detail\n\n[Missing](/missing)\n\n[Old title](/manual/intro#introduction)\n\n[No anchor](/manual#start-2)\n',
  });
  const result = await check({ contentDir, deep: true });
  expect(result.errors).toEqual([
    expect.stringContaining('no page resolves'),
    expect.stringContaining('no anchor #introduction'),
    expect.stringContaining('no anchor #start-2'),
  ]);
  expect(result.errors?.join('\n')).not.toContain('unmanaged');
});

it('does not execute MDX or inspect code examples and reports computed references', async () => {
  const contentDir = await fixture({ ...GUIDE,
    '01-guide/nested/detail.mdx': '# Detail\n\n`[Example](/missing)`\n\n```md\n![Example](/missing.png)\n```\n\n[Remote](https://example.invalid/missing)\n\n{(() => { throw new Error("must never execute"); })()}\n\n<img src={globalThis.unknown} />\n',
  });
  const result = await check({ contentDir, deep: true });
  expect(result.ok).toBe(true);
  expect(result.warnings.join('\n')).toMatch(/JavaScript is not evaluated/);
  expect(result.warnings.join('\n')).toMatch(/computed JSX/);
  expect((await check({ contentDir, deep: true, strict: true })).ok).toBe(false);
});

it('validates inline links with the same emphasis/code semantics as rendering', async () => {
  const contentDir = await fixture({ ...HOME,
    'sections/hero.yaml': 'type: cta-banner\nheadline: Home\nbody: "**[Missing](/missing)** and `[Example](/not-a-link)`"\n',
  });
  const result = await check({ contentDir, deep: true });
  expect(result.errors).toHaveLength(1);
  expect(result.errors?.[0]).toContain('/missing');
});

it('checks site media, nested section media, raw image URLs and asset containment', async () => {
  const contentDir = await fixture({ ...HOME, ...GUIDE,
    'site.yaml': 'name: Test\nogImage: assets/og.png\nbrand: {logo: {src: assets/logo.png, dark: assets/dark.png}}\n',
    'sections/hero.yaml': 'type: team-grid\npeople: [{name: Person, role: Test, photo: assets/photo.png}]\n',
    '01-guide/nested/detail.mdx': '# Detail\n\n![Wrong](assets/exists.svg)\n\n<Screenshot src="assets/../outside.svg" alt="Bad" />\n',
    'assets/exists.svg': '<svg/>', 'outside.svg': '<svg/>',
  });
  const result = await check({ contentDir, deep: true });
  expect(result.errors).toHaveLength(6);
  expect(result.errors?.join('\n')).toMatch(/og.png[\s\S]*logo.png|logo.png[\s\S]*og.png/);
  expect(result.errors?.join('\n')).toContain('outside /_zellij/assets/');
});

it('rejects asset symlinks escaping the assets directory and directories used as files', async () => {
  const contentDir = await fixture({ ...HOME,
    'sections/hero.yaml': 'type: hero\nheadline: Home\nvisual: assets/link.svg\n',
    'outside.svg': '<svg/>', 'assets/dir/child.svg': '<svg/>',
  });
  await symlink('../outside.svg', path.join(contentDir, 'assets/link.svg'));
  expect((await check({ contentDir, deep: true })).errors?.join('\n')).toContain('symlink escapes');
  await writeFile(path.join(contentDir, 'sections/hero.yaml'), 'type: hero\nheadline: Home\nvisual: assets/dir\n');
  expect((await check({ contentDir, deep: true })).errors?.join('\n')).toContain('not a file');
});

it('checks SectionPreview examples in their page context and rejects missing or recursive previews', async () => {
  const contentDir = await fixture({ ...GUIDE,
    'examples/sample.yaml': 'type: hero\nheadline: Example\ncta: [{label: Start, href: /manual#start}]\n',
    '01-guide/nested/detail.mdx': '# Detail\n\n<SectionPreview example="sample" />\n\n<SectionPreview example="missing" />\n',
  });
  expect((await check({ contentDir, deep: true })).errors).toEqual([expect.stringContaining('no section example named "missing"')]);
  await writeFile(path.join(contentDir, 'examples/sample.yaml'), 'type: markdown-prose\nbody: \'<SectionPreview example="sample" />\'\n');
  expect((await check({ contentDir, deep: true })).errors?.join('\n')).toContain('recursive section example');
});

it('checks footnote anchors and detects duplicate IDs across rendered prose sections', async () => {
  const contentDir = await fixture({
    'pages.yaml': '- name: home\n  sections: [one, two]\n',
    'sections/one.md': '## Repeat\n\nFootnote[^note].\n\n[^note]: Details.\n\n[Note](#user-content-fn-note)\n',
    'sections/two.md': '## Repeat\n',
  });
  const result = await check({ contentDir, deep: true });
  expect(result.errors).toEqual([expect.stringContaining('duplicate rendered anchor #repeat')]);
});

it('does not accept an assets root symlink pointing outside the content folder', async () => {
  const outside = await fixture({ 'image.svg': '<svg/>' });
  const contentDir = await fixture({ ...HOME,
    'sections/hero.yaml': 'type: hero\nheadline: Home\nvisual: assets/image.svg\n',
  });
  await symlink(outside, path.join(contentDir, 'assets'));
  expect((await check({ contentDir, deep: true })).errors?.join('\n')).toContain('assets directory symlink escapes');
});

it('uses the first Markdown reference definition, matching rendered links', async () => {
  const contentDir = await fixture({ ...GUIDE,
    '01-guide/nested/detail.mdx': '# Detail\n\n[Read][target]\n\n[target]: /missing\n[target]: /manual/intro\n',
  });
  expect((await check({ contentDir, deep: true })).errors).toEqual([expect.stringContaining('no page resolves to "/missing"')]);
});

it('verifies a rendered example fragment in its page without a contextless warning', async () => {
  const contentDir = await fixture({ ...GUIDE,
    'examples/sample.yaml': 'type: hero\nheadline: Example\ncta: [{label: Details, href: "#details"}]\n',
    '01-guide/nested/detail.mdx': '# Detail\n\n## Details\n\n<SectionPreview example="sample" />\n',
  });
  const result = await check({ contentDir, deep: true, strict: true });
  expect(result.errors).toEqual([]);
  expect(result.warnings).toEqual([]);
  expect(result.ok).toBe(true);
});

it('does not claim component props are rendered anchors or custom component URLs are verified', async () => {
  const contentDir = await fixture({ ...GUIDE,
    '01-guide/nested/detail.mdx': '# Detail\n\n<Callout id="maybe">Text</Callout>\n\n<Custom href="/not-necessarily-a-link" />\n\n[Target](#maybe)\n',
  });
  const result = await check({ contentDir, deep: true, strict: true });
  expect(result.errors).toEqual([]);
  expect(result.warnings.join('\n')).toContain('component id props');
  expect(result.warnings.join('\n')).toContain('anchor #maybe');
  expect(result.ok).toBe(false);
});
