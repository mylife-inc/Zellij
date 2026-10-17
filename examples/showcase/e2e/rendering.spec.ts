import { expect, test } from '@playwright/test';

test('renders formulas and Mermaid math labels with patched dependencies', async ({ page }) => {
  await page.goto('/guide/config/database');
  await expect(page.locator('mjx-container svg')).toHaveCount(2);
  const diagram = page.locator('.zellij-mermaid');
  await diagram.scrollIntoViewIfNeeded();
  await expect(diagram).toHaveAttribute('data-status', 'drawn');
  await expect(diagram.locator('svg')).toBeVisible();
  await expect(diagram.locator('math')).toBeAttached();
  await expect(page.locator('.zellij-mermaid-error')).toHaveCount(0);
});
