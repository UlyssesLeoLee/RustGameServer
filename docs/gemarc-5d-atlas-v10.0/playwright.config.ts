import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright 配置 — Universal GemArc 5D Design Atlas v10.0
 *
 * spec `quality_assurance.environments`:
 *   - "Chromium/Edge file://"        ← 主环境, 用系统 Chrome (channel: 'chrome')
 *   - "Chromium/Edge static-http"
 *   - "390/768/1024/1440/1920 视口"
 *   - "真实 GPU/真实鼠标若可用"
 *
 * 重要 (本机实测): npm 包的 playwright 版本与本机 ms-playwright 缓存里的
 * chromium 构建号对不上 (需 1248, 缓存只有 1223/1234/1243), 用默认 chromium
 * 会直接报 "Executable doesn't exist"。因此统一用 **系统 Chrome**:
 *   channel: 'chrome' → 走 C:\Program Files\Google\Chrome\Application\chrome.exe
 */
export default defineConfig({
  testDir: './tests/e2e',
  timeout: 60_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [['list']],
  use: {
    channel: 'chrome',
    headless: true,
    trace: 'off',
    screenshot: 'only-on-failure',
    launchOptions: {
      args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
    },
  },
  projects: [
    {
      name: 'file-desktop',
      use: { ...devices['Desktop Chrome'], channel: 'chrome', viewport: { width: 1440, height: 900 } },
    },
    {
      name: 'file-narrow',
      use: { ...devices['Desktop Chrome'], channel: 'chrome', viewport: { width: 390, height: 844 } },
    },
  ],
});
