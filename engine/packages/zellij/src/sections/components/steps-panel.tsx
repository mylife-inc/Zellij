import type { SectionProps } from '../types';
import type { StepsPanelSection } from '../schemas';
import { InlineMarkdown, SectionHeader, SectionImage, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';
import { Terminal } from './terminal';
import { Mermaid } from '../../mdx/mermaid';

/**
 * Numbered steps on one side, the result on the other.
 *
 * The list is an `<ol>` and the numbers are drawn by CSS from its own counter,
 * so the markup carries the sequence and the styling only shows it. Nothing
 * here writes "01" — an author who numbers by hand has to renumber by hand the
 * first time a step is inserted in the middle, and a screen reader would then
 * hear every number twice.
 */
export function StepsPanel({ section, site, defaultMotion }: SectionProps<StepsPanelSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;

  return (
    <SectionShell
      type="steps-panel"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-steps-panel"
    >
      <SectionHeader
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <div className="zellij-steps-panel-grid" data-media={section.media}>
        <ol className="zellij-steps-panel-list">
          {section.steps.map((step, index) => (
            <Reveal
              as="li"
              key={step.title}
              motion={motion}
              index={index}
              className="zellij-steps-panel-item"
            >
              <div className="zellij-steps-panel-copy">
                <h3 className="zellij-steps-panel-title">{step.title}</h3>
                {step.body ? (
                  <p className="zellij-steps-panel-body">
                    <InlineMarkdown text={step.body} />
                  </p>
                ) : null}
              </div>
            </Reveal>
          ))}
        </ol>

        {/*
          One Reveal around panel and caption together. Revealing them
          separately staggers a caption in after the thing it is describing has
          already arrived, which reads as the page still loading.
        */}
        <Reveal
          motion={motion}
          index={1}
          direction={section.media === 'left' ? 'right' : 'left'}
          className="zellij-steps-panel-media"
        >
          <div className="zellij-steps-panel-sticky">
            {section.diagram ? (
              <div className="zellij-steps-panel-diagram">
                <Mermaid code={section.diagram} />
              </div>
            ) : section.terminal ? (
              <Terminal title={section.terminal.title} body={section.terminal.body} />
            ) : section.visual ? (
              <SectionImage
                basePath={basePath}
                src={section.visual}
                alt=""
                ratio="4 / 3"
                sizes="(max-width: 900px) 100vw, 560px"
              />
            ) : null}

            {section.caption ? (
              <p className="zellij-steps-panel-caption">
                <InlineMarkdown text={section.caption} />
              </p>
            ) : null}
          </div>
        </Reveal>
      </div>
    </SectionShell>
  );
}
