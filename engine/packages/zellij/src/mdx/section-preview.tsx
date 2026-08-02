import { MDXRemote } from 'next-mdx-remote/rsc';

import type { Site } from '../content/site';
import { ZellijContentError } from '../errors';
import { SectionRenderer } from '../render/section-renderer';
import { CodeBlock } from './code-block';
import { mdxOptions } from './pipeline';
import { TabsClient } from './tabs-client';

export interface SectionPreviewProps {
  /** Name of a file under `examples/`, without the extension. */
  example: string;
  /** Overrides the label above the frame. Defaults to the section's type. */
  label?: string;
}

/**
 * Shows a documentation example twice: what an author writes, and what it
 * renders to.
 *
 * Both tabs read the same file. The preview is the real section, dispatched
 * through the same renderer a page uses, so it cannot show anything a page
 * would not — and the source is the file verbatim rather than a transcription
 * of it. Documenting a section this way makes the example build-verified: if it
 * stops parsing, or the section it names stops rendering, the docs site fails
 * to build instead of describing something that no longer works.
 */
export function SectionPreview({
  example,
  label,
  site,
}: SectionPreviewProps & { site: Site }) {
  const found = site.examples.get(example);

  if (!found) {
    const known = [...site.examples.keys()].sort();
    throw new ZellijContentError(
      `<SectionPreview example="${example}"> refers to no file under examples/.\n` +
        (known.length > 0
          ? `  Available: ${known.join(', ')}`
          : '  No examples/ directory was found in this content root.'),
    );
  }

  const source = (
    <MDXRemote
      source={['```yaml', found.source.trimEnd(), '```'].join('\n')}
      components={{ pre: CodeBlock }}
      options={mdxOptions()}
    />
  );

  const preview = (
    // The frame is a viewport, not a page: the section inside it is full-width
    // by design and would otherwise bleed to the edges of the guide column.
    <div className="zellij-preview-frame">
      <SectionRenderer
        section={found.section}
        site={site}
        // Examples never animate. A reveal that waits for a scroll would leave
        // the frame blank until the reader happened to scroll it into view,
        // and a preview that appears empty reads as one that is broken.
        defaultMotion="none"
      />
    </div>
  );

  return (
    <figure className="zellij-section-preview" data-example={example}>
      <figcaption className="zellij-section-preview-label">
        {label ?? found.section.type}
      </figcaption>
      <TabsClient
        panels={[
          { label: 'Preview', content: preview },
          { label: 'Source', content: source },
        ]}
      />
    </figure>
  );
}
