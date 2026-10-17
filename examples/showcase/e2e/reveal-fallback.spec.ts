import { test, expect } from '@playwright/test';

// Visibility assertions alone ignore opacity: inspect the actual rendered
// reveal wrappers as well as using the native FAQ control.
test.describe('content without JavaScript', () => {
  test.use({ javaScriptEnabled: false });

  test('keeps entrance content readable and the FAQ operable', async ({ page }) => {
    await page.goto('/lumen-3-pro');
    const reveals = page.locator('[data-zellij-reveal]');
    expect(await reveals.count()).toBeGreaterThan(0);
    expect(await reveals.evaluateAll(elements => elements.every(element => {
      const style = getComputedStyle(element);
      return style.opacity === '1' && style.transform === 'none';
    }))).toBe(true);

    const item = page.locator('.zellij-faq-item').first();
    await item.locator('summary').click({ delay: 200 });
    await expect(item.locator('.zellij-faq-answer')).toBeVisible();
    await item.locator('summary').click({ delay: 200 });
    await expect(item.locator('.zellij-faq-answer')).toBeHidden();
  });
});

test('late hydration preserves a visible and already-open FAQ', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  await page.route('**/_next/**/*.js*', async route => {
    await gate;
    await route.continue();
  });
  try {
    await page.goto('/lumen-3-pro', { waitUntil: 'domcontentloaded' });
    const item = page.locator('.zellij-faq-item').first();
    const reveal = item.locator('..');
    await expect(reveal).toHaveAttribute('data-zellij-reveal', 'pending');
    await expect(reveal).toHaveCSS('opacity', '1');
    await item.locator('summary').click({ delay: 200 });
    await expect(item.locator('.zellij-faq-answer')).toBeVisible();
    release();
    await expect(reveal).toHaveAttribute('data-zellij-reveal', 'in');
    await expect(reveal).toHaveCSS('opacity', '1');
    await expect(item.locator('.zellij-faq-answer')).toBeVisible();
    await item.locator('summary').click({ delay: 200 });
    await expect(item.locator('.zellij-faq-answer')).toBeHidden();
    expect(errors).toEqual([]);
  } finally {
    release();
    await page.unrouteAll({ behavior: 'wait' });
  }
});
