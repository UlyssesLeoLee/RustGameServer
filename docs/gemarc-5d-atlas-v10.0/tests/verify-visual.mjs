/**
 * verify-visual.mjs — 多维度真实浏览器截图 + 控制台错误采集
 *
 * 目的: 交付前的"看得见"验证 —— 每个维度截一张图, 收集控制台错误。
 * 这是 spec `visual_tests` 的基础档 (维度 × 典型状态 × 视口)。
 *
 * 用法: node tests/verify-visual.mjs [outDir]
 */

import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve, join } from 'node:path';
import { mkdirSync, writeFileSync } from 'node:fs';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const OUT = resolve(process.cwd(), process.argv[2] ?? 'test-results/visual');
mkdirSync(OUT, { recursive: true });

const DIMS = ['1D', '2D', '3D', '4D', '5D', 'Prototype'];
const browser = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});

const report = { index: INDEX, at: new Date().toISOString(), viewports: [], errors: [] };

for (const vp of [{ w: 1440, h: 900, name: 'desktop' }, { w: 390, h: 844, name: 'narrow' }]) {
  const ctx = await browser.newContext({ viewport: { width: vp.w, height: vp.h } });
  const page = await ctx.newPage();
  const errs = [];
  page.on('pageerror', (e) => errs.push(`[pageerror] ${e.message}`));
  page.on('console', (m) => { if (m.type() === 'error') errs.push(`[console] ${m.text()}`); });

  await page.goto(INDEX);
  await page.waitForTimeout(700);

  for (const dim of DIMS) {
    const tab = page.locator(`.dim-tab[data-dim="${dim}"]`);
    if (await tab.count() === 0) { report.errors.push(`${vp.name}/${dim}: 标签不存在`); continue; }
    await tab.click();
    await page.waitForTimeout(900);

    const state = await page.evaluate(() => ({
      dim: document.getElementById('app')?.getAttribute('data-dimension'),
      panelLen: document.getElementById('dimension-panel')?.innerHTML.length ?? 0,
      sidebarLen: document.getElementById('sidebar-body')?.innerHTML.length ?? 0,
      toolbarLen: document.getElementById('canvas-toolbar')?.innerHTML.length ?? 0,
      timelineHidden: document.getElementById('timeline')?.hasAttribute('hidden') ?? true,
      status: document.getElementById('status-text')?.textContent ?? '',
      inspectorLen: document.getElementById('inspector-body')?.innerHTML.length ?? 0,
      hasCanvas: !!document.querySelector('#dimension-panel svg, #dimension-panel canvas, #dimension-panel .konvajs-content'),
      hasTable: !!document.querySelector('#dimension-panel table'),
    }));

    const file = join(OUT, `${vp.name}-${dim}.png`);
    await page.screenshot({ path: file, fullPage: false });
    report.viewports.push({ viewport: vp.name, dim, state, screenshot: file });
    console.log(`[shot] ${vp.name}/${dim}  panel=${state.panelLen} side=${state.sidebarLen} status="${String(state.status).slice(0, 60)}"`);
  }

  if (errs.length) report.errors.push(`${vp.name}: ${errs.join(' | ')}`);
  await ctx.close();
}

await browser.close();
writeFileSync(join(OUT, 'visual-report.json'), JSON.stringify(report, null, 2), 'utf8');
console.log('\n=== 控制台错误 ===');
console.log(report.errors.length ? report.errors.join('\n') : '无');
console.log(`截图 ${report.viewports.length} 张 → ${OUT}`);
