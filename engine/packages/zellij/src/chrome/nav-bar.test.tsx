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

it.each([false, true])('keeps company and product destinations independent (new tab %s)', newTab => {
  const site = {
    config: siteConfigSchema.parse({
      name: 'Product docs', basePath: '/docs',
      brand: {
        company: { src: 'assets/company.svg', dark: 'assets/company-dark.svg', alt: 'Company', href: 'https://company.example', newTab },
        logo: { src: 'assets/product.svg', alt: 'Product home' },
      },
    }), menu: [], pages: [],
  } as unknown as Site;
  const html = renderToStaticMarkup(<NavBar site={site} />);
  expect(html).toContain('src="/docs/_zellij/assets/company.svg"');
  expect(html).toContain('src="/docs/_zellij/assets/company-dark.svg"');
  expect(html).toContain('src="/docs/_zellij/assets/product.svg"');
  expect(html.indexOf('class="zellij-company"')).toBeLessThan(html.indexOf('class="zellij-logo"'));
  const productLink = html.match(/<a class="zellij-logo"[^>]+>/)?.[0];
  expect(productLink).toContain('href="/"');
  expect(productLink).toContain('aria-label="Product home"');
  expect(html.includes('target="_blank" rel="noopener noreferrer"')).toBe(newTab);
});

it('permits a company mark with the product name as home link, and rejects unsafe company destinations', () => {
  const company = { src: 'assets/company.svg', alt: 'Company', href: 'https://company.example' };
  const site = { config: siteConfigSchema.parse({ name: 'Product', brand: { company } }), menu: [], pages: [] } as unknown as Site;
  const html = renderToStaticMarkup(<NavBar site={site} />);
  expect(html).toContain('aria-label="Product home"');
  expect(html).not.toContain('target="_blank"');
  for (const href of ['javascript:alert(1)', 'file:///tmp/company', '/company', 'http:company.example', 'https://company.example/white space']) {
    expect(siteConfigSchema.safeParse({ brand: { company: { ...company, href } } }).success).toBe(false);
  }
});
