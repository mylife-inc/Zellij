import { test, expect } from '@playwright/test';
import { openPanel } from './helpers';

/**
 * Covers the content model: the brand block, menu items resolved from
 * `menu.yaml`, and pages composed from named section files.
 */

test.describe('brand block', () => {
  test('renders the logo image and links it home', async ({ page }) => {
    await page.goto('/features');

    const logo = page.locator('.zellij-brand .zellij-logo');
    await expect(logo).toHaveAttribute('href', '/');

    const image = logo.locator('.zellij-brand-image[data-variant="light"]');
    await expect(image).toBeVisible();
    await expect(image).toHaveAttribute('alt', 'Acme');

    // The mark must actually load — a broken logo is invisible in a screenshot
    // because the alt text renders in roughly the same place.
    const loaded = await image.evaluate((el) => (el as HTMLImageElement).naturalWidth > 0);
    expect(loaded).toBe(true);
  });

  test('goes home when clicked', async ({ page }) => {
    await page.goto('/pricing');
    await page.locator('.zellij-brand .zellij-logo').click();
    await expect(page).toHaveURL(/\/$/);
  });
});

test.describe('menu', () => {
  test.skip(({ isMobile }) => Boolean(isMobile), 'the desktop bar is hidden on mobile');

  test('lists the items declared in menu.yaml, in order', async ({ page }) => {
    await page.goto('/');

    const items = page.locator('.zellij-nav-desktop .zellij-nav-item');
    await expect(items).toHaveText([
      'Features',
      'Pricing',
      'Solutions',
      'Resources',
      'Security',
    ]);
  });

  test('resolves a page reference to that page route', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Features', exact: true }).first().click();
    await expect(page).toHaveURL(/\/features$/);
  });

  test('marks an external item and opens it in a new tab', async ({ page }) => {
    await page.goto('/');
    await openPanel(page, 'Resources', '.zellij-mega-panel');

    const github = page.locator('.zellij-mega-link', { hasText: 'GitHub' });
    await expect(github).toBeVisible();
    await expect(github).toHaveAttribute('href', 'https://github.com/shebka/acme');
    await expect(github).toHaveAttribute('target', '_blank');
    await expect(github).toHaveAttribute('rel', /noopener/);
    await expect(github.locator('.zellij-external-icon')).toBeVisible();
  });

  test('leaves an internal item routed in place', async ({ page }) => {
    await page.goto('/');
    await openPanel(page, 'Resources', '.zellij-mega-panel');

    const about = page.locator('.zellij-mega-link', { hasText: 'About' });
    await expect(about).toHaveAttribute('href', '/about');
    await expect(about).not.toHaveAttribute('target', '_blank');
  });

  test('follows a url item carrying an anchor', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Security' }).first().click();
    await expect(page).toHaveURL(/\/features#security$/);
  });
});

test.describe('pages composed from section files', () => {
  test('renders every referenced section in the declared order', async ({ page }) => {
    await page.goto('/');

    const types = await page
      .locator('[data-zellij-section]')
      .evaluateAll((nodes) => nodes.map((n) => n.getAttribute('data-zellij-section')));

    expect(types).toEqual([
      'hero',
      'logo-strip',
      'feature-grid',
      'card-carousel',
      'zigzag',
      'stats-band',
      'media-panel',
      'showcase',
      'testimonial',
      'cta-banner',
    ]);
  });

  test('keeps the anchor ids the section files declare', async ({ page }) => {
    await page.goto('/features');
    for (const id of ['workflows', 'reporting', 'security', 'faq']) {
      await expect(page.locator(`#${id}`)).toHaveCount(1);
    }
  });

  test('resolves footer page references to routes', async ({ page }) => {
    await page.goto('/');
    const footer = page.getByRole('navigation', { name: 'Footer' });
    await expect(footer.getByRole('link', { name: 'Features' })).toHaveAttribute('href', '/features');
    await expect(footer.getByRole('link', { name: 'Guide', exact: true })).toHaveAttribute(
      'href',
      '/guide',
    );
  });
});
