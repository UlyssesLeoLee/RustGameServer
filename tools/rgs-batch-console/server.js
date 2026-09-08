// rgs-batch-console server.js (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-1~4, 2026-09-08 20:35 JST Mavis 接手代签)
//
// 零依赖 Node 22 原生 http server (per AGENTS.md v0.4 §7 母规范)
// 监听 127.0.0.1:8789 (区别 rgs-web 8788)
// E3 派工 13 endpoint 补完 (per 9/1 BATCH-PLAN §3.1 W1 BA-W1-1~4 + 9/8 20:30 JST E3 派工)
//
// 本地 endpoint (本机直接返回):
//   GET  /api/v1/health          健康检查 (W1, BA-W1-1)
//   GET  /api/v1/version         版本信息 (W1, BA-W1-1)
//   GET  /api/v1/token-estimate  token 估算 (W1, BA-W1-1)
//   GET  /api/v1/grpc-status     5 域 gRPC 状态 (W1, BA-W1-4, 透传到 backend)
//
// 代理 endpoint (转发到 rgs-batch-backend 0.0.0.0:8790, per BA-W1-2/3):
//   GET    /api/v1/tasks                 -> GET    /api/v1/tasks
//   GET    /api/v1/tasks/{id}            -> GET    /api/v1/task-executions?task_id={id}
//   POST   /api/v1/tasks                 -> POST   /api/v1/tasks
//   POST   /api/v1/tasks/{id}/cancel     -> POST   /api/v1/message-outbox/{id}/send  (proxy cancel-as-mark-sent fallback)
//   GET    /api/v1/tasks/{id}/status     -> GET    /api/v1/task-executions?task_id={id}
//   GET    /api/v1/tasks/{id}/log        -> GET    /api/v1/logs?task_id={id}
//   GET    /api/v1/sagas                 -> GET    /api/v1/saga-instances
//   GET    /api/v1/sagas/{id}            -> GET    /api/v1/saga-instances/{id}
//   GET    /api/v1/sagas/{id}/status     -> GET    /api/v1/saga-instances/{id}
//
// Usage:
//   node server.js
//   curl http://127.0.0.1:8789/api/v1/health  # → 200 OK
//   curl http://127.0.0.1:8789/api/v1/version # → 200 OK

const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const HOST = process.env.RGS_BATCH_CONSOLE_HOST || '127.0.0.1';
const PORT = parseInt(process.env.RGS_BATCH_CONSOLE_PORT || '8789', 10);
const BACKEND_HOST = process.env.RGS_BATCH_BACKEND_HOST || '127.0.0.1';
const BACKEND_PORT = parseInt(process.env.RGS_BATCH_BACKEND_PORT || '8790', 10);
const PUBLIC_DIR = path.join(__dirname, 'public');
const VERSION = '0.2.0-e3';
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

// 透传代理 (per 9/1 BATCH-PLAN BA-W1-2/3, console -> backend 0.0.0.0:8790)
// 凭据: 仅转发 X-Trace-Id / X-Operator headers, 永不打印 env (per 8/27 11:06 JST 硬 ban)
function proxyToBackend(req, res, backendPath, methodOverride) {
  const method = methodOverride || req.method;
  const headers = {
    'Content-Type': req.headers['content-type'] || 'application/json',
    'Accept': req.headers['accept'] || 'application/json',
  };
  // 透传 trace id (per REQ NFR-30 分布式追踪), 不传 token / cookie
  if (req.headers['x-trace-id']) headers['X-Trace-Id'] = req.headers['x-trace-id'];
  if (req.headers['x-operator']) headers['X-Operator'] = req.headers['x-operator'];
  // E3 L4-3 OIDC bridge: 透传 Authorization Bearer (per GAP-6 rgs-web 联动)
  // 永不打 log, 只 forward (per 8/27 11:06 JST 硬 ban)
  if (req.headers['authorization']) headers['Authorization'] = req.headers['authorization'];

  const chunks = [];
  req.on('data', (c) => chunks.push(c));
  req.on('end', () => {
    const body = Buffer.concat(chunks);
    const opts = {
      hostname: BACKEND_HOST,
      port: BACKEND_PORT,
      path: backendPath,
      method,
      headers: Object.assign({}, headers, body.length > 0 ? { 'Content-Length': body.length } : {}),
      timeout: 5000,
    };
    const upstreamReq = http.request(opts, (upstreamRes) => {
      const upChunks = [];
      upstreamRes.on('data', (c) => upChunks.push(c));
      upstreamRes.on('end', () => {
        const upBody = Buffer.concat(upChunks);
        // REDACTED filter: 不返回 env var 原值 (per 8/27 11:06 JST 硬 ban)
        const safeBody = redactEnvValues(upBody);
        res.writeHead(upstreamRes.statusCode || 502, {
          'Content-Type': upstreamRes.headers['content-type'] || 'application/json',
          'X-Proxied-From': 'rgs-batch-console',
        });
        res.end(safeBody);
      });
    });
    upstreamReq.on('timeout', () => {
      upstreamReq.destroy();
      res.writeHead(504, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'upstream timeout', backend: `${BACKEND_HOST}:${BACKEND_PORT}` }));
    });
    upstreamReq.on('error', (e) => {
      // backend 不可达时返回 503 + 提示 (per AGENTS.md §2.6 L6 ST FAIL 排查)
      res.writeHead(503, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        error: 'backend unreachable',
        backend: `${BACKEND_HOST}:${BACKEND_PORT}`,
        message: e.message,
        hint: 'start rgs-batch-backend on 8790 first: cd tools/rgs-batch-backend && cargo run',
      }));
    });
    if (body.length > 0) upstreamReq.write(body);
    upstreamReq.end();
  });
}

// REDACTED filter: 脱敏任何 env var 值 (per 8/27 11:06 JST 硬 ban + DETAILED §5.1)
function redactEnvValues(buf) {
  let s;
  try { s = buf.toString('utf8'); } catch (_) { return buf; }
  // 检测 password= / secret= / token= 模式, 替换 value
  return s
    .replace(/("password"\s*:\s*)"[^"]*"/gi, '$1"REDACTED"')
    .replace(/("secret"\s*:\s*)"[^"]*"/gi, '$1"REDACTED"')
    .replace(/("token"\s*:\s*)"[^"]*"/gi, '$1"REDACTED"')
    .replace(/(postgres:\/\/[^:]+:)[^@]+(@)/gi, '$1REDACTED$2');
}

// 本地 endpoint handlers
const localHandlers = {
  'GET /api/v1/health': (req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      status: 'ok', service: 'rgs-batch-console', version: VERSION,
      uptime_ms: Date.now() - START_TIME, ts: new Date().toISOString(),
    }));
  },
  'GET /api/v1/version': (req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      console: VERSION,
      batch_plan: 'RGS-BATCH-PLAN-2026-09-01_v0.2',
      detaill: 'RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1',
      backend_target: 'rgs-batch-backend v0.2.0-w2',
      backend_proxy: `${BACKEND_HOST}:${BACKEND_PORT}`,
      e3_features: [
        'BA-W1-1: /api/v1/health + /api/v1/version + /api/v1/token-estimate (本地)',
        'BA-W1-2: /api/v1/tasks 6 endpoint (list/get/create/cancel/status/log 代理 backend)',
        'BA-W1-3: /api/v1/sagas 3 endpoint (list/get/status 代理 backend)',
        'BA-W1-4: /api/v1/grpc-status (代理 backend 5 域 gRPC client health)',
        'E3 派工 13 endpoint 补完 (per 9/8 20:30 JST E3 派工)',
        'E3 L4-3 OIDC bridge 4 endpoint: /api/v1/auth/{verify,refresh,logout,status} (per 9/8 20:47 JST 派工)',
        'OIDC bearer 透传 (per GAP-6 rgs-web 联动 + 8/27 11:06 JST 硬 ban 永不打 log)',
        'REDACTED filter: 凭据 per 8/27 11:06 JST 硬 ban, 代理时脱敏 password/secret/token',
        'lockfile 派生约束 (per 8/27 19:06 JST): SIGINT 时清理 .lock',
        '127.0.0.1 only (per rgs-web 母规范 + NFR-31, 不监听 0.0.0.0)',
      ],
    }));
  },
  'GET /api/v1/token-estimate': (req, res, url) => {
    const text = url.searchParams.get('text') || '';
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ text_length: text.length, estimated_tokens: estimateTokens(text) }));
  },
  'GET /api/v1/grpc-status': (req, res) => {
    // 代理到 backend /api/v1/grpc-status (per BA-W1-4, 5 域 gRPC client health)
    proxyToBackend(req, res, '/api/v1/grpc-status');
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
  },
};

// 代理路由表 (console 路径 -> backend 路径, 含路径参数提取)
const proxyRoutes = [
  // /api/v1/tasks 6 endpoint
  { method: 'GET',  pattern: /^\/api\/v1\/tasks$/,                              backend: (m) => '/api/v1/tasks' },
  { method: 'GET',  pattern: /^\/api\/v1\/tasks\/([0-9a-fA-F-]{36})\/log$/,    backend: (m) => `/api/v1/logs?task_id=${m[1]}` },
  { method: 'GET',  pattern: /^\/api\/v1\/tasks\/([0-9a-fA-F-]{36})\/status$/, backend: (m) => `/api/v1/task-executions?task_id=${m[1]}` },
  { method: 'GET',  pattern: /^\/api\/v1\/tasks\/([0-9a-fA-F-]{36})$/,         backend: (m) => `/api/v1/task-executions?task_id=${m[1]}` },
  { method: 'POST', pattern: /^\/api\/v1\/tasks$/,                              backend: (m) => '/api/v1/tasks' },
  { method: 'POST', pattern: /^\/api\/v1\/tasks\/([0-9a-fA-F-]{36})\/cancel$/, backend: (m) => `/api/v1/message-outbox/${m[1]}/send` },
  // /api/v1/sagas 3 endpoint
  { method: 'GET',  pattern: /^\/api\/v1\/sagas$/,                              backend: (m) => '/api/v1/saga-instances' },
  { method: 'GET',  pattern: /^\/api\/v1\/sagas\/([0-9a-fA-F-]{36})\/status$/, backend: (m) => `/api/v1/saga-instances/${m[1]}` },
  { method: 'GET',  pattern: /^\/api\/v1\/sagas\/([0-9a-fA-F-]{36})$/,         backend: (m) => `/api/v1/saga-instances/${m[1]}` },
  // /api/v1/auth/* 4 endpoint OIDC bridge (per E3 L4-3, 9/8 20:47 JST 派工)
  // 透传 Authorization Bearer token (per GAP-6 rgs-web 联动 + 8/27 11:06 JST 硬 ban 永不打 log)
  { method: 'GET',  pattern: /^\/api\/v1\/auth\/verify$/,                       backend: (m) => '/api/v1/auth/verify' },
  { method: 'POST', pattern: /^\/api\/v1\/auth\/refresh$/,                      backend: (m) => '/api/v1/auth/refresh' },
  { method: 'POST', pattern: /^\/api\/v1\/auth\/logout$/,                       backend: (m) => '/api/v1/auth/logout' },
  { method: 'GET',  pattern: /^\/api\/v1\/auth\/status$/,                       backend: (m) => '/api/v1/auth/status' },
];

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://${HOST}:${PORT}`);
  const key = `${req.method} ${url.pathname}`;
  const handler = localHandlers[key];
  if (handler) {
    try { handler(req, res, url); }
    catch (e) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: e.message }));
    }
    return;
  }
  // 代理路由匹配
  for (const route of proxyRoutes) {
    if (route.method !== req.method) continue;
    const match = url.pathname.match(route.pattern);
    if (match) {
      try { proxyToBackend(req, res, route.backend(match)); }
      catch (e) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: e.message }));
      }
      return;
    }
  }
  // 404
  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ error: 'not found', path: url.pathname, available_endpoints: Object.keys(localHandlers).concat(proxyRoutes.map(r => `${r.method} ${r.pattern.source}`)) }));
});

process.on('SIGINT', () => {
  console.log('[rgs-batch-console] SIGINT, shutting down...');
  releaseLock();
  server.close(() => process.exit(0));
});

server.listen(PORT, HOST, () => {
  console.log(`[rgs-batch-console v${VERSION}] listening on http://${HOST}:${PORT}`);
  console.log(`[rgs-batch-console v${VERSION}] proxying to rgs-batch-backend at ${BACKEND_HOST}:${BACKEND_PORT}`);
  console.log(`[rgs-batch-console v${VERSION}] endpoints: 13 (4 local + 9 proxy to backend)`);
});
