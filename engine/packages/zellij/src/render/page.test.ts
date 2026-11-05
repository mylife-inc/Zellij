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
    routes: new Set(['/']),
  } as unknown as Site;
  vi.mocked(loadZellijRuntime).mockResolvedValue({ site, contentDir: '/fixture', layout: 'folder' });
  const metadata = await createZellijPages().generateMetadata({ params: Promise.resolve({ slug: [] }) });
  const expected = ogImage.startsWith('https:') ? ogImage : '/products/docs/_zellij/assets/og.png';
  expect(metadata.openGraph).toMatchObject({ url: '/products/docs/', images: [{ url: expected }] });
  expect(metadata.twitter).toMatchObject({ images: [expected] });
  expect(metadata.alternates).toMatchObject({ canonical: '/products/docs/' });
});


it.each([
  ['guide', 'Café', 'Release Notes'],
  ['guide', 'Caf%C3%A9', 'Release%20Notes'],
])('renders the registered guide for Next page params %j', async (...slug) => {
  const page = { route: '/guide/Café/Release Notes', title: 'Release notes', body: 'Details', guide: '/guide' };
  const site = {
    config: { name: 'Test', basePath: '/products/docs' },
    pages: [], guidePages: [page], guides: [], routes: new Set([page.route]),
  } as unknown as Site;
  vi.mocked(loadZellijRuntime).mockResolvedValue({ site, contentDir: '/fixture', layout: 'folder' });
  const zellij = createZellijPages();
  const params = Promise.resolve({ slug });
  expect((await zellij.Page({ params })).props.page).toBe(page);
  expect(await zellij.generateMetadata({ params })).toMatchObject({ title: 'Release notes — Test' });
  expect(await zellij.generateStaticParams()).toEqual([{ slug: ['guide', 'Café', 'Release Notes'] }]);
});

it('keeps unknown encoded pages as 404s', async () => {
  const site = {
    config: { name: 'Test', basePath: '' }, pages: [], guidePages: [], guides: [], routes: new Set(['/guide/a/b']),
  } as unknown as Site;
  vi.mocked(loadZellijRuntime).mockResolvedValue({ site, contentDir: '/fixture', layout: 'folder' });
  await expect(createZellijPages().Page({ params: Promise.resolve({ slug: ['guide', 'a%2Fb'] }) })).rejects.toThrow('NEXT_HTTP_ERROR_FALLBACK;404');
});
