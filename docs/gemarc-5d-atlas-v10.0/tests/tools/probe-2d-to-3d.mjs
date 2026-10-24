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

await p.locator('.dim-tab[data-dim="2D"]').click();
await p.waitForTimeout(1200);
const n = p.locator('#dimension-panel [data-node-id]').first();
const nid = await n.getAttribute('data-node-id');
await n.click();
await p.waitForTimeout(700);
console.log('clicked 2D node:', nid);
console.log('inspector after click:', (await p.locator('#inspector-body').textContent())?.slice(0, 70));

await p.locator('.dim-tab[data-dim="3D"]').click();
await p.waitForTimeout(1400);
const sb = await p.evaluate(() => document.getElementById('sidebar-body')?.textContent ?? '');
console.log('3D sidebar has "上游: 2D 選択" =', sb.includes('上游: 2D 選択'));
console.log('3D sidebar head:', JSON.stringify(sb.slice(0, 260)));
console.log('errors:', errs);
await b.close();
