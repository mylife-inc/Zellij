import type { ReactNode } from 'react';
import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';
import { NavBar } from '../chrome/nav-bar';
import { Footer } from '../chrome/footer';
import { themeInitScript } from '../chrome/theme-toggle';

/**
 * The wrapper app's root layout. Emits the html/body shell, the theme + mode
 * attributes all token CSS keys off (spec §6), and the nav and footer that are
 * shared by every marketing page and the guide docs layout (spec §5.5).
 */
export function createZellijRootLayout(options: ZellijRuntimeOptions = {}) {
  return async function ZellijRootLayout({ children }: { children: ReactNode }) {
    const { site } = await loadZellijRuntime(options);
    const { config } = site;

    return (
      <html
        lang="en"
        data-zellij-theme={config.theme}
        data-zellij-mode={config.mode}
        data-zellij-motion={config.motion}
        suppressHydrationWarning
      >
        <head>
          {config.nav.themeToggle ? (
            // Applies a stored mode before first paint to avoid a theme flash.
            <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
          ) : null}
        </head>
        <body>
          <a className="zellij-skip-link" href="#content">
            Skip to content
          </a>
          <NavBar site={site} />
          {children}
          <Footer site={site} />
        </body>
      </html>
    );
  };
}
