/**
 * probe-causal-prototype.mjs — 対抗審査 C の T7/T8/T9/T10 の修正確認
 *
 * T7: 無干渉 / 無関係干渉 → NO_DIFFERENCE (偽分歧を出さない)
 * T8: AddInteractiveElement が**本物の A→X→B 改写**になる (旧辺が実際に消える)
 * T9: RewireGraph が from/to を持ち、削除のみにならない
 * T10: Prototype プレビューが**クラッシュしない** / Konva ドラッグ → DOM 同期
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
page.on('pageerror', (e) => errs.push(`[pageerror] ${e.message}`));
page.on('console', (m) => { if (m.type() === 'error') errs.push(`[console] ${m.text()}`); });

const goDim = async (d) => {
  await page.locator(`.dim-tab[data-dim="${d}"]`).click();
  await page.waitForTimeout(800);
};
const verdict = () => page.locator('#d5-verdict').textContent();
const panelText = () => page.locator('#dimension-panel').innerText();

console.log('=== T7: 干渉なし → NO_DIFFERENCE ===');
await page.goto(INDEX);
await goDim('5D');
console.log(`  verdict = ${await verdict()}`);

console.log('\n=== T7: 経路外ノードへの干渉 → 依然 NO_DIFFERENCE ===');
await page.selectOption('#d5-op', 'InjectFault');
await page.waitForTimeout(200);
// gm-backend 以外のノード (経路外) を選ぶ
const opt = await page.evaluate(() => {
  const s = document.getElementById('d5-target');
  const o = [...s.options].find((x) => !x.textContent.includes('gm-backend'));
  s.value = o.value;
  return o.textContent;
});
await page.locator('[data-act="add"]').click();
await page.waitForTimeout(900);
console.log(`  対象 = ${opt.trim()}`);
console.log(`  verdict = ${await verdict()}`);

console.log('\n=== T8: AddInteractiveElement → 本物の A→X→B 改写 ===');
await page.goto(INDEX);
await goDim('5D');
await page.selectOption('#d5-op', 'AddInteractiveElement');
await page.waitForTimeout(200);
await page.evaluate(() => {
  const s = document.getElementById('d5-target');
  const o = [...s.options].find((x) => x.textContent.includes('gm-backend'));
  s.value = o.value;
});
await page.locator('[data-act="add"]').click();
await page.waitForTimeout(900);
const deltaText = await page.evaluate(() => {
  const panels = [...document.querySelectorAll('#dimension-panel .panel')];
  const p = panels.find((x) => x.textContent.includes('ScenarioGraphDelta'));
  return p ? p.innerText.replace(/\n+/g, ' | ') : '(图增量面板未找到)';
});
console.log(`  ${deltaText.slice(0, 300)}`);
console.log(`  verdict = ${await verdict()}`);

console.log('\n=== T9: RewireGraph に新接続先が要る ===');
await page.goto(INDEX);
await goDim('5D');
await page.selectOption('#d5-op', 'RewireGraph');
await page.waitForTimeout(300);
const rewireVisible = await page.locator('#d5-rewire').isVisible();
console.log(`  新接続先セレクト表示: ${rewireVisible ? '✅' : '❌'}`);

console.log('\n=== T10: Prototype プレビュー (クラッシュしないか) ===');
await page.goto(INDEX);
await goDim('Prototype');
await page.locator('[data-sub="preview"]').click();
await page.waitForTimeout(1200);
const prev = await page.evaluate(() => ({
  panelChildren: document.getElementById('dimension-panel')?.childElementCount ?? 0,
  cids: document.querySelectorAll('#dimension-panel [data-cid]').length,
  realInputs: document.querySelectorAll('#dimension-panel input,#dimension-panel textarea,#dimension-panel select').length,
  tabs: document.querySelectorAll('#dimension-panel [role="tab"]').length,
  navButtons: document.querySelectorAll('#dimension-panel nav button').length,
}));
console.log(`  ${JSON.stringify(prev)}`);

console.log('\n=== T10: Konva ドラッグ → DOM プレビュー同期 ===');
await page.locator('[data-sub="visual"]').click();
await page.waitForTimeout(1200);
const beforePos = await page.evaluate(() => {
  const s = document.getElementById('pt-status');
  return s?.textContent ?? '';
});
// Konva の最初のシェイプをドラッグ
const konvaCanvas = page.locator('#dimension-panel .konvajs-content canvas').first();
const kb = await konvaCanvas.boundingBox();
if (kb) {
  await page.mouse.move(kb.x + 60, kb.y + 40);
  await page.mouse.down();
  await page.mouse.move(kb.x + 160, kb.y + 100, { steps: 8 });
  await page.mouse.up();
  await page.waitForTimeout(600);
}
await page.locator('[data-sub="preview"]').click();
await page.waitForTimeout(900);
const after = await page.evaluate(() => {
  const els = [...document.querySelectorAll('#dimension-panel [data-cid]')];
  const abs = els.filter((e) => getComputedStyle(e).position === 'absolute');
  return {
    total: els.length,
    absolutePositioned: abs.length,
    sample: abs.slice(0, 3).map((e) => ({ cid: e.dataset.cid, left: e.style.left, top: e.style.top })),
  };
});
console.log(`  ${JSON.stringify(after)}`);

console.log('\n=== コンソールエラー ===');
console.log(errs.length ? errs.join('\n') : '  なし');
await browser.close();
