/**
 * probe-responsive.mjs — 390/768 ビューポートで横溢れが出ていないか実測
 *
 * 対抗審査 B の発見: 390px で app 外殻が 713px まで MLE し、横スクロールが発生していた。
 * CSS の grid 列 sizing (auto → max-content) が原因。
 */
import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const browser = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});

const DIMS = ['1D', '2D', '3D', '4D', '5D', 'Prototype'];
let worst = 0;

for (const vp of [{ w: 390, h: 844, n: 'mobile' }, { w: 768, h: 1024, n: 'tablet' }, { w: 1440, h: 900, n: 'desktop' }]) {
  const ctx = await browser.newContext({ viewport: { width: vp.w, height: vp.h } });
  const page = await ctx.newPage();
  await page.goto(INDEX);
  await page.waitForTimeout(600);

  for (const dim of DIMS) {
    await page.locator(`.dim-tab[data-dim="${dim}"]`).click();
    await page.waitForTimeout(600);
    const m = await page.evaluate(() => ({
      app: Math.round(document.getElementById('app').getBoundingClientRect().width),
      body: Math.round(document.querySelector('.app-body').getBoundingClientRect().width),
      docScroll: document.documentElement.scrollWidth,
      vw: window.innerWidth,
      inspectorVisible: (() => {
        const el = document.getElementById('app-inspector');
        const r = el.getBoundingClientRect();
        // 閉じていても描画可能か (display:none でないか) を確認
        return getComputedStyle(el).display !== 'none';
      })(),
    }));
    const over = Math.max(m.app, m.body, m.docScroll) - m.vw;
    if (over > worst) worst = over;
    console.log(
      `  ${vp.n.padEnd(8)} ${dim.padEnd(10)} app=${m.app} body=${m.body} docScroll=${m.docScroll} vw=${m.vw} overflow=${over > 0 ? '❌ +' + over : '✅'} inspector=${m.inspectorVisible ? '可用' : '❌none'}`,
    );
  }
  await ctx.close();
}
await browser.close();
console.log(`\n最大横溢れ: ${worst > 0 ? '❌ ' + worst + 'px' : '✅ 0px'}`);
process.exit(worst > 0 ? 1 : 0);
