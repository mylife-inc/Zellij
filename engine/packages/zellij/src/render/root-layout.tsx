import type { ReactNode } from 'react';
import { createHash } from 'node:crypto';
import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';
import { NavBar } from '../chrome/nav-bar';
import { Footer } from '../chrome/footer';
import { Analytics } from '../chrome/analytics';
import { themeInitScript } from '../chrome/theme-preferences';
import { THEME_NAMES } from '../config/schema';
import { emitThemeCss } from '../themes/emit';
import { fontVariables } from '../themes/fonts';
import { DevelopmentReload } from '../chrome/development-reload';
import { DEVELOPMENT_REVISION_FILE } from '../config/development';
import { withBasePath } from '../content/paths';

/**
 * The wrapper app's root layout. Emits the html/body shell, the theme + mode
 * attributes all token CSS keys off (spec §6), and the nav and footer that are
 * shared by every marketing page and the guide docs layout (spec §5.5).
 */
export function createZellijRootLayout(options: ZellijRuntimeOptions = {}) {
  return async function ZellijRootLayout({ children }: { children: ReactNode }) {
    const { site, developmentRevision } = await loadZellijRuntime(options);
    const { config } = site;
    const themeCss = emitThemeCss(config);
    // React's managed stylesheets are matched by resource identity, rather
    // than a cursor through head children that a host may add whitespace to.
    // Resource contents are immutable: changed CSS must receive a new href.
    const themeHref = `zellij-theme-${createHash('sha256').update(themeCss).digest('hex')}`;

    return (
      <html
        lang={config.language}
        dir={config.direction}
        className={fontVariables}
        data-zellij-theme={config.theme}
        data-zellij-mode={config.mode}
        data-zellij-motion={config.motion}
        suppressHydrationWarning
      >
        <head>
          {/* The active theme's tokens, or every theme when the picker is enabled. */}
          <style
            href={themeHref}
            precedence="zellij-theme"
          >{themeCss}</style>
        </head>
        <body>
          {config.nav.themeToggle || config.nav.themeSwitcher ? (
            // Run synchronously before visible body content, without relying
            // on ordinary head-child matching during hydration.
            <script
              data-mode-toggle={config.nav.themeToggle ? 'true' : undefined}
              data-themes={config.nav.themeSwitcher ? THEME_NAMES.join(' ') : undefined}
              dangerouslySetInnerHTML={{ __html: themeInitScript }}
            />
          ) : null}
          <a className="zellij-skip-link" href="#content">
            Skip to content
          </a>
          <NavBar site={site} />
          {children}
          <Footer site={site} />
          {process.env['NODE_ENV'] === 'development' && developmentRevision
            ? <DevelopmentReload
                url={withBasePath(`/_zellij/${DEVELOPMENT_REVISION_FILE}`, config.basePath)}
                revision={developmentRevision}
              />
            : null}
          {process.env['NODE_ENV'] === 'production' && config.analytics.provider === 'plausible'
            ? <Analytics domain={config.analytics.domain} />
            : null}
        </body>
      </html>
    );
  };
}
