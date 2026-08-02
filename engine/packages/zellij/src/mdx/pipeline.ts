import rehypeShiki from '@shikijs/rehype';
import { transformerMetaHighlight } from '@shikijs/transformers';
import type { ShikiTransformer } from 'shiki';
import rehypeSlug from 'rehype-slug';
import remarkGfm from 'remark-gfm';
import type { Pluggable } from 'unified';

import { rehypeMermaid } from './rehype-mermaid';

/**
 * Mathematics is optional, and costs about fifty megabytes to install.
 *
 * A documentation site with no formulas in it should not carry MathJax and its
 * speech-rule engine around, so the packages are optional dependencies and this
 * is where their absence is handled. Installed, `$…$` is typeset. Not installed,
 * it stays as written — which is what a site that never wanted maths expects
 * anyway, and is a great deal better than failing to build over a package
 * nobody asked for.
 *
 * Resolved once at module load rather than per page: the answer cannot change
 * while the process is running, and a `try` around every compile would pay for
 * asking a question that has one answer.
 */
let maths: Pluggable[][] | undefined;

/**
 * Resolved once and cached. Awaited rather than resolved at module load
 * because the package is built for CommonJS as well as ESM, and top-level
 * await cannot be expressed in CommonJS at all.
 */
async function loadMaths(): Promise<Pluggable[][]> {
  if (maths) return maths;
  maths = await resolveMaths();
  return maths;
}

async function resolveMaths(): Promise<Pluggable[][]> {
  try {
    const [{ default: remarkMath }, { default: rehypeMathjax }] = await Promise.all([
      import('remark-math'),
      import('rehype-mathjax'),
    ]);

    return [
      [remarkMath as Pluggable],
      // Display maths is left-aligned to sit with the prose rather than centred
      // over it. Width is handled in CSS.
      [[rehypeMathjax, { svg: { displayAlign: 'left' } }] as Pluggable],
    ];
  } catch {
    return [[], []];
  }
}

/** True when this build can typeset mathematics. `zel doctor` reports it. */
export async function supportsMaths(): Promise<boolean> {
  return (await loadMaths())[0]!.length > 0;
}

/**
 * Shiki themes are chosen per mode; CSS variables select which one is visible,
 * so a theme toggle needs no re-highlighting (spec §13.5).
 */
const SHIKI_THEMES = { light: 'github-light', dark: 'github-dark' } as const;

/**
 * Pulls `title="next.config.js"` out of a fence's meta string and promotes it
 * to a data attribute, so <CodeBlock> can render a filename label and
 * <CodeGroup> can use it as a tab label (spec §7.2).
 */
function parseMetaString(meta: string): Record<string, string> | undefined {
  const properties: Record<string, string> = {};

  const title = /(?:title|filename)=(?:"([^"]*)"|'([^']*)'|(\S+))/.exec(meta);
  if (title) {
    const value = title[1] ?? title[2] ?? title[3];
    if (value) properties['data-title'] = value;
  }

  return Object.keys(properties).length > 0 ? properties : undefined;
}

/**
 * Stamps the fence language onto the <pre>, so <CodeBlock> can label an
 * untitled block and <CodeGroup> has a tab label fallback. Shiki does not
 * expose the language as an attribute on its own.
 */
const transformerLanguageAttribute: ShikiTransformer = {
  name: 'zellij:language-attribute',
  pre(node) {
    const lang = this.options.lang;
    if (lang && lang !== 'text' && lang !== 'plaintext') {
      node.properties['data-lang'] = lang;
    }
  },
};

/**
 * The single MDX compile configuration, shared by guide pages and the
 * marketing `markdown-prose` section so there is one pipeline, not two.
 */
export async function mdxOptions() {
  const [remarkMaths, rehypeMaths] = await loadMaths();

  return {
    parseFrontmatter: false,
    mdxOptions: {
      // `remarkMath` reads $…$ and $$…$$ into maths nodes; `rehypeMathjax`
      // renders them. Rendering happens here, at build time, so a page of
      // theorems ships as SVG with no maths library, no stylesheet and no web
      // fonts behind it — and the glyphs inherit `currentColor`, so a formula
      // follows the theme into dark mode like any other text.
      remarkPlugins: [remarkGfm, ...remarkMaths!] as Pluggable[],
      rehypePlugins: [
        rehypeSlug,
        rehypeMermaid,
        ...rehypeMaths!,
        [
          rehypeShiki,
          {
            themes: SHIKI_THEMES,
            defaultColor: false,
            // Enables ```ts {2,4-6} line highlighting (spec §7.2).
            transformers: [transformerMetaHighlight(), transformerLanguageAttribute],
            parseMetaString,
          },
        ],
      ] as Pluggable[],
    },
  };
}
