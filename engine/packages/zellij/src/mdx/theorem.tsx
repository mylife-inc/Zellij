import type { ReactNode } from 'react';

/**
 * The environments a mathematical text is built from.
 *
 * Numbering is per-kind and per-page, the convention most textbooks use: a page
 * with two theorems and a lemma reads "Theorem 1", "Lemma 1", "Theorem 2". A
 * counter is not shared across kinds, because a reader looking for Theorem 2
 * should not have to know how many definitions came before it.
 */
const KINDS = {
  theorem: 'Theorem',
  lemma: 'Lemma',
  corollary: 'Corollary',
  proposition: 'Proposition',
  definition: 'Definition',
  example: 'Example',
  remark: 'Remark',
  proof: 'Proof',
} as const;

export type TheoremKind = keyof typeof KINDS;

export interface TheoremProps {
  /** Environment name. Defaults to `theorem`. */
  kind?: TheoremKind;
  /** The theorem's own name — "Fermat's little theorem" — shown in parentheses. */
  title?: string;
  /**
   * The number to display. Numbering is authored rather than derived: MDX
   * compiles each page in isolation with no shared render pass, so a counter
   * would restart wherever a component happened to be re-rendered, and a
   * theorem whose number changes is worse than one with no number at all.
   */
  number?: string | number;
  /** Suppresses the number for a one-off environment. */
  unnumbered?: boolean;
  children?: ReactNode;
}

/**
 * A numbered environment: theorem, lemma, definition, example and friends.
 *
 * `proof` is styled differently and takes the customary end-of-proof mark
 * rather than a number.
 */
export function Theorem({
  kind = 'theorem',
  title,
  number,
  unnumbered = false,
  children,
}: TheoremProps) {
  const label = KINDS[kind] ?? KINDS.theorem;
  const isProof = kind === 'proof';
  const showNumber = !isProof && !unnumbered && number !== undefined;

  return (
    <div className="zellij-theorem" data-kind={kind}>
      <p className="zellij-theorem-head">
        <span className="zellij-theorem-label">
          {label}
          {showNumber ? ` ${number}` : ''}
        </span>
        {title ? <span className="zellij-theorem-title"> ({title})</span> : null}
      </p>

      <div className="zellij-theorem-body">{children}</div>

      {isProof ? (
        // The tombstone that closes a proof. Decorative, so it is hidden from
        // a screen reader, which would otherwise read out a stray character.
        <span className="zellij-theorem-qed" aria-hidden="true">
          ∎
        </span>
      ) : null}
    </div>
  );
}

/** `<Proof>` is `<Theorem kind="proof">`, which is how it is always written. */
export function Proof({ title, children }: { title?: string; children?: ReactNode }) {
  return (
    <Theorem kind="proof" {...(title ? { title } : {})}>
      {children}
    </Theorem>
  );
}
