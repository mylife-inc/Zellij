import { MDXRemote } from 'next-mdx-remote/rsc';

import type { SectionProps } from '../types';
import type { MarkdownProseSection } from '../schemas';
import { SectionHeader, SectionShell } from '../shared';
import { mdxComponents } from '../../mdx/components';
import { mdxOptions } from '../../mdx/pipeline';

/**
 * Free-form prose inside marketing styling (spec §5.2 #11). Reuses the guide's
 * MDX pipeline and component set, so there is one MDX path in the engine and
 * `<Callout>` and friends work here too.
 */
export function MarkdownProse({ section, site, defaultMotion }: SectionProps<MarkdownProseSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="markdown-prose"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-markdown-prose"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />
      <div className="zellij-prose zellij-prose-marketing">
        <MDXRemote source={section.body} components={mdxComponents(site)} options={mdxOptions()} />
      </div>
    </SectionShell>
  );
}
