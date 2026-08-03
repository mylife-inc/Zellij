import { access, cp, mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/*
 * Self-contained on purpose, like the other CLI entry points: `bin/zellij.mjs`
 * runs this file as TypeScript, and Node's ESM resolver will not follow the
 * extensionless relative imports the rest of the engine is written with.
 */

async function exists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}

export interface InitOptions {
  /** Where to write the app. Created if missing. */
  target: string;
  /** An existing content folder to copy in. A starter folder is written when absent. */
  content?: string;
  /** Package name. Defaults to the target directory's name. */
  name?: string;
  /** Dev and start port. */
  port?: number;
  /** Write into a directory that already has files in it. */
  force?: boolean;
  /**
   * Optional engine features to install: `maths`, `diagrams`, or `all`.
   *
   * These are the expensive ones — roughly fifty megabytes for MathJax and a
   * hundred for Mermaid — and most documentation wants neither. They are
   * optional dependencies of the engine, so leaving them out of the manifest
   * means they are never fetched, and the engine renders without them rather
   * than failing over a package nobody asked for.
   */
  features?: string[];
  /**
   * Write only the content folder, with no app around it.
   *
   * For a project that wants documentation in its own repository and will build
   * it elsewhere — in CI, or from the base image — where a Next.js app checked
   * in beside the source would be scaffolding nobody maintains.
   */
  contentOnly?: boolean;
}

/** The engine's own package root, found from this file rather than from the caller's cwd. */
function engineRoot(): string {
  return path.resolve(fileURLToPath(import.meta.url), '..', '..', '..');
}

async function enginePackage(): Promise<{ version: string; peers: Record<string, string> }> {
  const raw = await readFile(path.join(engineRoot(), 'package.json'), 'utf8');
  const parsed = JSON.parse(raw) as {
    version: string;
    peerDependencies?: Record<string, string>;
  };
  return { version: parsed.version, peers: parsed.peerDependencies ?? {} };
}

/**
 * Turns a peer range into something a fresh app can install.
 *
 * The engine pins its peers to a minor line — `16.2.x` — which npm accepts, but
 * a generated app reads better with a caret so it picks up patches on its own.
 */
function installable(range: string): string {
  const minor = /^(\d+)\.(\d+)\.x$/.exec(range);
  return minor ? `^${minor[1]}.${minor[2]}.0` : range;
}

/*
 * `install-links` makes npm copy a `file:` dependency into node_modules instead
 * of symlinking it. Without it the engine stays a link pointing outside the
 * app, and Turbopack will not compile sources it cannot see beneath the project
 * root — the app installs cleanly and then fails to resolve `@shebka/zellij`,
 * which reads like a missing package rather than a link it declined to follow.
 */
const NPMRC = `install-links=true
`;

/*
 * The engine's stylesheet opens with `@import 'tailwindcss'`, so a consuming
 * app needs Tailwind's PostCSS plugin wired up before any Zellij CSS resolves.
 * That is the engine's requirement rather than the site's, so the scaffold
 * supplies it — without this the app installs, then fails its first build on a
 * missing module the author never asked for.
 */
const POSTCSS_CONFIG = `const config = { plugins: { '@tailwindcss/postcss': {} } };

export default config;
`;

const GITIGNORE = `node_modules/
.next/
out/
*.tsbuildinfo
.DS_Store

# Written by \`zellij prepare\` from the content folder.
public/_zellij/
`;

const TSCONFIG = {
  compilerOptions: {
    target: 'ES2022',
    lib: ['DOM', 'DOM.Iterable', 'ES2022'],
    jsx: 'preserve',
    module: 'ESNext',
    moduleResolution: 'bundler',
    strict: true,
    noEmit: true,
    allowJs: true,
    esModuleInterop: true,
    resolveJsonModule: true,
    isolatedModules: true,
    incremental: true,
    skipLibCheck: true,
    plugins: [{ name: 'next' }],
    paths: { '@/*': ['./src/*'] },
  },
  include: ['next-env.d.ts', '**/*.ts', '**/*.tsx', '.next/types/**/*.ts'],
  exclude: ['node_modules'],
};

const LAYOUT = `import '@shebka/zellij/styles.css';
import { createZellijRootLayout } from '@shebka/zellij';

export default createZellijRootLayout();
`;

const PAGE = `import { createZellijPages } from '@shebka/zellij';

const zellij = createZellijPages();

export const generateStaticParams = zellij.generateStaticParams;
export const generateMetadata = zellij.generateMetadata;
export default zellij.Page;
`;

/*
 * A documentation site nobody can find is a documentation site nobody reads,
 * and both of these are one line. Leaving them to the author meant every site
 * built from the container image shipped without them — the scaffold is the
 * only place that knows they should exist.
 */
const SITEMAP = `import { createZellijSitemap } from '@shebka/zellij';

// A metadata route is dynamic by default, and \`output: export\` has nowhere to
// run it — without this, asking for a static export fails on this file.
export const dynamic = 'force-static';

export default createZellijSitemap();
`;

const ROBOTS = `import { createZellijRobots } from '@shebka/zellij';

export const dynamic = 'force-static';

export default createZellijRobots();
`;

const NEXT_CONFIG = `import { withZellij } from '@shebka/zellij/next-config';

/**
 * Everything Zellij needs is in \`withZellij\`. Add your own Next options here
 * and they win — the helper only fills in what it must.
 *
 * @type {import('next').NextConfig}
 */
export default withZellij({
  typedRoutes: false,
});
`;

/** A content folder that builds on the first run, so a new app is never a blank page. */
function starterContent(name: string): Record<string, string> {
  return {
    'site.yaml': `name: ${name}
tagline: Built with Zellij.

theme: cupertino
mode: system
motion: subtle

home: home

nav:
  sticky: true
  search: true
  themeToggle: true

footer:
  columns:
    - heading: Docs
      links:
        - { label: Guide, page: guide }
  legal: "© ${name}"
`,

    'menu.yaml': `- { name: Guide, page: guide }
`,

    'pages.yaml': `- name: home
  title: ${name}
  description: A site built from a folder of content.
  layout: landing
  sections:
    - home/hero
`,

    'sections/home/hero.yaml': `type: hero
headline: ${name}
subhead: Edit content/sections/home/hero.yaml and this changes.
cta:
  - { label: Read the guide, href: /guide, style: primary }
`,

    'guide/_sidebar.yaml': `title: Guide
description: How this site is put together.

nav:
  - index
`,

    'guide/index.mdx': `---
title: Guide
description: Start here.
---

## You are editing content, not code

This page is \`content/guide/index.mdx\`. Add another \`.mdx\` file beside it and
list it in \`_sidebar.yaml\` to make it appear in the sidebar.

<Callout type="tip" title="Check before you build">
\`npx zellij check\` validates the whole content folder and names the file behind
any problem it finds.
</Callout>
`,
  };
}


/**
 * The packages each optional feature needs.
 *
 * One definition, used both when an app is generated and when an existing one
 * is retuned. A second copy — in a Dockerfile, say, where it would be three
 * lines of shell — is a copy that disagrees the first time a version moves.
 */
const FEATURE_PACKAGES: Record<string, Record<string, string>> = {
  maths: { 'remark-math': '^6.0.0', 'rehype-mathjax': '^7.1.0' },
  diagrams: { mermaid: '^11.16.0' },
};

export const FEATURE_NAMES = Object.keys(FEATURE_PACKAGES);

function featureDependencies(features: readonly string[]): Record<string, string> {
  // `all` is spelled out rather than passed through, so the manifest records
  // what was actually asked for and still reads clearly a year later.
  const wanted = new Set(
    features.flatMap((feature) => (feature === 'all' ? FEATURE_NAMES : [feature])),
  );

  const unknown = [...wanted].filter((feature) => !(feature in FEATURE_PACKAGES));
  if (unknown.length > 0) {
    throw new Error(
      `Unknown feature(s): ${unknown.join(', ')}\n  Available: ${FEATURE_NAMES.join(', ')}, all`,
    );
  }

  return Object.assign({}, ...[...wanted].map((feature) => FEATURE_PACKAGES[feature]));
}

/**
 * Turns optional features on in an app that already exists.
 *
 * This is what lets one base image serve every combination: the published
 * variants have their features baked in, and anyone using the plain one can
 * ask for maths or diagrams at the moment they build, before anything is
 * installed. Editing the manifest is all it takes, because the features are
 * only ever a question of which packages get fetched.
 */
export async function setFeatures(target: string, features: readonly string[]): Promise<string[]> {
  const manifest = path.join(path.resolve(target), 'package.json');
  const parsed = JSON.parse(await readFile(manifest, 'utf8')) as {
    dependencies?: Record<string, string>;
  };

  const wanted = featureDependencies(features);

  // Every optional package is cleared first, so turning a feature off works as
  // well as turning one on — otherwise the only way back would be a fresh app.
  const all = Object.assign({}, ...Object.values(FEATURE_PACKAGES)) as Record<string, string>;
  const dependencies = { ...parsed.dependencies };
  for (const name of Object.keys(all)) delete dependencies[name];

  parsed.dependencies = { ...dependencies, ...wanted };
  await writeFile(manifest, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8');

  return Object.keys(wanted);
}

/**
 * Generates a Next.js app around a content folder.
 *
 * The wrapper is deliberately thin — a layout, a catch-all route and a config —
 * because everything that makes the site is in the content folder, and an app
 * that carries logic of its own is an app that has to be maintained. It sits in
 * `src/`, and the content stays outside it at the app root: the wrapper and the
 * content it consumes are separate things and the layout should say so.
 */
export async function init(options: InitOptions): Promise<string> {
  const target = path.resolve(options.target);
  const name = options.name ?? path.basename(target);
  const port = options.port ?? 3000;

  if (await exists(target)) {
    const entries = (await readdir(target)).filter((entry) => entry !== '.DS_Store');
    if (entries.length > 0 && !options.force) {
      throw new Error(
        `${target} is not empty.\n` +
          `  Pass --force to write into it anyway, or choose another directory.`,
      );
    }
  }

  if (options.contentOnly) {
    const root = options.content ? target : path.join(target);
    for (const [rel, body] of Object.entries(starterContent(name))) {
      const file = path.join(root, rel);
      await mkdir(path.dirname(file), { recursive: true });
      await writeFile(file, body, 'utf8');
    }
    return root;
  }

  const { version, peers } = await enginePackage();

  const dependencies: Record<string, string> = {
    /*
     * A path dependency, because the engine is not published to a public
     * registry: whoever runs this has the source on disk, and that clone is
     * what the app installs from.
     *
     * Absolute rather than relative. A generated app rarely sits beside the
     * clone, and `file:../../Users/you/...` — which is what a relative path
     * turns into once it has climbed to the filesystem root — is both unreadable
     * and silently wrong the moment the app is moved. If the clone itself moves,
     * this line is the one thing to update.
     */
    '@shebka/zellij': `file:${engineRoot()}`,
  };
  for (const [pkg, range] of Object.entries(peers)) {
    dependencies[pkg] = installable(range);
  }

  const optional = featureDependencies(options.features ?? []);

  const packageJson = {
    name,
    version: '0.1.0',
    private: true,
    type: 'module',
    engines: { node: '>=20.11' },
    scripts: {
      // `prepare` syncs content assets into public/ and writes the search
      // index. Both are derived from the content folder, so they run before
      // every dev session and every build rather than being committed.
      predev: 'zellij prepare',
      dev: `next dev --port ${port}`,
      prebuild: 'zellij prepare',
      build: 'next build',
      start: `next start --port ${port}`,
      check: 'zellij check',
      typecheck: 'tsc --noEmit',
    },
    dependencies: { ...dependencies, ...optional },
    devDependencies: {
      '@tailwindcss/postcss': '^4.3.3',
      tailwindcss: '^4.3.3',
      '@types/node': '^22.10.2',
      '@types/react': '^19.2.0',
      '@types/react-dom': '^19.2.0',
      typescript: '^5.9.3',
    },
  };

  const files: Record<string, string> = {
    'package.json': `${JSON.stringify(packageJson, null, 2)}\n`,
    'tsconfig.json': `${JSON.stringify(TSCONFIG, null, 2)}\n`,
    'next.config.js': NEXT_CONFIG,
    '.gitignore': GITIGNORE,
    '.npmrc': NPMRC,
    'postcss.config.mjs': POSTCSS_CONFIG,
    'src/app/layout.tsx': LAYOUT,
    'src/app/[[...slug]]/page.tsx': PAGE,
    'src/app/sitemap.ts': SITEMAP,
    'src/app/robots.ts': ROBOTS,
    'README.md': readme(name, port, version),
  };

  for (const [rel, body] of Object.entries(files)) {
    const file = path.join(target, rel);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, body, 'utf8');
  }

  const contentDir = path.join(target, 'content');

  if (options.content) {
    const source = path.resolve(options.content);
    if (!(await exists(source))) throw new Error(`No content folder at ${source}`);
    await cp(source, contentDir, {
      recursive: true,
      filter: (from) => !/(^|[\\/])(node_modules|\.git|\.next)([\\/]|$)/.test(from),
    });
  } else {
    for (const [rel, body] of Object.entries(starterContent(name))) {
      const file = path.join(contentDir, rel);
      await mkdir(path.dirname(file), { recursive: true });
      await writeFile(file, body, 'utf8');
    }
  }

  return target;
}

function readme(name: string, port: number, version: string): string {
  return `# ${name}

A [Zellij](https://github.com/mylife-inc/Zellij) site. Everything that makes it
is in \`content/\`; \`src/\` is the Next.js app that renders it, and there is
almost nothing in there.

    ${name}/
    ├── content/          the site: config, menu, pages, sections, guides
    └── src/app/          the wrapper — a layout and one catch-all route

## Running it

    npm install
    npm run dev          # http://localhost:${port}

## Commands

| Command | What it does |
| --- | --- |
| \`npm run dev\` | Development server |
| \`npm run build\` | Static production build |
| \`npm run check\` | Validate \`content/\` without building |

Generated by \`zellij init\` (engine ${version}).
`;
}
