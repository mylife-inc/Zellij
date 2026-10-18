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
  reporter: process.env['CI'] ? 'line' : [['list']],
  /*
   * The hover-intent and scroll-snap tests drive real pointer input against
   * real timers — a 260ms menu dwell, a 30-step mouse traverse. Those are
   * sensitive to how much else the machine is doing, and with 240 tests in
   * parallel that shows. Retries keep that noise visible without discarding the
   * first failure; retained traces must still be investigated when a retry passes.
   */
  retries: process.env['CI'] ? 2 : 1,

  use: {
    baseURL,
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'desktop', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 7'] } },
    /*
     * Both projects above are Chromium, so until this one existed the engine
     * had never been tested in the browser most of its mobile visitors use.
     * Three layout bugs reached a phone through that gap — a drawer trapped in
     * the header's containing block, a search overlay trapped the same way, and
     * a callout a table could push off the screen. All three were found by hand
     * on a real iPhone.
     *
     * WebKit is not a perfect stand-in for Safari, and it does not need to be:
     * none of those were WebKit quirks. What was missing was a mobile viewport
     * that anything ran against beyond the top of a page, and a second engine
     * to disagree when Chromium is forgiving.
     */
    { name: 'mobile-safari', use: { ...devices['iPhone 14'] } },
  ],
  webServer: {
    command: `npx next start --port ${PORT}`,
    url: baseURL,
    reuseExistingServer: !process.env['CI'],
    timeout: 120_000,
  },
});
