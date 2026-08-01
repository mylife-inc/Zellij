import type { SectionProps } from '../types';
import type { FaqSection } from '../schemas';
import { InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/**
 * Accordion built on <details>/<summary>: keyboard-operable and openable
 * without JavaScript, which keeps answers available to search engines and to
 * readers before hydration (spec §11.7).
 */
export function Faq({ section, defaultMotion }: SectionProps<FaqSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="faq"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-faq"
    >
      <SectionHeader {...(section.heading ? { heading: section.heading } : {})} motion={motion}
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <div className="zellij-faq-list">
        {section.items.map((item, index) => (
          <Reveal motion={motion} index={index} key={item.question}>
            <details className="zellij-faq-item">
              <summary className="zellij-faq-question">
                <span>{item.question}</span>
                <svg
                  className="zellij-faq-chevron"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  aria-hidden="true"
                >
                  <path d="m6 9 6 6 6-6" />
                </svg>
              </summary>
              <div className="zellij-faq-answer">
                <p>
                  <InlineMarkdown text={item.answer} />
                </p>
              </div>
            </details>
          </Reveal>
        ))}
      </div>
    </SectionShell>
  );
}
