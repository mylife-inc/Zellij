import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it, vi } from 'vitest';
import { siteConfigSchema } from '../config/schema';
import type { Site } from '../content/site';
import { NavBar } from './nav-bar';

vi.mock('next/navigation', () => ({ usePathname: () => '/' }));

it('renders company branding as an external link and retains a separate site-home link', () => {
  const site = {
    config: siteConfigSchema.parse({
      name: 'Product docs', basePath: '/docs',
      brand: {
        logo: { src: 'assets/company.svg', dark: 'assets/company-dark.svg', alt: 'Company', href: 'https://company.example', height: '1.75rem' },
        title: 'Product docs',
      },
    }),
    menu: [], pages: [],
  } as unknown as Site;
  const html = renderToStaticMarkup(<NavBar site={site} />);
  expect(html).toContain('class="zellij-logo" href="https://company.example" aria-label="Company"');
  expect(html).toContain('src="/docs/_zellij/assets/company.svg"');
  expect(html).toContain('src="/docs/_zellij/assets/company-dark.svg"');
  expect(html).toContain('class="zellij-brand-title" href="/">Product docs</a>');
});
