import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';

import { FileSystemContentSource } from './filesystem-source';
import { loadSite } from './site';

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((d) => rm(d, { recursive: true, force: true })));
});

interface Fixture {
  config: string;
  /** Content-relative path → file body. */
  files: Record<string, string>;
}

/** Materializes a fixture site on disk, returning a source and its paths. */
async function makeSiteAt(
  fixture: Fixture,
  onWarning?: (message: string) => void,
): Promise<{ source: FileSystemContentSource; contentDir: string; configFile: string }> {
  const root = await mkdtemp(path.join(tmpdir(), 'zellij-test-'));
  tempDirs.push(root);

  const contentDir = path.join(root, 'content');
  const configFile = path.join(root, 'zellij.config.yaml');
  await writeFile(configFile, fixture.config, 'utf8');

  for (const [rel, body] of Object.entries(fixture.files)) {
    const abs = path.join(contentDir, rel);
    await mkdir(path.dirname(abs), { recursive: true });
    await writeFile(abs, body, 'utf8');
  }

  const source = new FileSystemContentSource({
    contentDir,
    configFile,
    ...(onWarning ? { onWarning } : {}),
  });
  return { source, contentDir, configFile };
}

async function makeSite(fixture: Fixture): Promise<FileSystemContentSource> {
  return (await makeSiteAt(fixture)).source;
}

const MINIMAL_LANDING = `
title: Home
description: A landing page.
sections:
  - type: hero
    headline: Hello
`;

const BASE_CONFIG = `
product:
  name: Test
content: ./content
`;

describe('happy path', () => {
  it('loads a minimal valid site', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: { 'marketing/landing.yaml': MINIMAL_LANDING },
    });

    const site = await loadSite(source);
    expect(site.marketingPages).toHaveLength(1);
    expect(site.marketingPages[0]?.route).toBe('/');
    expect(site.config.theme).toBe('paper');
    expect(site.config.motion).toBe('subtle');
  });

  it('applies a route override', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'marketing/legal.yaml': `
title: Legal
description: Terms.
route: /terms-of-service
sections: []
`,
      },
    });

    const site = await loadSite(source);
    expect([...site.routes]).toContain('/terms-of-service');
    expect([...site.routes]).not.toContain('/legal');
  });
});

describe('build-time validation', () => {
  it('rejects an unknown section type and lists the valid ones', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': `
title: Home
description: A landing page.
sections:
  - type: carousel-of-wonders
    headline: Nope
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/unknown section type "carousel-of-wonders"/);
    await expect(loadSite(source)).rejects.toThrow(/Valid types:.*hero/s);
  });

  it('rejects a section missing a required field, naming file and path', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': `
title: Home
description: A landing page.
sections:
  - type: hero
    subhead: "no headline here"
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/marketing\/landing\.yaml/);
    await expect(loadSite(source)).rejects.toThrow(/headline/);
  });

  it('rejects malformed YAML with a readable message', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: { 'marketing/landing.yaml': 'title: "unterminated\nsections: [' },
    });

    await expect(loadSite(source)).rejects.toThrow(/Could not parse YAML/);
  });

  it('fails when no page resolves to "/"', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/features.yaml': `
title: Features
description: Features.
sections: []
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/No page resolves to "\/"/);
  });

  it('rejects duplicate routes', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'marketing/features.yaml': `
title: Features
description: Features.
sections: []
`,
        'marketing/other.yaml': `
title: Other
description: Other.
route: /features
sections: []
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/Duplicate marketing routes/);
  });
});

/**
 * Spec §5.5 calls link validation "the single most valuable guardrail in the
 * whole nav system", and §11.2a requires a fixture proving it fails the build.
 */
describe('nav link validation', () => {
  it('fails the build on a nav link to a nonexistent page, naming label and href', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
nav:
  links:
    - { label: Ghost, href: /does-not-exist }
`,
      files: { 'marketing/landing.yaml': MINIMAL_LANDING },
    });

    await expect(loadSite(source)).rejects.toThrow(/Broken navigation links/);
    await expect(loadSite(source)).rejects.toThrow(/Ghost/);
    await expect(loadSite(source)).rejects.toThrow(/\/does-not-exist/);
  });

  it('fails on an anchor that no section declares, and lists the available ones', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
nav:
  links:
    - { label: Security, href: /features#security }
`,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'marketing/features.yaml': `
title: Features
description: Features.
sections:
  - type: hero
    id: overview
    headline: Features
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/declares no section with id: security/);
    await expect(loadSite(source)).rejects.toThrow(/Available anchors: overview/);
  });

  it('accepts a valid anchor into a declared section id', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
nav:
  links:
    - { label: Security, href: /features#security }
`,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'marketing/features.yaml': `
title: Features
description: Features.
sections:
  - type: showcase
    id: security
    headline: Security
`,
      },
    });

    await expect(loadSite(source)).resolves.toBeDefined();
  });

  it('validates footer links with the same rules', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
footer:
  columns:
    - heading: Product
      links:
        - { label: Missing, href: /nowhere }
`,
      files: { 'marketing/landing.yaml': MINIMAL_LANDING },
    });

    await expect(loadSite(source)).rejects.toThrow(/Broken navigation links/);
    await expect(loadSite(source)).rejects.toThrow(/Missing/);
  });

  it('validates links nested in dropdowns and mega panels', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
nav:
  links:
    - label: Resources
      mega:
        groups:
          - heading: Learn
            items:
              - { label: Broken, href: /missing-guide }
`,
      files: { 'marketing/landing.yaml': MINIMAL_LANDING },
    });

    await expect(loadSite(source)).rejects.toThrow(/Broken/);
    await expect(loadSite(source)).rejects.toThrow(/Resources.*Learn/s);
  });

  it('leaves external URLs unvalidated', async () => {
    const source = await makeSite({
      config: `
product:
  name: Test
content: ./content
nav:
  links:
    - { label: GitHub, href: "https://github.com/shebka/acme" }
`,
      files: { 'marketing/landing.yaml': MINIMAL_LANDING },
    });

    await expect(loadSite(source)).resolves.toBeDefined();
  });
});

describe('guide sidebar', () => {
  it('errors when a sidebar entry has no matching file', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'guide/index.mdx': '# Guide\n',
        'guide/_sidebar.yaml': `
- label: Start
  items:
    - nonexistent-page
`,
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/references pages that do not exist/);
    await expect(loadSite(source)).rejects.toThrow(/nonexistent-page/);
  });

  it('warns (but still builds) when a guide page is missing from the sidebar', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'guide/index.mdx': '# Guide\n',
        'guide/orphan.mdx': '# Orphan\n',
        'guide/_sidebar.yaml': '[]',
      },
    });

    const site = await loadSite(source);
    expect(site.warnings.join('\n')).toMatch(/orphan\.mdx is not listed/);
    expect(site.guidePages.some((p) => p.route === '/guide/orphan')).toBe(true);
  });

  it('derives guide titles from frontmatter, then first heading', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'guide/index.mdx': '# Guide\n',
        'guide/a.mdx': '---\ntitle: From Frontmatter\n---\n# Ignored\n',
        'guide/b.mdx': '# From Heading\n',
      },
    });

    const site = await loadSite(source);
    const byRoute = new Map(site.guidePages.map((p) => [p.route, p.title]));
    expect(byRoute.get('/guide/a')).toBe('From Frontmatter');
    expect(byRoute.get('/guide/b')).toBe('From Heading');
  });
});

describe('forward compatibility', () => {
  it('warns on unknown top-level config keys instead of failing (spec §10)', async () => {
    const warnings: string[] = [];
    const { source } = await makeSiteAt(
      {
        config: `${BASE_CONFIG}\nfutureFeature:\n  enabled: true\n`,
        files: { 'marketing/landing.yaml': MINIMAL_LANDING },
      },
      (m) => warnings.push(m),
    );

    await expect(loadSite(source)).resolves.toBeDefined();
    expect(warnings.join('\n')).toMatch(/unknown config key "futureFeature"/);
  });
});

describe('route collisions', () => {
  it('rejects a guide page colliding with a marketing route', async () => {
    const source = await makeSite({
      config: BASE_CONFIG,
      files: {
        'marketing/landing.yaml': MINIMAL_LANDING,
        'marketing/clash.yaml': `
title: Clash
description: Clash.
route: /guide/quickstart
sections: []
`,
        'guide/quickstart.mdx': '# Quickstart\n',
      },
    });

    await expect(loadSite(source)).rejects.toThrow(/Route collision/);
  });
});
