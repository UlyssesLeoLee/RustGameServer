/**
 * probe-listener-leak.mjs — 2D のリスナ累積が解消されたか検証
 *
 * 対抗審査の指摘: 外殻常驻要素 (#canvas-toolbar / #sidebar-body) への
 * デリゲートリスナが cleanup で解除されず、13 回 mount → 1 クリックで 13 回描画。
 */
import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const browser = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

// リスナを数える
// 注意: AbortController.abort() は removeEventListener を**呼ばない**ため、
// 「登録数 − 解除数」で測ると常に累積に見える。
// 正しく測るには**実際の呼び出し回数**を数える必要がある。
await page.addInitScript(() => {
  window.__invokeCounts = {};
  const orig = EventTarget.prototype.addEventListener;
  EventTarget.prototype.addEventListener = function (type, fn, opts) {
    const key = `${this.id || this.className || this.tagName}|${type}`;
    const wrapped = function (...a) {
      window.__invokeCounts[key] = (window.__invokeCounts[key] ?? 0) + 1;
      return fn.apply(this, a);
    };
    window.__wrappedRefs = window.__wrappedRefs ?? new Map();
    window.__wrappedRefs.set(fn, wrapped);
    return orig.call(this, type, wrapped, opts);
  };
});

await page.goto(INDEX);
await page.waitForTimeout(600);

// 2D ↔ 1D を 13 回往復
for (let i = 0; i < 13; i++) {
  await page.locator('.dim-tab[data-dim="2D"]').click();
  await page.waitForTimeout(220);
  await page.locator('.dim-tab[data-dim="1D"]').click();
  await page.waitForTimeout(220);
}
await page.locator('.dim-tab[data-dim="2D"]').click();
await page.waitForTimeout(600);

// 呼び出し回数をリセットしてから 1 クリック
await page.evaluate(() => { window.__invokeCounts = {}; });
const before = await page.evaluate(() => ({ ...window.__invokeCounts }));
await page.locator('#sidebar-body [data-diagram]').first().click();
await page.waitForTimeout(600);
const after = await page.evaluate(() => ({ ...window.__invokeCounts }));

console.log('=== 1 クリック後のリスナ呼び出し回数 (累積していれば >1) ===');
let bad = 0;
for (const [k, v] of Object.entries(after)) {
  const shell = k.startsWith('canvas-toolbar|') || k.startsWith('sidebar-body|');
  const flag = shell && v > 1 ? '  <== 累積' : '';
  if (shell && v > 1) bad++;
  console.log(`  ${k.padEnd(34)} ${v}${flag}`);
}
console.log(`\n(クリック前: ${JSON.stringify(before)})`);

await browser.close();
console.log(bad === 0 ? '\n✅ 外殻リスナは累積していない (1 クリックで 1 回だけ発火)' : `\n❌ ${bad} 要素で累積`);
process.exit(bad === 0 ? 0 : 1);
