import Image from 'next/image';
import Link from 'next/link';
import type { ReactNode } from 'react';

import type { Cta, MotionPreset } from '../config/schema';
import { assetUrl, isExternal } from '../content/paths';
import { Reveal } from '../motion/reveal';

/** Shared frame for every section: anchor target, motion scope, container. */
export function SectionShell({
  type,
  id,
  motion,
  background = 'default',
  className,
  children,
  full = false,
}: {
  type: string;
  id?: string;
  motion: MotionPreset;
  background?: 'default' | 'tinted' | 'inverted';
  className?: string;
  children: ReactNode;
  /** Skip the inner container for full-bleed sections. */
  full?: boolean;
}) {
  return (
    <section
      {...(id ? { id } : {})}
      className={['zellij-section', className].filter(Boolean).join(' ')}
      data-zellij-section={type}
      data-zellij-motion={motion}
      data-zellij-background={background}
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
}: {
  eyebrow?: string;
  heading?: string;
  subhead?: string;
  motion: MotionPreset;
  align?: 'center' | 'start';
}) {
  if (!eyebrow && !heading && !subhead) return null;

  return (
    <Reveal motion={motion} className="zellij-section-header" data-align={align}>
      {eyebrow ? <p className="zellij-eyebrow">{eyebrow}</p> : null}
      {heading ? <h2 className="zellij-section-heading">{heading}</h2> : null}
      {subhead ? <p className="zellij-section-subhead">{subhead}</p> : null}
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
 * (spec §11.6). `fill` avoids needing intrinsic dimensions in content, and
 * `next/image` applies basePath itself.
 */
export function SectionImage({
  src,
  alt,
  ratio = '16 / 10',
  priority = false,
  sizes = '(max-width: 900px) 100vw, 60vw',
  className,
}: {
  src: string;
  alt: string;
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
      <Image src={assetUrl(src)} alt={alt} fill sizes={sizes} priority={priority} />
    </div>
  );
}

/**
 * Minimal inline markdown for content fields the spec allows it in — zigzag
 * bodies and FAQ answers (spec §5.2). Deliberately not a markdown parser: block
 * content belongs in `markdown-prose` or a guide page.
 */
export function InlineMarkdown({ text }: { text: string }): ReactNode {
  const pattern = /(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  const parts = text.split(pattern).filter((part) => part !== '');

  return (
    <>
      {parts.map((part, index) => {
        if (part.startsWith('**') && part.endsWith('**')) {
          return <strong key={index}>{part.slice(2, -2)}</strong>;
        }
        if (part.startsWith('*') && part.endsWith('*')) {
          return <em key={index}>{part.slice(1, -1)}</em>;
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
