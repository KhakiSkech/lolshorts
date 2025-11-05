import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Testing Configuration for LoLShorts
 *
 * Tests the full application stack:
 * - Tauri desktop application
 * - React frontend
 * - Rust backend commands
 * - Authentication flows
 * - Recording functionality
 * - Video processing operations
 */

export default defineConfig({
  testDir: './tests/e2e',

  // Maximum time one test can run
  timeout: 60 * 1000,

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry on CI only
  retries: process.env.CI ? 2 : 0,

  // Reporter to use
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
  ],

  // Shared settings for all tests
  use: {
    // Base URL for the application
    baseURL: 'http://localhost:1420',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',

    // Maximum time each action can take
    actionTimeout: 10 * 1000,
  },

  // Configure projects for different browsers/scenarios
  projects: [
    {
      name: 'Desktop Chrome',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1280, height: 720 },
      },
    },

    {
      name: 'Desktop Firefox',
      use: {
        ...devices['Desktop Firefox'],
        viewport: { width: 1280, height: 720 },
      },
    },

    {
      name: 'Desktop Edge',
      use: {
        ...devices['Desktop Edge'],
        viewport: { width: 1280, height: 720 },
      },
    },
  ],

  // Run local dev server before starting tests
  webServer: {
    command: 'npm run tauri dev',
    url: 'http://localhost:1420',
    timeout: 120 * 1000,
    reuseExistingServer: !process.env.CI,
  },
});
