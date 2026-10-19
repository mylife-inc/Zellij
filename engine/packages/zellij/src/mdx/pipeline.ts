import rehypeShiki from '@shikijs/rehype';
import { transformerMetaHighlight } from '@shikijs/transformers';
import type { ShikiTransformer } from 'shiki';
import rehypeSlug from 'rehype-slug';
import remarkGfm from 'remark-gfm';
import type { Pluggable } from 'unified';

import { rehypeDiagrams } from './rehype-mermaid';

import { loadMaths } from './maths';
import { featureEnabled } from '../../feature-config.cjs';

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
        [rehypeDiagrams, { mermaid: featureEnabled('diagrams') }],
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
