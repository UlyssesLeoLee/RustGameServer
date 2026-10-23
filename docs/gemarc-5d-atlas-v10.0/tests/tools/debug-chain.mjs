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
page.on('pageerror', (e) => errs.push(`[pageerror] ${e.message}\n${(e.stack ?? '').split('\n').slice(0, 6).join('\n')}`));
page.on('console', (m) => { if (m.type() === 'error') errs.push(`[console] ${m.text()}`); });

await page.goto(INDEX);
await page.waitForTimeout(600);

for (const dim of ['1D', '2D', '3D', '4D', '5D', 'Prototype']) {
  errs.length = 0;
  await page.locator(`.dim-tab[data-dim="${dim}"]`).click();
  await page.waitForTimeout(1200);
  const st = await page.evaluate(() => ({
    toolbarLen: document.getElementById('canvas-toolbar')?.innerHTML.length ?? 0,
    panelLen: document.getElementById('dimension-panel')?.innerHTML.length ?? 0,
    sidebarLen: document.getElementById('sidebar-body')?.innerHTML.length ?? 0,
    canvas: !!document.querySelector('#dimension-panel canvas'),
    ids: [...document.querySelectorAll('#canvas-toolbar select')].map((s) => s.id),
  }));
  console.log(`\n=== ${dim} ===`);
  console.log('  ' + JSON.stringify(st));
  if (errs.length) console.log('  ERRORS:\n' + errs.map((e) => '    ' + e).join('\n'));
}
await browser.close();
