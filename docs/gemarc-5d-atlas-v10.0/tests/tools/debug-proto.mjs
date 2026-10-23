import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const logs = [];
page.on('pageerror', (e) => logs.push(`[pageerror] ${e.message}\n${e.stack}`));
page.on('console', (m) => { if (m.type() === 'error') logs.push(`[console] ${m.text()}`); });

await page.goto(INDEX);
await page.waitForTimeout(500);
await page.locator('.dim-tab[data-dim="Prototype"]').click();
await page.waitForTimeout(800);

console.log('=== 日志 ===');
console.log(logs.join('\n') || '(无)');

console.log('\n=== tablist innerHTML ===');
console.log(await page.evaluate(() => {
  const tl = document.querySelector('#dimension-panel [role="tablist"]');
  return tl ? tl.outerHTML.slice(0, 700) : 'NO TABLIST FOUND';
}));

console.log('\n=== 所有 button 在 panel 内的文本 ===');
console.log(await page.evaluate(() =>
  [...document.querySelectorAll('#dimension-panel button')].map((b) => JSON.stringify(b.textContent?.slice(0, 30))).join('\n')));

console.log('\n=== panel 结构 (前 3 层) ===');
console.log(await page.evaluate(() => {
  const p = document.getElementById('dimension-panel');
  const walk = (el, d) => {
    if (d > 3) return [];
    const out = [`${'  '.repeat(d)}<${el.tagName.toLowerCase()} class="${el.className}" role="${el.getAttribute('role') || ''}">`];
    for (const c of [...el.children].slice(0, 4)) out.push(...walk(c, d + 1));
    return out;
  };
  return walk(p, 0).join('\n');
}));

await browser.close();
