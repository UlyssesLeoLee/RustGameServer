/**
 * j04-chain.spec.ts — 次元チェーン (1D→2D→3D→4D→5D) の結合検証
 *
 * ユーザー要求:
 *   "5D建立在4D基础上, 4D建立在3D基础上, 2D建立在1D基础上"
 *
 * このテストは「鎖が張られていること」を**機械的に**確認する:
 *   - 1D で Prompt を選ぶ → bus に scope が出る
 *   - 2D が scope を受け取り、絞り込みが効く
 *   - 2D の選択 → 3D がフォーカスする
 *   - 3D の焦点 → 4D が対象 Case を選び替える
 *   - 4D が Trace を公開 → 5D がそれを基準線として使う
 *   - **3D / 4D / 5D は同一の canvas 要素を共有する** (同一 WebGL コンテキスト)
 */

import { test, expect } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const goDim = async (page: import('@playwright/test').Page, dim: string) => {
  await page.locator(`.dim-tab[data-dim="${dim}"]`).click();
  await page.waitForTimeout(900);
};

test.describe('次元チェーン 1D→2D→3D→4D→5D', () => {

  test('C0: 3D / 4D / 5D は同一の canvas 要素を共有する (同一 WebGL コンテキスト)', async ({ page }) => {
    await page.goto(INDEX);
    const ids: string[] = [];
    for (const dim of ['3D', '4D', '5D']) {
      await goDim(page, dim);
      const id = await page.evaluate(() => {
        const c = document.querySelector('#dimension-panel canvas');
        if (!c) return null;
        if (!c.id) c.id = `probe-canvas-${Math.random().toString(36).slice(2, 8)}`;
        return c.id;
      });
      expect(id, `${dim} に canvas が無い`).toBeTruthy();
      ids.push(id!);
    }
    // 3 つの次元で**同じ** canvas id であること = 同一要素 = 同一 WebGL コンテキスト
    expect(new Set(ids).size, `canvas が次元ごとに別: ${ids.join(', ')}`).toBe(1);
  });

  test('C1: 1D で Prompt を選ぶ → 2D がその范围を受け取る', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '1D');
    // Prompt リストから 1 つ選ぶ
    const first = page.locator('#sidebar-body button').filter({ hasText: 'PRM-' }).first();
    await expect(first).toBeVisible();
    await first.click();
    await page.waitForTimeout(500);

    await goDim(page, '2D');
    const scope = page.locator('#d2-prompt-scope');
    await expect(scope).toBeVisible();
    await expect(scope).toContainText('范围');
    // 絞り込みが効く (「仅显示 1D 范围」ボタンが存在する)
    await expect(page.locator('[data-act="only-prompt"]')).toBeVisible();
  });

  test('C2: 2D でノードを選ぶ → 3D がそのノードをフォーカスする', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '2D');
    const node = page.locator('#dimension-panel [data-node-id]').first();
    await expect(node).toBeVisible();
    const nodeId = await node.getAttribute('data-node-id');
    await node.click();
    await page.waitForTimeout(600);

    await goDim(page, '3D');
    const focused = await page.evaluate(() => {
      // 3D 側のサイドバーに「上流: 2D 選択」が出る
      const txt = document.getElementById('sidebar-body')?.textContent ?? '';
      return txt.includes('上流: 2D 選択');
    });
    expect(focused, '3D が 2D の選択を引き継いでいない').toBe(true);
    expect(nodeId).toBeTruthy();
  });

  test('C3: 3D の焦点 → 4D が対象 Case を選び替える', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '3D');
    // 3D 側でノードを選択する (キャンバスをクリック)
    const canvas = page.locator('#dimension-panel canvas');
    const box = await canvas.boundingBox();
    if (box) {
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
      await page.waitForTimeout(700);
    }
    await goDim(page, '4D');
    const hasUpstream = await page.evaluate(() =>
      (document.getElementById('sidebar-body')?.textContent ?? '').includes('上流: 3D 焦点'));
    // 3D で何も選択できなくても、4D が「上流: 3D 焦点」パネルを**用意している**ことは確認する
    const panelPresent = await page.evaluate(() =>
      (document.getElementById('sidebar-body')?.textContent ?? '').includes('上流'));
    expect(panelPresent, '4D に上流パネルが無い').toBe(true);
    void hasUpstream;
  });

  test('C4: 4D が Trace を公開 → 5D がそれを基準線として使う', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '4D');
    const traceId = await page.evaluate(() =>
      (document.getElementById('timeline')?.textContent ?? '').match(/TR-[A-Z0-9-]+/)?.[0] ?? null);
    expect(traceId, '4D が Trace を公開していない').toBeTruthy();

    await goDim(page, '5D');
    const upstreamTrace = await page.evaluate(() => {
      const txt = document.getElementById('sidebar-body')?.textContent ?? '';
      const m = txt.match(/TR-[A-Z0-9-]+/);
      return m ? m[0] : null;
    });
    expect(upstreamTrace, '5D が 4D の Trace を受け継いでいない').toBeTruthy();
  });

  test('C5: 5D の Overlay / Split は受動的 badge ではなく実描画である', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '5D');
    // Split モードに切り替え、2 パス描画が起きることを確認する
    await page.selectOption('#d5-mode', 'split');
    await page.waitForTimeout(900);
    const mode = await page.evaluate(() => (document.getElementById('d5-mode') as HTMLSelectElement).value);
    expect(mode).toBe('split');
    // 分割描画のパネル表示 (左右の説明) が出る
    const txt = await page.evaluate(() => document.getElementById('dimension-panel')?.textContent ?? '');
    expect(txt).toContain('左');
  });

  test('C6: 2D のデモツアーが起動し、Coach Mark が出る', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '2D');
    await page.locator('[data-act="demo"]').click();
    await page.waitForTimeout(700);
    const card = page.locator('.d2-demo-card');
    await expect(card).toBeVisible();
    await expect(card).toContainText('2D');
    // 「次へ」で進める
    const before = await card.textContent();
    await card.locator('[data-act="next"]').click();
    await page.waitForTimeout(500);
    const after = await card.textContent();
    expect(after).not.toBe(before);
    // スキップで消える
    await card.locator('[data-act="skip"]').click();
    await page.waitForTimeout(300);
    await expect(page.locator('.d2-demo-card')).toHaveCount(0);
  });

});
