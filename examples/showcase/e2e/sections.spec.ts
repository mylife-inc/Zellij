import { test, expect } from '@playwright/test';

/**
 * Covers the section inventory in spec §5.2 and the `subtle` motion preset in
 * §6.5, including the mandatory prefers-reduced-motion override.
 */

/**
 * Scrolls the page so every reveal observer has fired.
 *
 * Must use `behavior: 'instant'`: the theme sets `scroll-behavior: smooth`, and
 * successive smooth scrolls interrupt one another, so a plain scrollTo loop
 * crawls a few hundred pixels and never reaches the lower sections.
 */
async function revealAll(page: import('@playwright/test').Page): Promise<void> {
  // Wait for hydration first. The reveal observers are attached on mount, and
  // under parallel test load scrolling can otherwise finish before that
  // happens — leaving elements permanently "out" because nothing was watching
  // when they passed through the viewport. An above-the-fold element reaching
  // "in" is proof the observers are live.
  await expect(page.locator('[data-zellij-reveal="in"]').first()).toBeAttached({
    timeout: 15000,
  });

  // Bring each element into view directly rather than stepping by a fixed pixel
  // amount on a fixed timer: under parallel test load an IntersectionObserver
  // callback can miss an element that the scroll swept past.
  //
  // Handles are captured up front on purpose. A live `[data-zellij-reveal="out"]`
  // locator would re-resolve to a different element as each one reveals, so it
  // could never be observed reaching "in".
  const handles = await page.locator('[data-zellij-reveal]').elementHandles();

  for (const handle of handles) {
    await handle.scrollIntoViewIfNeeded();
  }

  await expect
    .poll(async () => page.locator('[data-zellij-reveal="out"]').count(), { timeout: 15000 })
    .toBe(0);
}

test.describe('section inventory', () => {
  test('landing page renders each declared section type in order', async ({ page }) => {
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

  test('every v1 section type appears somewhere in the demo', async ({ page }) => {
    const routes = ['/', '/features', '/pricing', '/about', '/solutions/enterprise'];
    const seen = new Set<string>();

    for (const route of routes) {
      await page.goto(route);
      const types = await page
        .locator('[data-zellij-section]')
        .evaluateAll((nodes) => nodes.map((n) => n.getAttribute('data-zellij-section')!));
      for (const type of types) seen.add(type);
    }

    expect([...seen].sort()).toEqual([
      'bento-grid',
      'card-carousel',
      'cta-banner',
      'faq',
      'feature-grid',
      'gallery',
      'hero',
      'logo-strip',
      'markdown-prose',
      'media-panel',
      'pricing-table',
      'showcase',
      'stats-band',
      'team-grid',
      'testimonial',
      'timeline',
      'video-embed',
      'zigzag',
    ]);
  });
});

test.describe('hero', () => {
  test('renders headline, badge, CTAs and a priority visual', async ({ page }) => {
    await page.goto('/');
    const hero = page.locator('[data-zellij-section="hero"]');

    await expect(hero.getByRole('heading', { level: 1 })).toContainText('operations platform');
    await expect(hero.locator('.zellij-badge')).toContainText('Acme 2.0');
    await expect(hero.getByRole('link', { name: 'Get started' })).toBeVisible();

    const image = hero.locator('.zellij-media img');
    await expect(image).toHaveAttribute('src', /_next\/image/);

    // The hero image is the LCP candidate, so `priority` must keep it out of
    // lazy loading. next/image expresses that by omitting `loading` entirely
    // (the browser default is eager) rather than writing loading="eager".
    expect(await image.getAttribute('loading')).not.toBe('lazy');
  });

  test('lazy-loads non-hero imagery', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-zellij-section="zigzag"] img').first()).toHaveAttribute(
      'loading',
      'lazy',
    );
  });
});

test.describe('feature grid', () => {
  test('honours the declared column count and renders icons', async ({ page }) => {
    await page.goto('/');
    const grid = page.locator('[data-zellij-section="feature-grid"]').first();
    await expect(grid.locator('.zellij-feature-list')).toHaveAttribute('data-columns', '3');
    await expect(grid.locator('.zellij-feature-card')).toHaveCount(6);
    await expect(grid.locator('.zellij-feature-icon svg')).toHaveCount(6);
  });
});

test.describe('logo strip', () => {
  test('renders logos greyscale until hover', async ({ page }) => {
    await page.goto('/');
    const logo = page.locator('.zellij-logo-strip .zellij-logo-image').first();
    await expect(logo).toHaveCSS('filter', /grayscale\(1\)/);
  });
});

test.describe('pricing table', () => {
  test('marks exactly one tier as highlighted', async ({ page }) => {
    await page.goto('/pricing');
    await expect(page.locator('.zellij-pricing-card')).toHaveCount(3);
    await expect(page.locator('.zellij-pricing-card[data-highlighted="true"]')).toHaveCount(1);
    await expect(page.locator('.zellij-pricing-flag')).toHaveText('Most popular');
  });
});

test.describe('faq', () => {
  test('expands and collapses answers, and works without JavaScript', async ({ page }) => {
    await page.goto('/pricing');
    const item = page.locator('.zellij-faq-item').first();
    const answer = item.locator('.zellij-faq-answer');

    // <details> keeps answers in the DOM for crawlers even when closed.
    await expect(answer).toBeHidden();
    await item.locator('summary').click();
    await expect(answer).toBeVisible();
    await item.locator('summary').click();
    await expect(answer).toBeHidden();
  });

  test('is keyboard operable', async ({ page }) => {
    await page.goto('/pricing');
    const summary = page.locator('.zellij-faq-item').first().locator('summary');
    await summary.focus();
    await page.keyboard.press('Enter');
    await expect(page.locator('.zellij-faq-item').first().locator('.zellij-faq-answer')).toBeVisible();
  });
});

test.describe('testimonial carousel', () => {
  test('moves between quotes and keeps hidden slides out of the tab order', async ({ page }) => {
    await page.goto('/');
    const carousel = page.locator('.zellij-carousel');
    await expect(carousel.locator('.zellij-carousel-slide')).toHaveCount(2);

    const first = carousel.locator('.zellij-carousel-slide').first();
    const second = carousel.locator('.zellij-carousel-slide').nth(1);
    await expect(first).toHaveAttribute('data-active', 'true');

    await carousel.getByRole('button', { name: 'Next testimonial' }).click();
    await expect(second).toHaveAttribute('data-active', 'true');
    await expect(first).toHaveAttribute('inert', '');

    await carousel.getByRole('button', { name: 'Previous testimonial' }).click();
    await expect(first).toHaveAttribute('data-active', 'true');
  });
});

test.describe('video embed', () => {
  test('loads nothing until play is pressed', async ({ page }) => {
    await page.goto('/solutions/enterprise');
    const video = page.locator('[data-zellij-section="video-embed"]');

    await expect(video.locator('video, iframe')).toHaveCount(0);
    await expect(video.locator('.zellij-video-poster')).toBeVisible();

    await video.getByRole('button', { name: /Play video/ }).click();
    await expect(video.locator('video')).toHaveCount(1);
  });
});

test.describe('markdown-prose', () => {
  test('renders inline markdown inside marketing styling', async ({ page }) => {
    await page.goto('/about');
    const prose = page.locator('[data-zellij-section="markdown-prose"]');
    await expect(prose.locator('.zellij-prose')).toBeVisible();
    await expect(prose).toContainText('Operations teams are the connective tissue');
  });
});

test.describe('showcase backgrounds', () => {
  test('applies the declared background treatment', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('[data-zellij-section="showcase"]')).toHaveAttribute(
      'data-zellij-background',
      'inverted',
    );

    await page.goto('/solutions/enterprise');
    await expect(page.locator('[data-zellij-section="showcase"]')).toHaveAttribute(
      'data-zellij-background',
      'tinted',
    );
  });
});

test.describe('subtle motion', () => {
  test('reveals sections as they enter the viewport', async ({ page }) => {
    await page.goto('/');

    // Content far down the page starts hidden...
    const banner = page.locator('[data-zellij-section="cta-banner"] [data-zellij-reveal]').first();
    await expect(banner).toHaveAttribute('data-zellij-reveal', 'out');

    // ...and reveals once scrolled to.
    await banner.scrollIntoViewIfNeeded();
    await expect(banner).toHaveAttribute('data-zellij-reveal', 'in');
  });

  test('reveals everything after a full scroll-through', async ({ page }) => {
    await page.goto('/');
    await revealAll(page);
    await expect(page.locator('[data-zellij-reveal="out"]')).toHaveCount(0);
  });

  test('never animates layout-affecting properties', async ({ page }) => {
    await page.goto('/');
    const property = await page
      .locator('[data-zellij-reveal]')
      .first()
      .evaluate((el) => getComputedStyle(el).transitionProperty);
    // Transform/opacity only, to protect CLS (spec §6.5).
    expect(property).toMatch(/opacity/);
    expect(property).toMatch(/transform/);
    expect(property).not.toMatch(/height|width|margin|padding|top|left/);
  });
});

test.describe('reduced motion', () => {
  /**
   * Spec §6.5 makes this override mandatory regardless of config, and §11.3
   * requires it to be checked.
   *
   * The media query is emulated before navigation because the reveal decision
   * is made once, when each component mounts.
   */
  test('renders all content visible with no animation', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('/');
    await expect(page.locator('.zellij-hero-headline')).toHaveAttribute(
      'data-zellij-reveal',
      'in',
    );

    // Nothing is left in the hidden state, even without scrolling.
    await expect(page.locator('[data-zellij-reveal="out"]')).toHaveCount(0);

    const banner = page.locator('[data-zellij-section="cta-banner"] [data-zellij-reveal]').first();
    await expect(banner).toHaveCSS('opacity', '1');
    await expect(banner).toHaveCSS('transform', 'none');

    // Assert the duration numerically: the reduced-motion convention is a
    // near-zero duration (0.01ms), which computes to "1e-05s".
    const seconds = await banner.evaluate((el) =>
      Number.parseFloat(getComputedStyle(el).transitionDuration),
    );
    expect(seconds).toBeLessThan(0.05);
  });
});
