/**
 * probe-chain-visual.mjs — 次元チェーンと 2D デモツアーの視覚確認用スクリーンショット
 */
import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { mkdirSync } from 'node:fs';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const OUT = resolve(process.cwd(), 'qa/visual');
mkdirSync(OUT, { recursive: true });

const b = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});
const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
const errs = [];
p.on('pageerror', (e) => errs.push(e.message));
p.on('console', (m) => { if (m.type() === 'error') errs.push(m.text()); });

const go = async (d) => { await p.locator(`.dim-tab[data-dim="${d}"]`).click(); await p.waitForTimeout(1100); };

await p.goto(INDEX);
await p.waitForTimeout(500);

// 1) 1D → 2D: Prompt 范围
await go('1D');
const prm = p.locator('#sidebar-body button').filter({ hasText: 'PRM-' }).first();
if (await prm.count()) { await prm.click(); await p.waitForTimeout(500); }
await go('2D');
await p.screenshot({ path: `${OUT}/chain-1D-to-2D.png` });
console.log('chain-1D-to-2D.png');

// 2) 2D デモツアー
await p.locator('[data-act="demo"]').click();
await p.waitForTimeout(900);
await p.screenshot({ path: `${OUT}/chain-2D-demo-tour.png` });
console.log('chain-2D-demo-tour.png');
// 2 ステップ目へ
await p.locator('.d2-demo-card [data-act="next"]').click();
await p.waitForTimeout(800);
await p.screenshot({ path: `${OUT}/chain-2D-demo-step2.png` });
console.log('chain-2D-demo-step2.png');
await p.locator('.d2-demo-card [data-act="skip"]').click();
await p.waitForTimeout(300);

// 3) 2D → 3D: ノード選択が 3D に伝わる
const node = p.locator('#dimension-panel [data-node-id]').first();
if (await node.count()) { await node.click(); await p.waitForTimeout(600); }
await go('3D');
await p.screenshot({ path: `${OUT}/chain-2D-to-3D.png` });
console.log('chain-2D-to-3D.png');

// 4) 3D → 4D: 同じステージで Case 再生
await go('4D');
await p.locator('[data-act="play"]').click();
await p.waitForTimeout(1400);
await p.screenshot({ path: `${OUT}/chain-4D-on-3D-stage.png` });
console.log('chain-4D-on-3D-stage.png');

// 5) 5D split
await go('5D');
await p.selectOption('#d5-mode', 'split');
await p.waitForTimeout(1200);
await p.screenshot({ path: `${OUT}/chain-5D-split.png` });
console.log('chain-5D-split.png');

console.log('errors:', errs.length ? errs : 'なし');
await b.close();
