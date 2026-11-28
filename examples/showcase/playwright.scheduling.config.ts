import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir: './e2e', testMatch: 'resize-scheduling.spec.ts', fullyParallel: true,
  workers: 2, retries: 0, reporter: [['line'], ['json', { outputFile: 'scheduling-results/results.json' }]],
  outputDir: 'scheduling-results/cases', timeout: 45_000,
  use: { ...devices['Desktop Safari'], baseURL: 'http://127.0.0.1:4317', trace: 'retain-on-failure' },
  webServer: { command: 'npx next start --port 4317', url: 'http://127.0.0.1:4317', timeout: 120_000 },
});
