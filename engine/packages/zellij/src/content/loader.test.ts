import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';

import { FolderContentSource } from './folder-source';
import { LegacyContentSource } from './legacy-source';
import { loadSite, type Site } from './site';
import type { SidebarGroup } from './types';

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((d) => rm(d, { recursive: true, force: true })));
});

/** Content-relative path → file body. */
type Files = Record<string, string>;

async function materialize(files: Files, dirName = 'content'): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), 'zellij-test-'));
  tempDirs.push(root);

  const contentDir = path.join(root, dirName);
  await mkdir(contentDir, { recursive: true });

  for (const [rel, body] of Object.entries(files)) {
    const abs = path.join(contentDir, rel);
    await mkdir(path.dirname(abs), { recursive: true });
    await writeFile(abs, body, 'utf8');
  }

  return contentDir;
}

async function makeSource(
  files: Files,
  onWarning?: (message: string) => void,
): Promise<FolderContentSource> {
  const contentDir = await materialize(files);
  return new FolderContentSource({ contentDir, ...(onWarning ? { onWarning } : {}) });
}

async function load(files: Files): Promise<Site> {
  return loadSite(await makeSource(files));
}

async function loadWithWarnings(files: Files): Promise<{ site: Site; warnings: string[] }> {
  const warnings: string[] = [];
  const source = await makeSource(files, (message) => warnings.push(message));
  const site = await loadSite(source);
  return { site, warnings: [...warnings, ...site.warnings] };
}

const HERO = 'type: hero\nheadline: Hello\n';
const SITE = 'name: Test\n';
const ONE_PAGE = '- name: home\n  title: Home\n  description: A landing page.\n  sections: [hero]\n';

/* ------------------------------------------------------------- happy path */

describe('happy path', () => {
  it('loads a minimal site: site.yaml, pages.yaml and one section file', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
    });

    expect(site.config.name).toBe('Test');
    expect(site.pages).toHaveLength(1);
    expect(site.pages[0]?.route).toBe('/');
    expect(site.pages[0]?.sections[0]?.type).toBe('hero');
  });

  it('builds with no site.yaml, naming the site after its folder', async () => {
    const contentDir = await materialize(
      { 'chapter-one/_sidebar.yaml': '- intro\n', 'chapter-one/intro.mdx': '# Intro\n' },
      'my-handbook',
    );

    const site = await loadSite(new FolderContentSource({ contentDir }));

    expect(site.config.name).toBe('My handbook');
    expect(site.pages).toHaveLength(0);
    expect(site.guides).toHaveLength(1);
    expect(site.home?.route).toBe('/chapter-one');
  });

  it('honours an explicit route and leaves it there when it is home', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': '- name: home\n  route: /start\n  sections: [hero]\n',
      'sections/hero.yaml': HERO,
    });

    expect(site.pages[0]?.route).toBe('/start');
    expect(site.homeRedirect).toBe('/start');
    expect(site.routes.has('/')).toBe(true);
  });

  it('reads a markdown section as prose without a declared type', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': '- name: home\n  sections: [intro]\n',
      'sections/intro.md': '---\nbackground: tinted\n---\n\nSome prose.\n',
    });

    const section = site.pages[0]?.sections[0] as { type: string; body: string; background: string };
    expect(section.type).toBe('markdown-prose');
    expect(section.body).toBe('Some prose.');
    expect(section.background).toBe('tinted');
  });
});

/* --------------------------------------------------------------- sections */

describe('section references', () => {
  it('renders the same section twice with different presentation', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml':
        '- name: home\n  sections:\n' +
        '    - hero\n' +
        '    - { section: hero, id: second, background: inverted }\n',
      'sections/hero.yaml': HERO,
    });

    const [first, second] = site.pages[0]!.sections as Array<{
      headline: string;
      id?: string;
      background?: string;
    }>;
    expect(first?.headline).toBe('Hello');
    expect(second?.headline).toBe('Hello');
    expect(second?.id).toBe('second');
    expect(second?.background).toBe('inverted');
    // The point of the pair: an override on one reference must not reach the
    // other. `glow` is what a hero gets when nobody says otherwise, so the
    // first one keeping it is the assertion — it read `toBeUndefined()` when a
    // hero had no default ground, which tested the same thing by proxy.
    expect(first?.background).toBe('glow');
  });

  it('accepts a gradient background as an override', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml':
        '- name: home\n  sections:\n    - { section: hero, background: { gradient: [accent, "#101018"] } }\n',
      'sections/hero.yaml': HERO,
    });

    expect((site.pages[0]?.sections[0] as { background: unknown }).background).toEqual({
      gradient: ['accent', '#101018'],
    });
  });

  it('rejects a reference to a section file that does not exist, listing what does', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': '- name: home\n  sections: [missing]\n',
        'sections/hero.yaml': HERO,
      }),
    ).rejects.toThrow(/refers to "missing"[\s\S]*Available: hero/);
  });

  it('warns about a section file no page references', async () => {
    const { warnings } = await loadWithWarnings({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'sections/unused.yaml': HERO,
    });

    expect(warnings.join('\n')).toMatch(/sections\/unused\.yaml is not referenced/);
  });

  it('rejects an unknown section type and lists the valid ones', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': 'type: carousel\n',
      }),
    ).rejects.toThrow(/unknown section type "carousel"[\s\S]*Valid types:[\s\S]*hero/);
  });

  it('rejects a section missing a required field, naming the page and section', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': 'type: hero\n',
      }),
    ).rejects.toThrow(/pages\.yaml › home › hero[\s\S]*headline/);
  });

  it('rejects malformed YAML with a readable message', async () => {
    await expect(
      load({ 'site.yaml': SITE, 'pages.yaml': ONE_PAGE, 'sections/hero.yaml': 'type: [hero\n' }),
    ).rejects.toThrow(/Could not parse YAML in sections\/hero\.yaml/);
  });
});

/* ------------------------------------------------------------------ menus */

describe('menu', () => {
  const TWO_PAGES =
    '- name: home\n  sections: [hero]\n- name: pricing\n  title: Pricing\n  sections: [hero]\n';

  it('resolves page references to routes', async () => {
    const site = await load({
      // Without an explicit home, "/" would be the first menu item — which is
      // this very page, and the assertion below would be testing that instead.
      'site.yaml': 'name: Test\nhome: home\n',
      'menu.yaml': '- { name: Plans, page: pricing }\n',
      'pages.yaml': TWO_PAGES,
      'sections/hero.yaml': HERO,
    });

    expect(site.menu).toEqual([
      { label: 'Plans', href: '/pricing', external: false, newTab: false },
    ]);
  });

  it('marks an external url and opens it in a new tab', async () => {
    const site = await load({
      'site.yaml': SITE,
      'menu.yaml': '- { name: Blog, url: "https://example.com" }\n',
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
    });

    expect(site.menu[0]).toMatchObject({ external: true, newTab: true });
  });

  it('treats a child with its own items as a heading column', async () => {
    const site = await load({
      'site.yaml': SITE,
      'menu.yaml':
        '- name: More\n  items:\n    - name: Learn\n      items:\n        - { name: Plans, page: pricing }\n',
      'pages.yaml': TWO_PAGES,
      'sections/hero.yaml': HERO,
    });

    expect(site.menu[0]?.groups).toEqual([
      {
        heading: 'Learn',
        items: [{ label: 'Plans', href: '/pricing', external: false, newTab: false }],
      },
    ]);
  });

  it('rejects a page reference that names nothing', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'menu.yaml': '- { name: Ghost, page: nowhere }\n',
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': HERO,
      }),
    ).rejects.toThrow(/No page or guide folder is named "nowhere"/);
  });

  it('derives a menu from pages and guide folders when none is declared', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': TWO_PAGES,
      'sections/hero.yaml': HERO,
      'manual/_sidebar.yaml': '- intro\n',
      'manual/intro.mdx': '# Intro\n',
    });

    // The union rule: a guide folder absent from pages.yaml still gets in.
    expect(site.menu.map((item) => item.href)).toEqual(['/', '/pricing', '/manual']);
  });

  it('collapses menu items past the overflow threshold into one drawer', async () => {
    const pages = Array.from(
      { length: 5 },
      (_, i) => `- name: p${i}\n  title: P${i}\n  sections: [hero]\n`,
    ).join('');

    const site = await load({
      'site.yaml': 'name: Test\nmenuOverflowAfter: 3\n',
      'pages.yaml': pages,
      'sections/hero.yaml': HERO,
    });

    expect(site.menu).toHaveLength(4);
    expect(site.menu[3]?.label).toBe('More');
    expect(site.menu[3]?.children).toHaveLength(2);
  });
});

/* ------------------------------------------------------------------- home */

describe('home', () => {
  const PAGES = '- name: alpha\n  sections: [hero]\n- name: beta\n  sections: [hero]\n';

  it('defaults to the first page when nothing says otherwise', async () => {
    const site = await load({ 'site.yaml': SITE, 'pages.yaml': PAGES, 'sections/hero.yaml': HERO });
    expect(site.home?.name).toBe('alpha');
    expect(site.pages[0]?.route).toBe('/');
  });

  it('follows the first menu item that resolves to a page', async () => {
    const site = await load({
      'site.yaml': SITE,
      'menu.yaml': '- { name: Docs, url: "https://example.com" }\n- { name: Beta, page: beta }\n',
      'pages.yaml': PAGES,
      'sections/hero.yaml': HERO,
    });

    expect(site.home?.name).toBe('beta');
  });

  it('obeys an explicit home in site.yaml', async () => {
    const site = await load({
      'site.yaml': 'name: Test\nhome: beta\n',
      'pages.yaml': PAGES,
      'sections/hero.yaml': HERO,
    });

    expect(site.home?.name).toBe('beta');
  });

  it('rejects a home that names nothing', async () => {
    await expect(
      load({
        'site.yaml': 'name: Test\nhome: ghost\n',
        'pages.yaml': PAGES,
        'sections/hero.yaml': HERO,
      }),
    ).rejects.toThrow(/home "ghost" names no page or guide folder/);
  });

  it('redirects "/" to a guide folder rather than moving its pages', async () => {
    const site = await load({
      'site.yaml': 'name: Test\nhome: manual\n',
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'manual/_sidebar.yaml': '- intro\n',
      'manual/intro.mdx': '# Intro\n',
    });

    expect(site.homeRedirect).toBe('/manual');
    expect(site.guidePages[0]?.route).toBe('/manual/intro');
  });
});

/* ---------------------------------------------------------- guide folders */

describe('guide folders', () => {
  const TREE: Files = {
    'site.yaml': SITE,
    '02-advanced/_sidebar.yaml': '- tuning\n',
    '02-advanced/tuning.mdx': '# Tuning\n',
    '01-basics/_sidebar.yaml': 'title: The Basics\nnav:\n  - intro\n',
    '01-basics/intro.mdx': '# Intro\n',
  };

  it('strips the ordering prefix from routes and labels', async () => {
    const site = await load(TREE);

    expect(site.guides.map((guide) => guide.route)).toEqual(['/basics', '/advanced']);
    expect(site.guides.map((guide) => guide.title)).toEqual(['The Basics', 'Advanced']);
    expect(site.guidePages.map((page) => page.route)).toEqual(['/basics/intro', '/advanced/tuning']);
  });

  it('lets an explicit order beat the prefix', async () => {
    const site = await load({
      ...TREE,
      '01-basics/_sidebar.yaml': 'title: The Basics\norder: 9\nnav:\n  - intro\n',
    });

    expect(site.guides.map((guide) => guide.route)).toEqual(['/advanced', '/basics']);
  });

  it('mounts each tree at its own route and keeps sidebars separate', async () => {
    const site = await load(TREE);

    expect(site.guides[0]?.sidebar).toEqual([
      { kind: 'link', label: 'Intro', route: '/basics/intro' },
    ]);
    expect(site.guidePages[0]?.guide).toBe('/basics');
    expect(site.guidePages[1]?.guide).toBe('/advanced');
  });

  it('maps index.mdx to the tree root', async () => {
    const site = await load({
      'site.yaml': SITE,
      'manual/_sidebar.yaml': '- index\n- intro\n',
      'manual/index.mdx': '# Manual\n',
      'manual/intro.mdx': '# Intro\n',
    });

    expect(site.guidePages.map((page) => page.route)).toContain('/manual');
  });

  it('errors when a sidebar entry has no matching file', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'manual/_sidebar.yaml': '- intro\n- ghost\n',
        'manual/intro.mdx': '# Intro\n',
      }),
    ).rejects.toThrow(/manual\/_sidebar\.yaml references pages that do not exist/);
  });

  it('keeps groups nested inside groups', async () => {
    const site = await load({
      'site.yaml': SITE,
      'manual/_sidebar.yaml':
        '- label: Top\n  items:\n    - intro\n    - label: Deep\n      items:\n        - nested\n',
      'manual/intro.mdx': '# Intro\n',
      'manual/nested.mdx': '# Nested\n',
    });

    const top = site.guides[0]?.sidebar[0] as SidebarGroup;
    expect(top.kind).toBe('group');
    const deep = top.children[1] as SidebarGroup;
    expect(deep.label).toBe('Deep');
    expect(deep.children[0]).toEqual({ kind: 'link', label: 'Nested', route: '/manual/nested' });
  });

  it('omits a group with no resolvable entries rather than rendering an empty one', async () => {
    const site = await load({
      'site.yaml': SITE,
      'manual/_sidebar.yaml': '- intro\n- label: Empty\n  items: []\n',
      'manual/intro.mdx': '# Intro\n',
    });

    expect(site.guides[0]?.sidebar).toHaveLength(1);
  });

  it('warns (but still builds) when a page is missing from the sidebar', async () => {
    const { site, warnings } = await loadWithWarnings({
      'site.yaml': SITE,
      'manual/_sidebar.yaml': '- intro\n',
      'manual/intro.mdx': '# Intro\n',
      'manual/orphan.mdx': '# Orphan\n',
    });

    expect(site.guidePages).toHaveLength(2);
    expect(warnings.join('\n')).toMatch(/manual\/orphan\.mdx is not listed/);
  });

  it('derives titles from frontmatter, then the first heading, then the filename', async () => {
    const site = await load({
      'site.yaml': SITE,
      'manual/_sidebar.yaml': '- a\n- b\n- c\n',
      'manual/a.mdx': '---\ntitle: From frontmatter\n---\n\n# Ignored\n',
      'manual/b.mdx': '# From heading\n',
      'manual/c.mdx': 'No heading at all.\n',
    });

    expect(site.guidePages.map((page) => page.title)).toEqual([
      'From frontmatter',
      'From heading',
      'C',
    ]);
  });

  it('warns about a folder with no _sidebar.yaml instead of silently ignoring it', async () => {
    const { warnings } = await loadWithWarnings({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'notes/scratch.mdx': '# Scratch\n',
    });

    expect(warnings.join('\n')).toMatch(/notes\/ has no _sidebar\.yaml/);
  });
});

/* ------------------------------------------------------------- collisions */

describe('collisions', () => {
  it('rejects a page and a guide folder sharing a name', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': '- name: manual\n  sections: [hero]\n',
        'sections/hero.yaml': HERO,
        'manual/_sidebar.yaml': '- intro\n',
        'manual/intro.mdx': '# Intro\n',
      }),
    ).rejects.toThrow(/Duplicate page names/);
  });

  it('rejects two destinations claiming the same route', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml':
          '- name: home\n  sections: [hero]\n- name: other\n  route: /manual/intro\n  sections: [hero]\n',
        'sections/hero.yaml': HERO,
        'manual/_sidebar.yaml': '- intro\n',
        'manual/intro.mdx': '# Intro\n',
      }),
    ).rejects.toThrow(/Route collisions[\s\S]*\/manual\/intro/);
  });
});

/* ---------------------------------------------------------- link checking */

describe('link validation', () => {
  const base = (extra: string): Files => ({
    'site.yaml': `name: Test\n${extra}`,
    'pages.yaml': '- name: home\n  sections: [hero]\n',
    'sections/hero.yaml': 'type: hero\nheadline: Hello\nid: top\n',
  });

  it('fails on a CTA pointing at a nonexistent page', async () => {
    await expect(load(base('nav:\n  cta:\n    - { label: Go, href: /nope }\n'))).rejects.toThrow(
      /No page resolves to "\/nope"/,
    );
  });

  it('fails on an anchor no section declares, and lists the available ones', async () => {
    await expect(load(base('nav:\n  cta:\n    - { label: Go, href: "/#ghost" }\n'))).rejects.toThrow(
      /declares no section with id: ghost[\s\S]*Available anchors: top/,
    );
  });

  it('accepts a valid anchor into a declared section id', async () => {
    const site = await load(base('nav:\n  cta:\n    - { label: Go, href: "/#top" }\n'));
    expect(site.anchors.get('/')?.has('top')).toBe(true);
  });

  it('resolves footer page references', async () => {
    const site = await load(
      base(
        'footer:\n  columns:\n    - heading: More\n      links:\n        - { label: Home, page: home }\n',
      ),
    );
    expect(site.footer[0]?.links[0]?.href).toBe('/');
  });

  it('rejects a footer page reference that names nothing', async () => {
    await expect(
      load(
        base(
          'footer:\n  columns:\n    - heading: More\n      links:\n        - { label: Ghost, page: nope }\n',
        ),
      ),
    ).rejects.toThrow(/No page or guide folder is named "nope"/);
  });

  it('leaves external URLs unvalidated', async () => {
    const site = await load(
      base('nav:\n  cta:\n    - { label: Docs, href: "https://example.com/nope" }\n'),
    );
    expect(site.config.nav.cta[0]?.href).toBe('https://example.com/nope');
  });
});

/* --------------------------------------------------------- forward compat */

describe('forward compatibility', () => {
  it('warns on unknown site.yaml keys instead of failing (spec §10)', async () => {
    const { site, warnings } = await loadWithWarnings({
      'site.yaml': 'name: Test\nfutureThing: true\n',
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
    });

    expect(site.config.name).toBe('Test');
    expect(warnings.join('\n')).toMatch(/unknown key "futureThing"/);
  });

  it('passes unknown keys inside a section file through untouched', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': 'type: hero\nheadline: Hello\nfutureField: 1\n',
    });

    expect(site.pages[0]?.sections[0]).toMatchObject({ headline: 'Hello' });
  });
});

/* --------------------------------------------------------- legacy layout */

describe('legacy layout', () => {
  it('reads marketing/ and guide/ into the current model', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'zellij-legacy-'));
    tempDirs.push(root);

    const contentDir = path.join(root, 'content');
    const configFile = path.join(root, 'zellij.config.yaml');
    await writeFile(
      configFile,
      'product:\n  name: Legacy\n  logo: assets/logo.svg\n' +
        'nav:\n  links:\n    - { label: Docs, href: /guide }\n',
      'utf8',
    );

    const files: Files = {
      'marketing/landing.yaml':
        'title: Home\ndescription: d\nsections:\n  - type: hero\n    headline: Hi\n',
      'guide/_sidebar.yaml': '- intro\n',
      'guide/intro.mdx': '# Intro\n',
    };
    for (const [rel, body] of Object.entries(files)) {
      const abs = path.join(contentDir, rel);
      await mkdir(path.dirname(abs), { recursive: true });
      await writeFile(abs, body, 'utf8');
    }

    const site = await loadSite(new LegacyContentSource({ contentDir, configFile }));

    expect(site.config.name).toBe('Legacy');
    expect(site.config.brand.logo?.src).toBe('assets/logo.svg');
    expect(site.pages[0]?.route).toBe('/');
    expect(site.guides[0]?.route).toBe('/guide');
    expect(site.guidePages[0]?.route).toBe('/guide/intro');
    expect(site.menu[0]).toMatchObject({ label: 'Docs', href: '/guide', external: false });
  });
});

/* --------------------------------------------------------------- new blocks */

describe('media-panel', () => {
  const panel = (extra: string): Files => ({
    'site.yaml': SITE,
    'pages.yaml': '- name: home\n  sections: [panel]\n',
    'sections/panel.yaml': `type: media-panel\n${extra}`,
  });

  it('accepts an image', async () => {
    const site = await load(panel('image: assets/a.png\ntitle: Hello\n'));
    expect(site.pages[0]?.sections[0]).toMatchObject({ type: 'media-panel', ratio: 'wide' });
  });

  it('rejects a panel with neither image nor video', async () => {
    await expect(load(panel('title: Hello\n'))).rejects.toThrow(
      /exactly one of: image, video/,
    );
  });

  it('rejects a panel declaring both', async () => {
    await expect(
      load(panel('image: assets/a.png\nvideo: assets/a.mp4\n')),
    ).rejects.toThrow(/exactly one of: image, video/);
  });

  it('rejects a scrim outside 0–1', async () => {
    await expect(load(panel('image: assets/a.png\nscrim: 2\n'))).rejects.toThrow(/scrim/);
  });
});

describe('hero-split', () => {
  const split = (extra: string): Files => ({
    'site.yaml': SITE,
    'pages.yaml': '- name: home\n  sections: [opener]\n',
    'sections/opener.yaml': `type: hero-split\nheadline: Ship it\n${extra}`,
  });

  it('accepts a terminal transcript', async () => {
    const site = await load(split('terminal:\n  body: "$ zel dev"\n'));
    expect(site.pages[0]?.sections[0]).toMatchObject({
      type: 'hero-split',
      media: 'right',
      // The bar is labelled even when the author does not say so.
      terminal: { title: 'terminal', body: '$ zel dev' },
    });
  });

  it('accepts an image instead', async () => {
    const site = await load(split('visual: assets/a.png\nmedia: left\n'));
    expect(site.pages[0]?.sections[0]).toMatchObject({ type: 'hero-split', media: 'left' });
  });

  it('rejects declaring neither', async () => {
    await expect(load(split(''))).rejects.toThrow(/exactly one of: visual, terminal/);
  });

  it('rejects declaring both', async () => {
    await expect(
      load(split('visual: assets/a.png\nterminal:\n  body: "$ zel dev"\n')),
    ).rejects.toThrow(/exactly one of: visual, terminal/);
  });

  it('caps highlights at four', async () => {
    const five = ['a', 'b', 'c', 'd', 'e'].map((l) => `    - label: ${l}`).join('\n');
    await expect(
      load(split(`terminal:\n  body: "$ x"\nhighlights:\n${five.replace(/^ {4}/gm, '  ')}\n`)),
    ).rejects.toThrow(/highlights/);
  });

  it('renders the page heading, like hero', async () => {
    const site = await load(split('terminal:\n  body: "$ zel dev"\n'));
    expect(site.pages[0]?.sections[0]?.type).toBe('hero-split');
  });
});

describe('card-carousel', () => {
  it('needs at least two cards to be a carousel', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': '- name: home\n  sections: [row]\n',
        'sections/row.yaml': 'type: card-carousel\nitems:\n  - { title: Only one }\n',
      }),
    ).rejects.toThrow(/items/);
  });

  it('defaults the card width', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': '- name: home\n  sections: [row]\n',
      'sections/row.yaml': 'type: card-carousel\nitems:\n  - { title: A }\n  - { title: B }\n',
    });
    expect(site.pages[0]?.sections[0]).toMatchObject({ cardWidth: 'medium' });
  });
});

describe('titleGradient', () => {
  it('is a presentation override a page may apply', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml':
        '- name: home\n  sections:\n    - { section: hero, titleGradient: [accent, fg] }\n',
      'sections/hero.yaml': HERO,
    });
    expect(site.pages[0]?.sections[0]).toMatchObject({ titleGradient: ['accent', 'fg'] });
  });

  it('rejects a single stop, which is not a gradient', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': `${HERO}titleGradient: [accent]\n`,
      }),
    ).rejects.toThrow(/titleGradient/);
  });
});

describe('numeric text fields', () => {
  it('accepts a bare year on a timeline, which YAML hands over as a number', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': '- name: home\n  sections: [story]\n',
      'sections/story.yaml':
        'type: timeline\nitems:\n  - { date: 2019, title: First }\n  - { date: 2026, title: Now }\n',
    });

    const section = site.pages[0]?.sections[0] as { items: Array<{ date: string }> };
    expect(section.items[0]?.date).toBe('2019');
  });

  it('accepts a bare number as a stat value', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': '- name: home\n  sections: [stats]\n',
      'sections/stats.yaml':
        'type: stats-band\nstats:\n  - { value: 99.9, label: Uptime }\n' +
        '  - { value: 12, label: Regions }\n  - { value: 4, label: Clouds }\n',
    });

    const section = site.pages[0]?.sections[0] as { stats: Array<{ value: string }> };
    expect(section.stats[0]?.value).toBe('99.9');
  });
});

describe('anchor uniqueness', () => {
  it('rejects two sections on one page claiming the same anchor', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml':
          '- name: home\n  sections:\n    - hero\n    - { section: hero, id: top }\n',
        'sections/hero.yaml': `${HERO}id: top\n`,
      }),
    ).rejects.toThrow(/two sections with id: top/);
  });
});

/* ------------------------------------------------------ documentation examples */

describe('examples/', () => {
  it('loads each file, keyed by its name, with the source kept verbatim', async () => {
    const source = 'type: hero\nheadline: Ship it\n# a trailing comment\n';
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'examples/hero-basic.yaml': source,
    });

    const example = site.examples.get('hero-basic');
    expect(example).toBeDefined();
    // Verbatim, comments included: the source tab shows the file, not a
    // re-serialization of what the parser made of it.
    expect(example?.source).toBe(source);
    expect(example?.section).toMatchObject({ type: 'hero', headline: 'Ship it' });
  });

  it('names nested examples by their path below examples/', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'examples/openers/hero.yaml': HERO,
    });

    expect([...site.examples.keys()]).toEqual(['openers/hero']);
  });

  it('fails the build when an example does not validate', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': HERO,
        // A documented example that the engine would reject is worse than no
        // example: it shows readers something that cannot work.
        'examples/broken.yaml': 'type: hero\n',
      }),
    ).rejects.toThrow(/examples\/broken\.yaml/);
  });

  it('fails on an unknown section type, naming the file', async () => {
    await expect(
      load({
        'site.yaml': SITE,
        'pages.yaml': ONE_PAGE,
        'sections/hero.yaml': HERO,
        'examples/nope.yaml': 'type: carousel\n',
      }),
    ).rejects.toThrow(/examples\/nope\.yaml/);
  });

  it('is empty, not absent, when the folder does not exist', async () => {
    const site = await load({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
    });

    expect(site.examples.size).toBe(0);
  });

  it('does not treat examples/ as a guide folder', async () => {
    const { site, warnings } = await loadWithWarnings({
      'site.yaml': SITE,
      'pages.yaml': ONE_PAGE,
      'sections/hero.yaml': HERO,
      'examples/hero.yaml': HERO,
    });

    expect(site.guides).toEqual([]);
    expect(warnings.filter((w) => w.includes('examples/'))).toEqual([]);
  });
});
