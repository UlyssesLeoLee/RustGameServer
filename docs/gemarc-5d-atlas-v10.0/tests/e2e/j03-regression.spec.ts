/**
 * j03-regression.spec.ts — 対抗審査で見つかった不具合の回帰防止
 *
 * spec `quality_assurance.regression` の実測項目を、そのまま E2E として固定する。
 * 各ケースは「その不具合が再発したら落ちる」ことを狙ったもの。
 */

import { test, expect } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const goDim = async (page: import('@playwright/test').Page, dim: string) => {
  await page.locator(`.dim-tab[data-dim="${dim}"]`).click();
  await page.waitForTimeout(800);
};

test.describe('対抗審査Finding の回帰防止', () => {

  test('R1: SQL コメント内の CREATE TABLE を拾わない (幻影テーブル "IF" 0 件)', () => {
    const m = JSON.parse(readFileSync(resolve(process.cwd(), 'models/architecture.json'), 'utf8'));
    const tables = m.nodes.filter((n: { kind: string }) => n.kind === 'Table');
    expect(tables.filter((t: { name: string }) => /^(IF|NOT|EXISTS|TABLE)$/i.test(t.name))).toHaveLength(0);
    // 重複 qualifiedName 0 件
    const names = tables.map((t: { qualifiedName: string }) => t.qualifiedName);
    expect(new Set(names).size).toBe(names.length);
    // PK を持つ表が 1 つ以上ある (PK 検出が壊れていない)
    expect(tables.filter((t: { properties: { pkColumns?: string[] } }) => (t.properties.pkColumns ?? []).length).length)
      .toBeGreaterThan(0);
  });

  test('R2: Implemented 辺に use インポート/文字列リテラルが混ざっていない', () => {
    const m = JSON.parse(readFileSync(resolve(process.cwd(), 'models/architecture.json'), 'utf8'));
    const ev = JSON.parse(readFileSync(resolve(process.cwd(), 'models/evidence.json'), 'utf8'));
    const byId = new Map(ev.records.map((r: { id: string }) => [r.id, r]));
    for (const e of m.edges.filter((x: { verification: string }) => x.verification === 'Implemented')) {
      const rec = byId.get(e.evidenceIds[0]);
      const snippet = String(rec?.snippet ?? '').trim();
      expect(snippet, `${e.id} が use インポート`).not.toMatch(/^use\s/);
      const withoutStrings = snippet.replace(/"(?:[^"\\]|\\.)*"/g, '""').replace(/'(?:[^'\\]|\\.)*'/g, "''");
      expect(withoutStrings, `${e.id} が文字列リテラル内のみ`).toContain(`${e.properties.calleeService}Client`);
    }
  });

  test('R3: 3D ラベルが 1 つも描画されない (LabelLayer 公開 API 欠落)', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '3D');
    const n = await page.evaluate(() => {
      const els = [...document.querySelectorAll('.d3-labels .d3-label')];
      return { total: els.length, visible: els.filter((e) => e.style.display !== 'none' && e.textContent?.trim()).length };
    });
    expect(n.total).toBeGreaterThan(0);
    expect(n.visible).toBeGreaterThan(0);
  });

  test('R4: Prototype プレビューがクラッシュしない (tabs/items がオブジェクト配列)', async ({ page }) => {
    const errs: string[] = [];
    page.on('pageerror', (e) => errs.push(e.message));
    await page.goto(INDEX);
    await goDim(page, 'Prototype');
    await page.locator('[data-sub="preview"]').click();
    await page.waitForTimeout(1200);
    const st = await page.evaluate(() => ({
      children: document.getElementById('dimension-panel')?.childElementCount ?? 0,
      cids: document.querySelectorAll('#dimension-panel [data-cid]').length,
      inputs: document.querySelectorAll('#dimension-panel input,#dimension-panel textarea,#dimension-panel select').length,
    }));
    expect(st.children).toBeGreaterThan(0);
    expect(st.cids).toBeGreaterThan(0);
    expect(st.inputs).toBeGreaterThan(0);      // 実 DOM のフォームが存在する
    expect(errs).toHaveLength(0);
  });

  test('R5: 狭幅 (390px) で横溢れが出ず、検査器/目录のハンドルに到達できる', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(INDEX);
    await page.waitForTimeout(500);
    // ヘッダーのハンドルが表示されている (旧実装はドロワー内 → 画面外)
    await expect(page.locator('#btn-toggle-right')).toBeVisible();
    await expect(page.locator('#btn-toggle-left')).toBeVisible();
    await page.locator('#btn-toggle-right').click();
    await page.waitForTimeout(400);
    const m = await page.evaluate(() => ({
      app: Math.round(document.getElementById('app')!.getBoundingClientRect().width),
      scroll: document.documentElement.scrollWidth,
      vw: window.innerWidth,
      inspectorX: Math.round(document.getElementById('app-inspector')!.getBoundingClientRect().x),
    }));
    expect(m.app).toBeLessThanOrEqual(m.vw);
    expect(m.scroll).toBeLessThanOrEqual(m.vw);
    expect(m.inspectorX).toBeLessThan(m.vw);    // 検査器が画面内
  });

  test('R6: 2D の外殻リスナが累積しない (13 回往復 → 1 クリックで 1 回だけ)', async ({ page }) => {
    await page.addInitScript(() => {
      (window as unknown as { __inv: Record<string, number> }).__inv = {};
      const orig = EventTarget.prototype.addEventListener;
      EventTarget.prototype.addEventListener = function (type: string, fn: EventListener, opts?: unknown) {
        const key = `${this.id || this.className}|${type}`;
        const wrapped: EventListener = (ev: Event) => {
          const w = window as unknown as { __inv: Record<string, number> };
          w.__inv[key] = (w.__inv[key] ?? 0) + 1;
          return fn.call(this, ev);
        };
        return orig.call(this, type, wrapped, opts);
      };
    });
    await page.goto(INDEX);
    await page.waitForTimeout(500);
    for (let i = 0; i < 13; i++) {
      await page.locator('.dim-tab[data-dim="2D"]').click();
      await page.waitForTimeout(180);
      await page.locator('.dim-tab[data-dim="1D"]').click();
      await page.waitForTimeout(180);
    }
    await page.locator('.dim-tab[data-dim="2D"]').click();
    await page.waitForTimeout(500);
    await page.evaluate(() => { (window as unknown as { __inv: Record<string, number> }).__inv = {}; });
    await page.locator('#sidebar-body [data-diagram]').first().click();
    await page.waitForTimeout(500);
    const inv = await page.evaluate(() => (window as unknown as { __inv: Record<string, number> }).__inv);
    for (const [k, v] of Object.entries(inv)) {
      if (k.startsWith('canvas-toolbar|') || k.startsWith('sidebar-body|')) {
        expect(v, `${k} が累積`).toBe(1);
      }
    }
  });

  test('R7: 5D は無干渉で NO_DIFFERENCE (偽分歧を作らない)', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '5D');
    await expect(page.locator('#d5-verdict')).toContainText('NO_DIFFERENCE');
  });

  test('R8: 5D の図増分が RewireGraph で「削除のみ」にならない', async ({ page }) => {
    await page.goto(INDEX);
    await goDim(page, '5D');
    await page.selectOption('#d5-op', 'RewireGraph');
    await page.waitForTimeout(300);
    // 新接続先のセレクトが現れる (無いと削除のみになる)
    await expect(page.locator('#d5-rewire')).toBeVisible();
  });

});
