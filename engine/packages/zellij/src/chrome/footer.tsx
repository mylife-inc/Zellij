import Link from 'next/link';
import type { Site } from '../content/site';

/**
 * Declarative footer sharing the nav's link-validation guarantees (spec §8).
 * A server component — nothing here is interactive.
 */
export function Footer({ site }: { site: Site }) {
  const { footer, name, tagline } = site.config;
  const hasColumns = site.footer.length > 0;

  return (
    <footer className="zellij-footer">
      <div className="zellij-container zellij-footer-inner">
        <div className="zellij-footer-brand">
          <span className="zellij-logo-text">{name}</span>
          {tagline ? <p className="zellij-footer-tagline">{tagline}</p> : null}
        </div>

        {hasColumns ? (
          <nav className="zellij-footer-columns" aria-label="Footer">
            {site.footer.map((column) => (
              <div className="zellij-footer-column" key={column.heading}>
                <h2 className="zellij-footer-heading">{column.heading}</h2>
                <ul>
                  {column.links.map((link) => (
                    <li key={`${column.heading}-${link.href}`}>
                      {link.external ? (
                        <a
                          className="zellij-footer-link"
                          href={link.href}
                          target="_blank"
                          rel="noreferrer noopener"
                        >
                          {link.label}
                        </a>
                      ) : (
                        <Link className="zellij-footer-link" href={link.href}>
                          {link.label}
                        </Link>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        ) : null}
      </div>

      <div className="zellij-container zellij-footer-bottom">
        {footer.legal ? <p className="zellij-footer-legal">{footer.legal}</p> : null}
        <FooterSocial social={footer.social ?? site.config.nav.social} />
      </div>
    </footer>
  );
}

function FooterSocial({ social }: { social: Site['config']['footer']['social'] }) {
  if (!social) return null;

  const entries = Object.entries(social).filter(([, url]) => Boolean(url)) as Array<
    [string, string]
  >;
  if (entries.length === 0) return null;

  return (
    <ul className="zellij-footer-social">
      {entries.map(([network, url]) => (
        <li key={network}>
          <a
            href={url}
            target="_blank"
            rel="noreferrer noopener"
            className="zellij-footer-social-link"
          >
            {label(network)}
          </a>
        </li>
      ))}
    </ul>
  );
}

function label(network: string): string {
  if (network === 'x') return 'X';
  return network.charAt(0).toUpperCase() + network.slice(1);
}
