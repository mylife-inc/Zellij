import { defineConfig, devices } from '@playwright/test';

const PORT = Number(process.env['ZELLIJ_E2E_PORT'] ?? 4317);
const baseURL = `http://127.0.0.1:${PORT}`;

/**
 * E2E runs against the built demo, so it exercises the same static output the
 * acceptance criteria are written against (spec §13.7).
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: Boolean(process.env['CI']),
  retries: process.env['CI'] ? 1 : 0,
  reporter: process.env['CI'] ? 'line' : [['list']],
  use: {
    baseURL,
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 7'] } },
  ],
  webServer: {
    command: `npx next start --port ${PORT}`,
    url: baseURL,
    reuseExistingServer: !process.env['CI'],
    timeout: 120_000,
  },
});
