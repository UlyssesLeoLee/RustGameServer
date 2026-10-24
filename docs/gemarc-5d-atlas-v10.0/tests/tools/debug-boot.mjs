import { chromium } from '@playwright/test';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const INDEX = pathToFileURL(resolve(process.cwd(), 'index.html')).href;
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

const logs = [];
page.on('pageerror', (e) => logs.push(`[pageerror] ${e.message}\n${e.stack ?? ''}`));
page.on('console', (m) => logs.push(`[console:${m.type()}] ${m.text()}`));

await page.goto(INDEX);
await page.waitForTimeout(2000);

console.log('=== 浏览器日志 ===');
console.log(logs.join('\n') || '(无)');
console.log('\n=== 关键 DOM 状态 ===');
console.log(JSON.stringify(await page.evaluate(() => ({
  status: document.getElementById('status-text')?.textContent,
  revision: document.getElementById('revision-text')?.textContent,
  gateBtn: !!document.getElementById('gate-indicator'),
  dimTabs: [...document.querySelectorAll('.dim-tab')].map((t) => t.dataset.dim),
  currentDim: document.getElementById('app')?.getAttribute('data-dimension'),
  panelHtmlLen: document.getElementById('dimension-panel')?.innerHTML.length,
})), null, 2));

await browser.close();
