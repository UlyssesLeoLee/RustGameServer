import { defineConfig } from 'vitest/config';

/**
 * vitest 設定 — 単体テストのみ
 *
 * 陷阱: vitest の既定 include は「全 .test/.spec ファイル」なので、
 * Playwright の E2E ファイル (tests/e2e/ 配下の spec) まで拾って
 * 「Playwright Test did not expect test.describe() to be called here」で落ちる。
 * include を tests/unit に限定して**ディレクトリ境界**を守る。
 */
export default defineConfig({
  test: {
    include: ['tests/unit/**/*.test.ts'],
    exclude: ['node_modules/**', 'tests/e2e/**', 'dist/**', '.build/**'],
    environment: 'node',
    reporters: process.env.CI ? ['basic'] : ['basic'],
    coverage: {
      provider: 'v8',
      include: ['src/sim/**', 'src/core/graph.ts'],
      reporter: ['text-summary', 'json-summary'],
      reportsDirectory: 'qa/coverage',
    },
  },
});
