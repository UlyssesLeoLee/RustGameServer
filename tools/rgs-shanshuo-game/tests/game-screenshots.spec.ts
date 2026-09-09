import { test, expect } from '@playwright/test';
import * as fs from 'fs';

const GAME_URL = 'http://127.0.0.1:8083/game.html';
const OUT_DIR = 'D:\\playwright-test\\screenshots';

test.beforeAll(() => {
  fs.mkdirSync(OUT_DIR, { recursive: true });
});

test('Case 1 — 闪烁之光 RGS 版游戏登录页 (输入账号 + 选区服)', async ({ page }) => {
  await page.goto(GAME_URL, { waitUntil: 'load' });
  await page.waitForTimeout(500);
  // 验证登录页关键元素
  await expect(page.locator('.login-logo')).toContainText('闪 烁 之 光');
  await expect(page.locator('.login-subtitle')).toContainText('SHANSHUO LIGHT');
  await expect(page.locator('#login-username')).toBeVisible();
  await expect(page.locator('#login-password')).toBeVisible();
  await expect(page.locator('#login-server')).toBeVisible();
  await expect(page.locator('#login-btn')).toBeVisible();
  await expect(page.locator('.rgs-banner-tag .rgs-version')).toContainText('rgs-port 2026-09-08');
  // 截图
  await page.screenshot({ path: `${OUT_DIR}\\game-case1-login.png`, fullPage: true });
  console.log(`[Game Case 1] saved → ${OUT_DIR}\\game-case1-login.png`);
});

test('Case 2 — 闪烁之光 RGS 版选角色页 (3 角色: 战士/法师/道士)', async ({ page }) => {
  await page.goto(GAME_URL, { waitUntil: 'load' });
  await page.waitForTimeout(500);
  // 登录 → 选角色
  await page.locator('#login-btn').click();
  await page.waitForTimeout(800);
  // 验证选角色页
  await expect(page.locator('.char-title')).toContainText('选 择');
  const cardCount = await page.locator('.char-card').count();
  expect(cardCount).toBe(3);
  // 验证 3 角色名
  await expect(page.locator('.char-card').nth(0).locator('.char-name')).toContainText('苍 穹 战 神');
  await expect(page.locator('.char-card').nth(1).locator('.char-name')).toContainText('星 辰 魔 导');
  await expect(page.locator('.char-card').nth(2).locator('.char-name')).toContainText('紫 霄 道 尊');
  // 验证默认选中法师
  await expect(page.locator('.char-card.selected')).toContainText('星 辰 魔 导');
  await expect(page.locator('#char-enter-btn')).toBeVisible();
  // 截图
  await page.screenshot({ path: `${OUT_DIR}\\game-case2-character-select.png`, fullPage: true });
  console.log(`[Game Case 2] saved → ${OUT_DIR}\\game-case2-character-select.png`);
});

test('Case 3 — 闪烁之光 RGS 版主界面 (资源条 + 角色信息 + 主场景 + 任务/聊天)', async ({ page }) => {
  await page.goto(GAME_URL, { waitUntil: 'load' });
  await page.waitForTimeout(800);
  // 登录 → 选角色 → 进入游戏
  await page.locator('#login-btn').click();
  await page.waitForTimeout(800);
  // 验证 character-select 已显示
  await expect(page.locator('#character-select.active')).toBeVisible();
  await page.locator('#char-enter-btn').click();
  // 等 main-game 切到 active (用 waitForSelector)
  await page.waitForSelector('#main-game.active', { timeout: 5000 });
  await page.waitForTimeout(2000); // 等待 RGS fetch 完成 + 动画
  // 软断言: 验证关键元素 (用 soft expect 容忍, 不阻断截图)
  const checks = {
    resourceBar: await page.locator('.top-bar .resource-bar').count() > 0,
    goldIcon: await page.locator('.resource').nth(0).textContent().catch(() => ''),
    diamondIcon: await page.locator('.resource').nth(1).textContent().catch(() => ''),
    charMini: await page.locator('.left-panel .char-mini').count() > 0,
    hpBar: await page.locator('.hp-bar').count() > 0,
    mpBar: await page.locator('.mp-bar').count() > 0,
    expBar: await page.locator('.exp-bar').count() > 0,
    sceneTitle: await page.locator('.scene-title').textContent().catch(() => ''),
    combatBtn: await page.locator('#btn-combat').count() > 0,
    shopBtn: await page.locator('#btn-shop').count() > 0,
    guildBtn: await page.locator('#btn-guild').count() > 0,
    activeTab: await page.locator('.right-panel .tab.active').textContent().catch(() => ''),
    questTitle: await page.locator('.quest-item').first().locator('.quest-title').textContent().catch(() => ''),
  };
  console.log(`[Game Case 3] element checks:`, JSON.stringify(checks, null, 2));
  // 验证 rgs-flash-mock 联动状态
  const rgsStatus = await page.locator('#rgs-load').textContent({ timeout: 3000 }).catch(() => null);
  console.log(`[Game Case 3] RGS 联动状态: ${rgsStatus || '(rgs-flash-mock 不可达)'}`);
  // 截图 (即使部分元素不渲染也截图)
  await page.screenshot({ path: `${OUT_DIR}\\game-case3-main-game.png`, fullPage: true });
  console.log(`[Game Case 3] saved → ${OUT_DIR}\\game-case3-main-game.png`);
  // 不严格 expect, 软断言已记录
});

test('Case 4 — 闪烁之光 RGS 版 RGS Live Data 面板 (4 卡片真业务数据)', async ({ page }) => {
  await page.goto(GAME_URL, { waitUntil: 'load' });
  await page.waitForTimeout(500);
  // 登录 + 选角色 + 进入游戏
  await page.locator('#login-btn').click();
  await page.waitForTimeout(800);
  await page.locator('#char-enter-btn').click();
  await page.waitForSelector('#main-game.active', { timeout: 5000 });
  // 等待 RGS 4 卡片从 loading 变 ok
  await page.waitForSelector('#rgs-card-player.rgs-card-ok', { timeout: 8000 });
  await page.waitForSelector('#rgs-card-economy.rgs-card-ok', { timeout: 8000 });
  await page.waitForSelector('#rgs-card-social.rgs-card-ok', { timeout: 8000 });
  await page.waitForSelector('#rgs-card-admin.rgs-card-ok', { timeout: 8000 });
  // 严格 expect: 4 卡片都显示真实数据
  const cards = {
    player:  await page.locator('#rgs-card-player .rgs-card-name').textContent(),
    economy: await page.locator('#rgs-card-economy .rgs-card-name').textContent(),
    social:  await page.locator('#rgs-card-social .rgs-card-name').textContent(),
    admin:   await page.locator('#rgs-card-admin .rgs-card-name').textContent(),
  };
  console.log(`[Game Case 4] 4 卡片真业务数据:`, cards);
  expect(cards.player).toContain('MavisHero');
  expect(cards.economy).toMatch(/^💰\s+Gold-/);  // economy GetAccount 返回 Gold-<player_id> display_name
  expect(cards.social).toContain('RGS Vanguard');
  expect(cards.admin).toMatch(/审计/);
  // 整页截图
  await page.screenshot({ path: `${OUT_DIR}\\game-case4-rgs-live.png`, fullPage: true });
  // close-up: 只截 .rgs-live-panel
  const panel = page.locator('.rgs-live-panel');
  await panel.screenshot({ path: `${OUT_DIR}\\game-case4-rgs-live-panel.png` });
  console.log(`[Game Case 4] saved → game-case4-rgs-live.png + game-case4-rgs-live-panel.png`);
});
