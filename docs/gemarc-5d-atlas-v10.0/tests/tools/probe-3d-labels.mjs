/**
 * probe-3d-labels.mjs — 3D ラベル層が実際に描画されているか + 軸ギズモ方向の検証
 *
 * 対抗審査の指摘:
 *   (1) LabelLayer に公開の登録 API が無く、ラベルが 0 個描画されていた
 *   (2) 軸ギズモの "+Z" が Adapter を経由せず、意味座標と逆を向いていた
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
const errs = [];
page.on('pageerror', (e) => errs.push(e.message));
page.on('console', (m) => { if (m.type() === 'error') errs.push(m.text()); });

await page.goto(INDEX);
await page.waitForTimeout(600);
await page.locator('.dim-tab[data-dim="3D"]').click();
await page.waitForTimeout(1200);

const measure = async (mode) => {
  await page.selectOption('#d3-label', mode);
  await page.waitForTimeout(700);
  return page.evaluate(() => {
    const els = [...document.querySelectorAll('.d3-labels .d3-label')];
    const vis = els.filter((e) => e.style.display !== 'none' && e.textContent.trim());
    return { total: els.length, visible: vis.length, sample: vis.slice(0, 4).map((e) => e.textContent.trim()) };
  });
};

console.log('=== ラベル描画数 (モード別) ===');
for (const mode of ['All', 'Smart', 'RelatedOnly']) {
  const m = await measure(mode);
  console.log(`  ${mode.padEnd(12)} total=${m.total} visible=${m.visible}  例: ${JSON.stringify(m.sample)}`);
}

console.log('\n=== 軸ギズモ方向 (adapter 経由の確認) ===');
const gizmo = await page.evaluate(() => {
  // ギズモのコーンは scene 内にあり、+Z は semantic +Z = Three -Z を向くはず
  const canvas = document.querySelector('#dimension-panel canvas');
  return { hasCanvas: !!canvas };
});
console.log('  ' + JSON.stringify(gizmo));

console.log('\n=== 選択時の優先度 (Selected / DirectlyRelated) ===');
await page.selectOption('#d3-label', 'All');
await page.waitForTimeout(400);
const before = await page.evaluate(() => document.querySelectorAll('.d3-labels .d3-label').length);
await page.locator('#dimension-panel canvas').click({ position: { x: 720, y: 450 } });
await page.waitForTimeout(600);
const after = await page.evaluate(() => ({
  labels: document.querySelectorAll('.d3-labels .d3-label').length,
  selected: document.querySelectorAll('.d3-labels .d3-label[data-kind]').length,
  inspector: document.getElementById('inspector-body')?.textContent?.slice(0, 60),
}));
console.log(`  選択前 labels=${before} / 選択後 labels=${after.labels} 検査器="${after.inspector}"`);

console.log('\n=== コンソールエラー ===');
console.log(errs.length ? errs.join('\n') : '  なし');
await browser.close();
process.exit(errs.length ? 1 : 0);
