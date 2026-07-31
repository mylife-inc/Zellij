import Link from 'next/link';
import type { ReactNode } from 'react';
import { Icon } from '../icons';
import { isExternal } from '../content/paths';

export function CardGrid({ columns = 2, children }: { columns?: number; children?: ReactNode }) {
  return (
    <div className="zellij-card-grid" data-columns={columns}>
      {children}
    </div>
  );
}

export function Card({
  title,
  href,
  icon,
  children,
}: {
  title: string;
  href?: string;
  icon?: string;
  children?: ReactNode;
}) {
  const body = (
    <>
      {icon ? (
        <span className="zellij-card-icon">
          <Icon name={icon} context={`<Card title="${title}">`} />
        </span>
      ) : null}
      <p className="zellij-card-title">{title}</p>
      {children ? <div className="zellij-card-body">{children}</div> : null}
    </>
  );

  if (!href) return <div className="zellij-card">{body}</div>;

  if (isExternal(href)) {
    return (
      <a className="zellij-card" href={href} target="_blank" rel="noreferrer noopener">
        {body}
      </a>
    );
  }

  return (
    <Link className="zellij-card" href={href}>
      {body}
    </Link>
  );
}
