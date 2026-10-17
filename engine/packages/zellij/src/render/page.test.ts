import { expect, it, vi } from 'vitest';
import type { Site } from '../content/site';
import { loadZellijRuntime } from '../config/runtime';
import { createZellijPages } from './page';

vi.mock('../config/runtime', () => ({ loadZellijRuntime: vi.fn() }));
vi.mock('./guide-page', () => ({ GuidePageView: () => null }));
vi.mock('./sections-page', () => ({ SectionsPageView: () => null }));

it.each(['assets/og.png', 'https://cdn.example.com/og.png'])('keeps social images and canonical metadata prefix-correct: %s', async (ogImage) => {
  const site = {
    config: { name: 'Test', basePath: '/products/docs', siteUrl: 'https://example.com', ogImage },
    pages: [{ kind: 'sections', route: '/', title: 'Home', description: 'Description' }],
    guidePages: [],
  } as unknown as Site;
  vi.mocked(loadZellijRuntime).mockResolvedValue({ site, contentDir: '/fixture', layout: 'folder' });
  const metadata = await createZellijPages().generateMetadata({ params: Promise.resolve({ slug: [] }) });
  const expected = ogImage.startsWith('https:') ? ogImage : '/products/docs/_zellij/assets/og.png';
  expect(metadata.openGraph).toMatchObject({ url: '/products/docs/', images: [{ url: expected }] });
  expect(metadata.twitter).toMatchObject({ images: [expected] });
  expect(metadata.alternates).toMatchObject({ canonical: '/products/docs/' });
});
