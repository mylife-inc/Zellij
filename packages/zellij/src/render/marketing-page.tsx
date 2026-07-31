import type { Site } from '../content/site';
import type { MarketingPage } from '../content/types';
import { SectionRenderer } from './section-renderer';

const LAYOUT_CLASS: Record<MarketingPage['layout'], string> = {
  stack: 'zellij-layout-stack',
  'stack-narrow': 'zellij-layout-stack-narrow',
  'stack-sidebar': 'zellij-layout-stack-sidebar',
  landing: 'zellij-layout-landing',
};

export function MarketingPageView({ site, page }: { site: Site; page: MarketingPage }) {
  const motion = page.motion ?? site.config.motion;

  return (
    <main
      id="content"
      className={LAYOUT_CLASS[page.layout]}
      data-zellij-layout={page.layout}
      data-zellij-motion={motion}
    >
      {page.sections.map((section, index) => (
        <SectionRenderer
          key={`${section.type}-${index}`}
          section={section}
          site={site}
          defaultMotion={motion}
        />
      ))}
    </main>
  );
}
