// rgs-batch-console server.js (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-1, 2026-09-02 01:45 JST Mavis 接手代签)
//
// 零依赖 Node 22 原生 http server (per AGENTS.md v0.4 §7 母规范)
// 监听 127.0.0.1:8789 (区别 rgs-web 8788)
// 提供 /api/v1/health + /api/v1/version 端点 (per DETAILED-DESIGN v0.1 §1.1)
//
// Usage:
//   node server.js
//   curl http://127.0.0.1:8789/api/v1/health  # → 200 OK
//   curl http://127.0.0.1:8789/api/v1/version # → 200 OK

const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const HOST = '127.0.0.1';
const PORT = 8789;
const PUBLIC_DIR = path.join(__dirname, 'public');
const VERSION = '0.1.0';
const START_TIME = Date.now();

// Token 估算 (per BATCH-PLAN v0.2 §6 + RGS-OLU-REPORT-token-OLU-2026-09-02 v0.2 §1.2)
function estimateTokens(text) {
  if (!text) return 0;
  return Math.ceil((text.length || 0) / 4);
}

// 请求锁 (per 8/27 19:06 JST lockfile 派生约束)
const LOCK_FILE = path.join(__dirname, '.lock');
function acquireLock(reqId) {
  if (fs.existsSync(LOCK_FILE)) {
    return { ok: false, holder: fs.readFileSync(LOCK_FILE, 'utf8') };
  }
  fs.writeFileSync(LOCK_FILE, reqId);
  return { ok: true };
}
function releaseLock() {
  if (fs.existsSync(LOCK_FILE)) fs.unlinkSync(LOCK_FILE);
}

const routes = {
  'GET /api/v1/health': (req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      status: 'ok', service: 'rgs-batch-console', version: VERSION,
      uptime_ms: Date.now() - START_TIME, ts: new Date().toISOString()
    }));
  },
  'GET /api/v1/version': (req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      console: VERSION,
      batch_plan: 'RGS-BATCH-PLAN-2026-09-01_v0.2',
      detaill: 'RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1',
      backend_target: 'rgs-batch-backend v0.1.0'
    }));
  },
  'GET /api/v1/token-estimate': (req, res, url) => {
    const text = url.searchParams.get('text') || '';
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ text_length: text.length, estimated_tokens: estimateTokens(text) }));
  },
  'GET /': (req, res) => {
    const indexPath = path.join(PUBLIC_DIR, 'index.html');
    if (fs.existsSync(indexPath)) {
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      fs.createReadStream(indexPath).pipe(res);
    } else {
      res.writeHead(200, { 'Content-Type': 'text/plain' });
      res.end('rgs-batch-console v' + VERSION + '\nGET /api/v1/health for status\n');
    }
  }
};

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://${HOST}:${PORT}`);
  const key = `${req.method} ${url.pathname}`;
  const handler = routes[key];
  if (handler) {
    try { handler(req, res, url); }
    catch (e) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: e.message }));
    }
  } else {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'not found', path: url.pathname }));
  }
});

process.on('SIGINT', () => {
  console.log('[rgs-batch-console] SIGINT, shutting down...');
  releaseLock();
  server.close(() => process.exit(0));
});

server.listen(PORT, HOST, () => {
  console.log(`[rgs-batch-console v${VERSION}] listening on http://${HOST}:${PORT}`);
});
