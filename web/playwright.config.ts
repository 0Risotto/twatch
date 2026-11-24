import { defineConfig, devices } from '@playwright/test'

const PORT = 4173
const TOKEN = 'e2e-token'

export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 7'] } },
  ],
  webServer: {
    command: `rm -rf /tmp/twatch-playwright && XDG_CONFIG_HOME=/tmp/twatch-playwright/config XDG_CACHE_HOME=/tmp/twatch-playwright/cache cargo run -p twatchd -- --port ${PORT} --token ${TOKEN} --web-dir web/dist`,
    cwd: '..',
    url: `http://127.0.0.1:${PORT}/api/health`,
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
})
