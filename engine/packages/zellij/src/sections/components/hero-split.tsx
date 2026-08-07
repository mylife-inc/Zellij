import type { SectionProps } from '../types';
import type { HeroSplitSection } from '../schemas';
import {
  CtaGroup,
  GradientHeadline,
  SectionImage,
  SectionShell,
  gradientTextProps,
  hasMarkedRun,
} from '../shared';
import { Reveal } from '../../motion/reveal';
import { resolveIcon } from '../../icons';

/**
 * A terminal transcript, as text rather than a screenshot.
 *
 * A picture of a terminal is the obvious way to do this and the wrong one: it
 * cannot be selected or copied, it is unreadable to a screen reader, it ships
 * a few hundred kilobytes, and it is wrong the day the output changes. This is
 * a `<pre>`.
 *
 * Prompt and result lines are told apart by their first character, which is
 * the same convention the transcript already uses for a human reader — `$` is
 * a command someone typed, `✓` and `✗` are outcomes. Nothing else is
 * interpreted, so a line that happens to start with neither is left alone.
 */
function Terminal({ title, body }: { title: string; body: string }) {
  const lines = body.replace(/\n$/, '').split('\n');

  return (
    <div className="zellij-terminal">
      <div className="zellij-terminal-bar">
        <span className="zellij-terminal-lights" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="zellij-terminal-title">{title}</span>
      </div>

      <pre className="zellij-terminal-body">
        <code>
          {lines.map((line, index) => {
            const kind = line.startsWith('$')
              ? 'command'
              : line.startsWith('✓')
                ? 'ok'
                : line.startsWith('✗')
                  ? 'fail'
                  : undefined;

            return (
              <span key={index} className="zellij-terminal-line" {...(kind ? { 'data-kind': kind } : {})}>
                {line || ' '}
                {'\n'}
              </span>
            );
          })}
        </code>
      </pre>
    </div>
  );
}

export function HeroSplit({ section, site, defaultMotion }: SectionProps<HeroSplitSection>) {
  const motion = section.motion ?? defaultMotion;
  const { basePath } = site.config;
  const BadgeIcon = section.badgeIcon
    ? resolveIcon(section.badgeIcon, `hero-split badge (${section.id ?? 'unnamed'})`)
    : undefined;

  return (
    <SectionShell
      type="hero-split"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-hero-split"
    >
      <div className="zellij-hero-split-grid" data-media={section.media}>
        <div className="zellij-hero-split-copy">
          {section.badge ? (
            <Reveal motion={motion} as="p" className="zellij-badge">
              {BadgeIcon ? <BadgeIcon className="zellij-badge-icon" aria-hidden="true" /> : null}
              {section.badge}
            </Reveal>
          ) : null}

          {/*
            The headline takes gradient props on the element only when nothing
            inside it is marked. With a marked run the gradient belongs to that
            run's span, and painting the whole element as well would put the
            unmarked half in gradient too — which is the thing marking exists to
            avoid.
          */}
          <Reveal
            motion={motion}
            as="h1"
            index={section.badge ? 1 : 0}
            {...(hasMarkedRun(section.headline)
              ? { className: 'zellij-hero-headline' }
              : gradientTextProps(section.titleGradient, 'zellij-hero-headline'))}
          >
            <GradientHeadline text={section.headline} stops={section.titleGradient} />
          </Reveal>

          {section.subhead ? (
            <Reveal motion={motion} as="p" className="zellij-hero-subhead" index={2}>
              {section.subhead}
            </Reveal>
          ) : null}

          {section.cta.length > 0 ? (
            <Reveal motion={motion} index={3}>
              <CtaGroup cta={section.cta} />
            </Reveal>
          ) : null}

          {section.highlights.length > 0 ? (
            <Reveal motion={motion} index={4}>
              <ul className="zellij-hero-highlights">
                {section.highlights.map((item) => {
                  const Icon = item.icon
                    ? resolveIcon(item.icon, `hero-split highlight "${item.label}"`)
                    : undefined;

                  return (
                    <li key={item.label}>
                      {Icon ? <Icon className="zellij-hero-highlight-icon" aria-hidden="true" /> : null}
                      {item.label}
                    </li>
                  );
                })}
              </ul>
            </Reveal>
          ) : null}
        </div>

        <Reveal
          motion={motion}
          index={2}
          direction={section.media === 'left' ? 'right' : 'left'}
          className="zellij-hero-split-media"
        >
          {section.terminal ? (
            <Terminal title={section.terminal.title} body={section.terminal.body} />
          ) : section.visual ? (
            <SectionImage
              basePath={basePath}
              src={section.visual}
              alt=""
              ratio="4 / 3"
              // Beside the headline, this is the LCP element as often as the
              // text is.
              priority
              sizes="(max-width: 900px) 100vw, 560px"
            />
          ) : null}
        </Reveal>
      </div>
    </SectionShell>
  );
}
