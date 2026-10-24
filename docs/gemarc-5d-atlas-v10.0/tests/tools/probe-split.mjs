import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const b = await chromium.launch({
  channel: 'chrome', headless: true,
  args: ['--allow-file-access-from-files', '--enable-webgl', '--use-gl=swiftshader'],
});
const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
const errs = [];
p.on('pageerror', (e) => errs.push(e.message));
await p.goto(INDEX);
await p.waitForTimeout(500);
await p.locator('.dim-tab[data-dim="5D"]').click();
await p.waitForTimeout(1400);

const before = await p.evaluate(() => document.getElementById('dimension-panel')?.textContent ?? '');
console.log('panels before:', await p.evaluate(() => document.querySelectorAll('#dimension-panel > div').length));
console.log('has 左 before:', before.includes('左'));

await p.selectOption('#d5-mode', 'split');
await p.waitForTimeout(1200);

const after = await p.evaluate(() => document.getElementById('dimension-panel')?.textContent ?? '');
console.log('mode value:', await p.evaluate(() => document.getElementById('d5-mode')?.value));
console.log('panels after:', await p.evaluate(() => document.querySelectorAll('#dimension-panel > div').length));
console.log('has 左 after:', after.includes('左'));
console.log('has ベースライン after:', after.includes('ベースライン'));
console.log('first 400:', JSON.stringify(after.slice(0, 400)));
console.log('errors:', errs);
await b.close();
