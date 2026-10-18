import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { siteConfigSchema } from '../config/schema';
import { loadSite } from '../content/site';
import type { ContentSource, SectionsPage } from '../content/types';
import { SectionsPageView } from './sections-page';

it('renders an authored page theme into the server output without changing the shared site theme', async () => {
  const config = siteConfigSchema.parse({ name: 'Test', theme: 'cupertino' });
  const page: SectionsPage = { kind: 'sections', name: 'launch', route: '/launch', sourceFile: 'pages.yaml', title: 'Launch', description: '', layout: 'stack', theme: 'mono', sections: [] };
  const source: ContentSource = {
    loadSiteConfig: async () => config, loadMenu: async () => undefined,
    listPages: async () => [page], listGuides: async () => [], listExamples: async () => [],
    resolveAsset: async source => ({ source, url: source }),
  };
  const site = await loadSite(source);
  const html = renderToStaticMarkup(<SectionsPageView site={site} page={page} />);
  expect(html).toContain('data-zellij-theme="mono"');
  expect(html).toContain('--zellij-bg:');
  expect(html).toContain('prefers-color-scheme: dark');
  expect(site.config.theme).toBe('cupertino');

  const { theme: _, ...ordinary } = page;
  expect(renderToStaticMarkup(<SectionsPageView site={site} page={ordinary} />)).not.toContain('data-zellij-theme');
});
