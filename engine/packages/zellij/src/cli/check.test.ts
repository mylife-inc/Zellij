import { mkdtemp, mkdir, writeFile, rm, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { check, type JsonCheckReport } from './check';

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

it('emits one JSON report for success, warnings, strict failure and deep findings', async () => {
  const contentDir = await fixture({ ...HOME, ...GUIDE, 'site.yaml': 'name: JSON\nfutureOption: true\n' });
  for (const strict of [false, true]) {
    vi.mocked(console.log).mockClear();
    const result = await check({ contentDir, json: true, deep: true, strict });
    expect(result.ok).toBe(!strict);
    expect(console.log).toHaveBeenCalledTimes(1);
    const report: JsonCheckReport = JSON.parse(vi.mocked(console.log).mock.calls[0]![0]);
    expect(report).toMatchObject({ schema_version: 1, target: 'static', root: contentDir, deep: true, strict, ok: !strict });
    expect(report.pages).toBeGreaterThan(0);
    expect(report.diagnostics).toEqual([expect.objectContaining({ severity: 'warning', code: 'warning', file: null, message: expect.stringContaining('unknown key') })]);
    expect(console.warn).not.toHaveBeenCalled();
    expect(console.error).not.toHaveBeenCalled();
  }
  await writeFile(path.join(contentDir, 'site.yaml'), 'name: JSON\n');
  await writeFile(path.join(contentDir, '01-guide/intro.mdx'), '# Intro\n\n[Missing](/missing)\n');
  vi.mocked(console.log).mockClear();
  expect((await check({ contentDir, json: true, deep: true })).ok).toBe(false);
  const report: JsonCheckReport = JSON.parse(vi.mocked(console.log).mock.calls[0]![0]);
  expect(report.diagnostics).toEqual([expect.objectContaining({ severity: 'error', code: 'deep', message: expect.stringContaining('no page resolves') })]);
  expect(console.error).not.toHaveBeenCalled();
});

it('keeps bootstrap and schema failures machine-readable without dumping the loaded site', async () => {
  const contentDir = await fixture({ ...HOME, 'site.yaml': 'name: [\n' });
  expect((await check({ contentDir, json: true })).ok).toBe(false);
  const report: JsonCheckReport = JSON.parse(vi.mocked(console.log).mock.calls[0]![0]);
  expect(report).toMatchObject({ ok: false, target: 'static', root: contentDir, pages: null });
  expect(report.diagnostics[0]).toMatchObject({ severity: 'error', code: 'content' });
  expect(report).not.toHaveProperty('site');
  expect(console.warn).not.toHaveBeenCalled();
  expect(console.error).not.toHaveBeenCalled();
});

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

it('warns about discarded section keys and fails strict checks without changing rendered defaults', async () => {
  const contentDir = await fixture({
    'site.yaml': 'name: Section diagnostics\n',
    'pages.yaml': '- name: home\n  sections: [cards]\n',
    'sections/cards.yaml': 'type: bento-grid\nitems:\n  - title: A card\n    description: This text used to disappear silently.\n    body: Visible text\n    future: {nested: hidden}\n  - title: Second card\n',
  });
  for (const strict of [false, true]) {
    vi.mocked(console.log).mockClear();
    const result = await check({ contentDir, strict, json: true });
    expect(result.errors).toEqual([]);
    expect(result.ok).toBe(!strict);
    expect(result.warnings).toHaveLength(2);
    expect(result.warnings.join('\n')).toContain('sections/cards.yaml');
    expect(result.warnings.join('\n')).toContain('items[0].description');
    expect(result.warnings.join('\n')).toContain('items[0].future');
    expect(result.warnings.join('\n')).not.toContain('This text used to disappear');
    expect(result.site?.pages[0]?.sections[0]).toMatchObject({ type: 'bento-grid', items: [{ title: 'A card', body: 'Visible text' }, { title: 'Second card' }] });
    const report: JsonCheckReport = JSON.parse(vi.mocked(console.log).mock.calls[0]![0]);
    expect(report.ok).toBe(!strict);
    expect(report.diagnostics.every(d => d.severity === 'warning' && d.code === 'warning')).toBe(true);
    expect(console.log).toHaveBeenCalledTimes(1);
    expect(console.warn).not.toHaveBeenCalled();
    expect(console.error).not.toHaveBeenCalled();
  }
});

it.each([false, true])('checks section-reference overrides before they are stripped (wrapped pages: %s)', async wrapped => {
  const pages = [{ name: 'home', sections: [{ section: 'hero', paddding: 'loose', background: { gradient: ['red', 'blue'], direction: 'left' } }] }];
  const contentDir = await fixture({ ...HOME, 'site.yaml': 'name: Overrides\n', 'pages.yaml': JSON.stringify(wrapped ? { pages } : pages) });
  const result = await check({ contentDir, strict: true });
  expect(result.ok).toBe(false);
  expect(result.warnings).toHaveLength(2);
  expect(result.warnings.join('\n')).toContain('pages.yaml');
  expect(result.warnings.join('\n')).toContain('sections[0].paddding');
  expect(result.warnings.join('\n')).toContain('sections[0].background.direction');
});

it('reports ignored Markdown frontmatter and example fields in fast checks', async () => {
  const contentDir = await fixture({
    'site.yaml': 'name: Prose\n',
    'pages.yaml': '- name: home\n  sections: [intro]\n',
    'sections/intro.mdx': '---\npaddding: loose\n---\n\n# Hello\n',
    'examples/sample.yaml': 'type: hero\nheadline: Example\ncta: [{label: Read, href: /, newTba: true}]\n',
  });
  const result = await check({ contentDir, strict: true });
  expect(result.ok).toBe(false);
  expect(result.warnings).toHaveLength(2);
  expect(result.warnings.join('\n')).toMatch(/sections\/intro.mdx.*paddding/);
  expect(result.warnings.join('\n')).toMatch(/examples\/sample.yaml.*cta\[0\].newTba/);
});

it('reports ignored legacy inline section fields with their section index', async () => {
  const contentDir = await fixture({
    'marketing/index.yaml': 'title: Legacy\ndescription: Legacy page\nsections:\n  - type: hero\n    headline: Hello\n    subhed: Lost text\n',
    'zellij.config.yaml': 'product: {name: Legacy}\ncontent: .\n',
  });
  const result = await check({ contentDir, cwd: contentDir, strict: true });
  expect(result.ok).toBe(false);
  expect(result.errors).toEqual([]);
  expect(result.warnings.join('\n')).toMatch(/marketing\/index.yaml.*sections\[0\].*subhed/);
});

it('checks links in the newly retained FAQ introduction', async () => {
  const contentDir = await fixture({
    'site.yaml': 'name: FAQ\n',
    'pages.yaml': '- name: home\n  sections: [faq]\n',
    'sections/faq.yaml': 'type: faq\neyebrow: Questions\nsubhead: "[More answers](/missing)"\nitems: [{question: Why?, answer: Because.}]\n',
  });
  expect((await check({ contentDir, strict: true })).ok).toBe(true);
  const result = await check({ contentDir, strict: true, deep: true });
  expect(result.ok).toBe(false);
  expect(result.warnings).toEqual([]);
  expect(result.errors.join('\n')).toMatch(/sections\/faq.yaml.*subhead.*no page resolves/s);
});

it('checks nested composition references with source paths and strict diagnostics', async () => {
  const section = { type: 'compose', version: 1, label: 'Composition', children: [{ type: 'group', children: [
    { type: 'image', image: 'assets/missing.svg', alt: 'Missing' },
    { type: 'button', label: 'Missing page', href: '/missing-compose-target' },
    { type: 'text', text: 'See [missing](/missing-inline-target)' },
  ] }] };
  const contentDir = await fixture({ 'site.yaml': 'name: Compose check\n',
    'pages.yaml': '- name: home\n  sections: [composition]\n', 'sections/composition.yaml': JSON.stringify(section) });
  const result = await check({ contentDir, deep: true, strict: true });
  expect(result.ok).toBe(false);
  const errors = result.errors.join('\n');
  expect(errors).toContain('assets/missing.svg');
  expect(errors).toContain('/missing-compose-target');
  expect(errors).toContain('/missing-inline-target');
  expect(errors).toContain('sections/composition.yaml');
});
