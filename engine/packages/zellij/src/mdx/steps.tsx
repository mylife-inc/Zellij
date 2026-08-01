import { Children, isValidElement, type ReactNode } from 'react';

/**
 * Numbered procedure. Numbering is derived from child order rather than
 * authored, so inserting a step never renumbers the rest by hand (spec §7.2).
 */
export function Steps({ children }: { children?: ReactNode }) {
  const steps = Children.toArray(children).filter(isValidElement);

  return (
    <ol className="zellij-steps">
      {steps.map((step, index) => (
        <li className="zellij-step" key={index}>
          <span className="zellij-step-marker" aria-hidden="true">
            {index + 1}
          </span>
          <div className="zellij-step-body">{step}</div>
        </li>
      ))}
    </ol>
  );
}

export function Step({ title, children }: { title?: string; children?: ReactNode }) {
  return (
    <div className="zellij-step-content">
      {title ? <p className="zellij-step-title">{title}</p> : null}
      {children}
    </div>
  );
}
