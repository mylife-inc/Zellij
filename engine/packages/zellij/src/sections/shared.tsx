import Image from 'next/image';
import Link from 'next/link';
import type { CSSProperties, ReactNode } from 'react';

import type { Cta, MotionPreset } from '../config/schema';
import type { SectionBackground } from './registry';
import { isExternal } from '../content/paths';
import { assetHref } from '../content/asset-href';
import { Reveal } from '../motion/reveal';

/** Palette tokens a gradient stop may name, so it follows the active theme. */
const PALETTE_TOKENS = new Set([
  'bg',
  'fg',
  'muted',
  'surface',
  'border',
  'accent',
  'accent-fg',
  'inverted-bg',
  'inverted-fg',
]);

/**
 * A gradient stop: a palette token name, or a literal CSS colour. Literals are
 * validated rather than trusted — they come from a content file but end up
 * interpolated into a style attribute.
 */
function gradientStop(value: string): string | undefined {
  const token = value.trim();
  if (PALETTE_TOKENS.has(token)) return `var(--zellij-${token})`;
  return /^[#a-zA-Z0-9(),.%\s/-]+$/.test(token) && !token.includes('</') ? token : undefined;
}

/** `linear-gradient(...)` from authored stops, or nothing if too few resolve. */
function gradientImage(stops: readonly string[], angle: string): string | undefined {
  const resolved = stops.map(gradientStop).filter(Boolean) as string[];
  if (resolved.length < 2) return undefined;
  return `linear-gradient(${angle}, ${resolved.join(', ')})`;
}

function backgroundStyle(background: SectionBackground | undefined): CSSProperties | undefined {
  if (!background || typeof background === 'string') return undefined;

  const image = gradientImage(background.gradient, '160deg');
  return image ? { backgroundImage: image } : undefined;
}

/**
 * Props that paint text with a gradient — spread onto the heading element.
 *
 * The gradient is clipped to the glyphs, which means the text has no colour of
 * its own; `.zellij-gradient-text` restores one under forced-colors, where
 * clipped backgrounds are dropped and the heading would otherwise vanish.
 */
export function gradientTextProps(
  stops: readonly string[] | undefined,
  className?: string,
): { className?: string; style?: CSSProperties } {
  const image = stops ? gradientImage(stops, '100deg') : undefined;
  if (!image) return className ? { className } : {};

  return {
    className: [className, 'zellij-gradient-text'].filter(Boolean).join(' '),
    style: { backgroundImage: image },
  };
}

/**
 * A headline where only part of it takes the gradient.
 *
 * `*marked*` runs get the treatment and the rest stays the body colour. A
 * headline with no markers behaves as it always did — the whole line is
 * painted — so this changes nothing for existing content.
 *
 * The reason to want it: a two-sentence headline where both sentences are
 * gradient has no emphasis in it, only decoration. Colouring the second and
 * leaving the first in the text colour is what makes the second one land, and
 * it is what most designs of this shape actually do.
 *
 * Marking without a `titleGradient` renders the marked run in the accent
 * colour, which is the sane reading of "emphasise this" when no stops were
 * given.
 */
export function GradientHeadline({
  text,
  stops,
}: {
  text: string;
  stops?: readonly string[] | undefined;
}) {
  const parts = text.split(/\*([^*]+)\*/g);

  // Odd indices are the marked runs — `split` with one capture group
  // alternates between the text around the match and the match itself.
  if (parts.length === 1) return <>{text}</>;

  const image = stops ? gradientImage(stops, '100deg') : undefined;

  return (
    <>
      {parts.map((part, index) =>
        index % 2 === 1 ? (
          <span
            key={index}
            className={image ? 'zellij-gradient-text' : 'zellij-headline-accent'}
            {...(image ? { style: { backgroundImage: image } } : {})}
          >
            {part}
          </span>
        ) : (
          part
        ),
      )}
    </>
  );
}

/** True when a headline marks part of itself for emphasis. */
export function hasMarkedRun(text: string): boolean {
  return /\*[^*]+\*/.test(text);
}

/** Shared frame for every section: anchor target, motion scope, container. */
export function SectionShell({
  type,
  id,
  motion,
  background = 'default',
  padding,
  className,
  children,
  full = false,
}: {
  type: string;
  id?: string;
  motion: MotionPreset;
  background?: SectionBackground;
  padding?: 'none' | 'tight' | 'normal' | 'loose';
  className?: string;
  children: ReactNode;
  /** Skip the inner container for full-bleed sections. */
  full?: boolean;
}) {
  const style = backgroundStyle(background);

  return (
    <section
      {...(id ? { id } : {})}
      className={['zellij-section', className].filter(Boolean).join(' ')}
      data-zellij-section={type}
      data-zellij-motion={motion}
      data-zellij-background={typeof background === 'string' ? background : 'gradient'}
      {...(padding ? { 'data-zellij-padding': padding } : {})}
      {...(style ? { style } : {})}
    >
      {full ? children : <div className="zellij-container">{children}</div>}
    </section>
  );
}

/** Eyebrow + heading + subhead, the standard section header. */
export function SectionHeader({
  eyebrow,
  heading,
  subhead,
  motion,
  align = 'center',
  titleGradient,
}: {
  eyebrow?: string;
  heading?: string;
  subhead?: string;
  motion: MotionPreset;
  align?: 'center' | 'start';
  titleGradient?: readonly string[];
}) {
  if (!eyebrow && !heading && !subhead) return null;

  return (
    <Reveal motion={motion} className="zellij-section-header" data-align={align}>
      {eyebrow ? <p className="zellij-eyebrow">{eyebrow}</p> : null}
      {heading ? (
        <h2 {...gradientTextProps(titleGradient, 'zellij-section-heading')}>{heading}</h2>
      ) : null}
      {/* Inline markdown, like every other body field. A subhead is prose and
          authors write prose with emphasis in it; without this the asterisks
          reach the page as asterisks. */}
      {subhead ? (
        <p className="zellij-section-subhead">
          <InlineMarkdown text={subhead} />
        </p>
      ) : null}
    </Reveal>
  );
}

export function CtaLink({ cta, className }: { cta: Cta; className?: string }) {
  const classes = ['zellij-button', className].filter(Boolean).join(' ');

  if (isExternal(cta.href)) {
    return (
      <a className={classes} data-style={cta.style} href={cta.href} target="_blank" rel="noreferrer noopener">
        {cta.label}
      </a>
    );
  }

  return (
    <Link className={classes} data-style={cta.style} href={cta.href}>
      {cta.label}
    </Link>
  );
}

export function CtaGroup({ cta, className }: { cta: readonly Cta[]; className?: string }) {
  if (cta.length === 0) return null;
  return (
    <div className={['zellij-cta-group', className].filter(Boolean).join(' ')}>
      {cta.map((action) => (
        <CtaLink key={`${action.href}-${action.label}`} cta={action} />
      ))}
    </div>
  );
}

/**
 * All section imagery goes through `next/image` for the Lighthouse budget
 * (spec §11.6). `fill` avoids needing intrinsic dimensions in content.
 *
 * `basePath` has to be applied to `src` here, which is easy to get wrong:
 * `next/image` prefixes the *optimizer endpoint* but leaves its `url`
 * parameter alone, and the optimizer resolves that against the origin root. A
 * content asset lives under the prefix, so an unprefixed `url` fetches the
 * 404 page and the optimizer answers "the requested resource isn't a valid
 * image" — every image on the site, silently, on any prefixed deployment.
 */
export function SectionImage({
  src,
  alt,
  basePath = '',
  ratio = '16 / 10',
  priority = false,
  sizes = '(max-width: 900px) 100vw, 60vw',
  className,
}: {
  src: string;
  alt: string;
  basePath?: string;
  ratio?: string;
  priority?: boolean;
  sizes?: string;
  className?: string;
}) {
  return (
    <div
      className={['zellij-media', className].filter(Boolean).join(' ')}
      style={{ aspectRatio: ratio }}
    >
      <Image
        src={assetHref(src, basePath)}
        alt={alt}
        fill
        sizes={sizes}
        priority={priority}
      />
    </div>
  );
}

/**
 * Minimal inline markdown for content fields the spec allows it in — zigzag
 * bodies and FAQ answers (spec §5.2). Deliberately not a markdown parser: block
 * content belongs in `markdown-prose` or a guide page.
 */
export function InlineMarkdown({ text }: { text: string }): ReactNode {
  /*
   * Bold's contents may contain single asterisks, so that emphasis can nest
   * inside it: `(?:[^*]|\*(?!\*))+` is "anything but an asterisk, or an
   * asterisk that does not begin a `**`". With a plain `[^*]+` the bold
   * alternative simply failed to match `**a *b* c**`, the single-asterisk
   * alternative matched the wrong pairs, and the sentence came apart.
   *
   * Bold is listed first for the same reason: `**` has to be tried before `*`,
   * or every bold marker is read as two empty emphases.
   */
  const pattern = /((?:\*\*(?:[^*]|\*(?!\*))+\*\*)|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  const parts = text.split(pattern).filter((part) => part !== '');

  return (
    <>
      {parts.map((part, index) => {
        /*
         * Emphasis recurses; code does not.
         *
         * One flat pass reads `**a `b` c**` as bold containing three literal
         * backticks, because the bold alternative matched first and nothing
         * looked inside it again. Naming a command inside a bold clause is
         * ordinary writing, so the inside has to be parsed too.
         *
         * The recursion terminates because the inner text is always strictly
         * shorter than the part it came from.
         *
         * Code is the exception on purpose: backticks mean "this is literal",
         * and an asterisk in a shell glob is an asterisk.
         */
        if (part.startsWith('**') && part.endsWith('**')) {
          return (
            <strong key={index}>
              <InlineMarkdown text={part.slice(2, -2)} />
            </strong>
          );
        }
        if (part.startsWith('*') && part.endsWith('*')) {
          return (
            <em key={index}>
              <InlineMarkdown text={part.slice(1, -1)} />
            </em>
          );
        }
        if (part.startsWith('`') && part.endsWith('`')) {
          return <code key={index}>{part.slice(1, -1)}</code>;
        }

        const link = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(part);
        if (link) {
          const [, label, href] = link;
          return isExternal(href!) ? (
            <a key={index} href={href!} target="_blank" rel="noreferrer noopener">
              {label}
            </a>
          ) : (
            <Link key={index} href={href!}>
              {label}
            </Link>
          );
        }

        return part;
      })}
    </>
  );
}
