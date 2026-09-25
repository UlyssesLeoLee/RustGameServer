// tools/h5_e2e/e2e_login.mjs
//
// E2E test (D3): drive the in-browser SmartSocket clone (smart_socket_test.html)
// with Playwright + cached chromium-1234. Captures:
//   - WS frames sent by the browser (cmd=1199 heartbeat + cmd=1110 login)
//   - WS frames received from the server
//   - Console errors (especially "unpackData_error …" which means the wire
//     format doesn't match SmartSocket 1:1)
//
// Outputs:
//   - tools/h5_e2e/e2e_evidence.json (frames + verdict)
//   - tools/h5_e2e/screenshot.png   (browser screenshot at end of run)
//
// Per issue ULYS-6 (任务 D). Tracking issue ULYS-2.

import { chromium } from 'playwright-core';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const CHROMIUM_PATH = process.env.CHROMIUM_PATH ||
  'C:/Users/leo19/AppData/Local/ms-playwright/chromium-1234/chrome-win64/chrome.exe';

const PAGE_URL = process.env.PAGE_URL || 'http://127.0.0.1:18001/smart_socket_test.html';
const WS_URL = process.env.WS_URL || 'ws://127.0.0.1:18000/websocket';

const EVIDENCE_PATH = path.join(__dirname, 'e2e_evidence.json');
const SCREENSHOT_PATH = path.join(__dirname, 'screenshot.png');

const CMD_HEARTBEAT = 1199;
const CMD_LOGIN = 1110;

async function main() {
  console.log('[e2e] target page:', PAGE_URL);
  console.log('[e2e] target ws  :', WS_URL);
  console.log('[e2e] chromium   :', CHROMIUM_PATH);

  if (!fs.existsSync(CHROMIUM_PATH)) {
    console.error('[e2e] FAIL: chromium not found at', CHROMIUM_PATH);
    console.error('[e2e] hint: set CHROMIUM_PATH env to your chrome.exe');
    process.exit(2);
  }

  const browser = await chromium.launch({
    executablePath: CHROMIUM_PATH,
    headless: true,
    args: [
      '--no-sandbox',
      '--disable-dev-shm-usage',
      // WebGL/swiftshader helps if any rendering happens; harmless here
      '--use-gl=swiftshader',
      '--enable-unsafe-webgpu',
    ],
  });

  const context = await browser.newContext({ viewport: { width: 800, height: 600 } });
  const page = await context.newPage();

  const consoleLogs = [];
  const consoleErrors = [];
  page.on('console', (msg) => {
    consoleLogs.push({ at: Date.now(), type: msg.type(), text: msg.text() });
    if (msg.type() === 'error') consoleErrors.push({ at: Date.now(), text: msg.text() });
  });
  page.on('pageerror', (err) => {
    consoleErrors.push({ at: Date.now(), text: '[pageerror] ' + err.message });
  });

  // Navigate
  await page.addInitScript((wsUrl) => {
    window.GAMEA_WS_URL = wsUrl;
  }, WS_URL);

  await page.goto(PAGE_URL, { waitUntil: 'domcontentloaded' });

  // Wait for the page to set window.__[游戏A]Frames__
  await page.waitForFunction(() => Array.isArray(window.__[游戏A]Frames__));

  // 1) Trigger Connect
  await page.click('#connect-btn');
  console.log('[e2e] clicked Connect');

  // Wait for first heartbeat roundtrip (send 1199 → recv 1199)
  await page.waitForFunction(
    () => {
      const f = window.__[游戏A]Frames__;
      const sent = f.some((x) => x.dir === 'send' && x.cmd === 1199);
      const recv = f.some((x) => x.dir === 'recv' && x.cmd === 1199);
      return sent && recv;
    },
    { timeout: 5000 }
  );
  console.log('[e2e] ✓ heartbeat roundtrip captured');

  // 2) Trigger login
  await page.click('#login-btn');
  console.log('[e2e] clicked Login');

  // Wait for login reply
  await page.waitForFunction(
    () => window.__[游戏A]Frames__.some((x) => x.dir === 'recv' && x.cmd === 1110),
    { timeout: 5000 }
  );
  console.log('[e2e] ✓ login reply captured');

  // Let the page settle (small delay so any async errors surface)
  await page.waitForTimeout(500);

  // Screenshot
  await page.screenshot({ path: SCREENSHOT_PATH, fullPage: true });
  console.log('[e2e] screenshot →', SCREENSHOT_PATH);

  // Snapshot the in-browser frames
  const frames = await page.evaluate(() => window.__[游戏A]Frames__);
  const logBuf = await page.evaluate(() => window.__[游戏A]LogBuf__ || []);
  const errs = consoleErrors.slice();
  const logs = consoleLogs.slice();

  // Verdict
  const sentHb = frames.some((x) => x.dir === 'send' && x.cmd === CMD_HEARTBEAT);
  const recvHb = frames.some((x) => x.dir === 'recv' && x.cmd === CMD_HEARTBEAT);
  const sentLogin = frames.some((x) => x.dir === 'send' && x.cmd === CMD_LOGIN);
  const recvLogin = frames.some((x) => x.dir === 'recv' && x.cmd === CMD_LOGIN);
  const noUnpackErr = !errs.some((e) => /unpackData_error/i.test(e.text));

  const verdict = {
    pass: sentHb && recvHb && sentLogin && recvLogin && noUnpackErr,
    checks: {
      sent_heartbeat_1199: sentHb,
      recv_heartbeat_1199: recvHb,
      sent_login_1110: sentLogin,
      recv_login_1110: recvLogin,
      no_unpackData_error: noUnpackErr,
    },
  };

  const evidence = {
    issue: 'ULYS-6 (任务 D)',
    parent: 'ULYS-2',
    generated_at: new Date().toISOString(),
    page_url: PAGE_URL,
    ws_url: WS_URL,
    chromium_path: CHROMIUM_PATH,
    frames,
    console_errors: errs,
    console_logs: logs.slice(-50),  // tail only
    log_buf: logBuf.slice(-50),
    verdict,
  };
  fs.writeFileSync(EVIDENCE_PATH, JSON.stringify(evidence, null, 2));
  console.log('[e2e] evidence   →', EVIDENCE_PATH);

  // Friendly summary
  console.log('---');
  console.log('[e2e] summary:');
  console.log('  sent 1199 heartbeat :', sentHb);
  console.log('  recv 1199 heartbeat :', recvHb);
  console.log('  sent 1110 login     :', sentLogin);
  console.log('  recv 1110 login     :', recvLogin);
  console.log('  console errors      :', errs.length);
  console.log('  verdict             :', verdict.pass ? 'PASS ✅' : 'FAIL ❌');

  await context.close();
  await browser.close();
  process.exit(verdict.pass ? 0 : 1);
}

main().catch((e) => {
  console.error('[e2e] FAIL:', e.stack || e.message);
  process.exit(2);
});