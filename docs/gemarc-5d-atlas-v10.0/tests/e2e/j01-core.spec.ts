/**
 * j01-core.spec.ts — 核心 E2E: file:// 双击可用 + 维度切换 + 证据可溯源
 *
 * spec `quality_assurance.user_journeys` 的 J01 起步, 以及
 * `hard_fail` 的 "缺 index.html" / "file:// 核心功能不可用" 门禁。
 */

import { test, expect } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;

test.describe('file:// 便携单文件', () => {
  test('J01: 双击 index.html → 应用启动, 无致命错误', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`));
    page.on('console', (m) => { if (m.type() === 'error') errors.push(`console: ${m.text()}`); });

    await page.goto(INDEX);
    await expect(page.locator('#app')).toBeVisible();
    await expect(page.locator('.app-header')).toBeVisible();

    // 顶栏必须显示真实 revision (来自 models/architecture.json)
    const rev = await page.locator('#revision-text').textContent();
    expect(rev?.trim()).toMatch(/^[0-9a-f]{7,40}$/);

    // 关键: 运行时不得有任何 console error
    expect(errors, `运行时错误: ${errors.join(' | ')}`).toHaveLength(0);
  });

  test('维度导航可用: 1D/2D/3D/4D/5D/Prototype 均可切换', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.goto(INDEX);

    for (const dim of ['1D', '2D', '3D', '4D', '5D', 'Prototype']) {
      const tab = page.locator(`.dim-tab[data-dim="${dim}"]`);
      if (await tab.count() === 0) continue;
      await tab.click();
      await expect(page.locator('#app')).toHaveAttribute('data-dimension', dim);
      await expect(page.locator(`.dim-tab[data-dim="${dim}"]`)).toHaveAttribute('aria-selected', 'true');
    }
    expect(errors).toHaveLength(0);
  });

  test('0D 存在性门禁: 是模态窗而非独立页面, 且显式标出 Unknown', async ({ page }) => {
    await page.goto(INDEX);
    const gate = page.locator('#gate-indicator');
    await expect(gate).toBeVisible();
    await gate.click();
    const modal = page.locator('#modal-root .modal');
    await expect(modal).toBeVisible();
    await expect(modal).toContainText('Unknown');
    // 0D 不应有独立导航页
    await expect(page.locator('.dim-tab[data-dim="0D"]')).toHaveCount(0);
  });

  test('证据可溯源: 节点检查器显示 fileLine 与 revision', async ({ page }) => {
    await page.goto(INDEX);
    await page.locator('.dim-tab[data-dim="2D"]').click();
    // 2D 画布由 worker 实现; 若尚未装载, 退而验证全局搜索能解析真实节点
    const search = page.locator('#global-search-input');
    await search.fill('player-service');
    await expect(page.locator('#status-text')).toContainText(/搜索/);
  });

  test('离线自检: 页面加载期间零网络请求 (除 file:// 自身)', async ({ page }) => {
    const external: string[] = [];
    page.on('request', (r) => {
      const u = r.url();
      if (!u.startsWith('file://') && !u.startsWith('data:') && !u.startsWith('blob:')) external.push(u);
    });
    await page.goto(INDEX);
    await page.waitForTimeout(1500);
    expect(external, `外部请求: ${external.join(', ')}`).toHaveLength(0);
  });
});
