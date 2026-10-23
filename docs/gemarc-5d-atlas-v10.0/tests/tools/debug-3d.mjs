import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const browser = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const logs = [];
page.on('pageerror', (e) => logs.push(`[pageerror] ${e.message}\n${e.stack ?? ''}`));
page.on('console', (m) => { if (m.type() === 'error') logs.push(`[console] ${m.text()}`); });

await page.goto(INDEX);
await page.waitForTimeout(500);
await page.locator('.dim-tab[data-dim="3D"]').click();
await page.waitForTimeout(1500);

console.log('=== 日志 ===');
console.log(logs.join('\n---\n') || '(无)');
console.log('\n=== 3D 面板内容 ===');
console.log((await page.locator('#dimension-panel').innerHTML()).slice(0, 900));
console.log('\n=== 工具栏 ===');
console.log((await page.locator('#canvas-toolbar').innerHTML()).slice(0, 400));

await browser.close();
