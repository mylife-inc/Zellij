import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
 testDir: '.', testMatch: 'focus.spec.ts', workers: 1, retries: 0, repeatEach: 2,
 timeout: 20000, outputDir: process.env.ZEL_FOCUS_RESULTS + '/cases',
 reporter: [['line'], ['json', { outputFile: process.env.ZEL_FOCUS_RESULTS + '/results.json' }]],
 use: { trace: 'retain-on-failure' },
 projects: [
  { name: 'chromium', use: { ...devices['Desktop Chrome'], channel: 'chrome' } },
  { name: 'webkit', use: { ...devices['Desktop Safari'] } },
 ],
});
