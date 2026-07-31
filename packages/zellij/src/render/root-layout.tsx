import type { ReactNode } from 'react';
import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';

/**
 * The wrapper app's root layout. Emits the html/body shell, the theme + mode
 * attributes that all token CSS keys off (spec §6), and the skip link required
 * by §5.5 accessibility.
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
        <body>
          <a className="zellij-skip-link" href="#content">
            Skip to content
          </a>
          {children}
        </body>
      </html>
    );
  };
}
