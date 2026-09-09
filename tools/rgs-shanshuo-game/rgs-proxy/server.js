// RGS 5+3 域 gRPC → HTTP JSON proxy
// Browser fetch → HTTP 8084 → gRPC 50051 (player-service) / 50052 (economy) / etc.
// Mavis 接手, 2026-09-08, per 9/8 21:00 JST 用户拍板真实 RGS 联动路径

const path = require('path');
const http = require('http');
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// 加载 5 域 proto
const PROTO_DIR = 'D:/RustGameServer/crates';
const protos = {
  player:   { path: path.join(PROTO_DIR, 'player-service', 'proto', 'player', 'v1', 'player.proto'),
              addr: '127.0.0.1:50061', service: 'player.v1.PlayerService',
              include: [path.join(PROTO_DIR, 'shared-platform', 'proto')] },
  economy:  { path: path.join(PROTO_DIR, 'economy-service', 'proto', 'economy', 'v1', 'economy.proto'),
              addr: '127.0.0.1:50062', service: 'economy.v1.EconomyService',
              include: [path.join(PROTO_DIR, 'shared-platform', 'proto')] },
  match:    { path: path.join(PROTO_DIR, 'match-service', 'proto', 'match', 'v1', 'match.proto'),
              addr: '127.0.0.1:50063', service: 'match.v1.MatchService',
              include: [path.join(PROTO_DIR, 'shared-platform', 'proto')] },
  social:   { path: path.join(PROTO_DIR, 'social-service', 'proto', 'social', 'v1', 'social.proto'),
              addr: '127.0.0.1:50064', service: 'social.v1.SocialService',
              include: [path.join(PROTO_DIR, 'shared-platform', 'proto')] },
  admin:    { path: path.join(PROTO_DIR, 'admin-service', 'proto', 'admin', 'v1', 'admin.proto'),
              addr: '127.0.0.1:50065', service: 'admin.v1.AdminService',
              include: [path.join(PROTO_DIR, 'shared-platform', 'proto')] },
};

const clients = {};
for (const [name, cfg] of Object.entries(protos)) {
  try {
    const def = protoLoader.loadSync(cfg.path, { keepCase: true, longs: String, enums: String, defaults: true, oneofs: true, includeDirs: cfg.include });
    const proto = grpc.loadPackageDefinition(def);
    // service 形如 'player.v1.PlayerService', proto 结构: proto.player.v1.PlayerService
    const parts = cfg.service.split('.');
    let ServiceCtor = proto;
    for (const p of parts) ServiceCtor = ServiceCtor?.[p];
    console.log(`[PROXY] ${name} resolved:`, ServiceCtor ? 'OK' : 'UNDEFINED');
    if (ServiceCtor) {
      clients[name] = new ServiceCtor(cfg.addr, grpc.credentials.createInsecure());
      console.log(`[PROXY] ${name} client ready → ${cfg.addr}`);
    }
  } catch (e) {
    console.log(`[PROXY] ${name} proto load skip: ${e.message.slice(0, 80)}`);
  }
}

// HTTP server
const server = http.createServer(async (req, res) => {
  // CORS
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  if (req.method === 'OPTIONS') { res.writeHead(200); res.end(); return; }

  const url = new URL(req.url, 'http://localhost:8084');
  const path = url.pathname;
  console.log(`[PROXY] ${req.method} ${path}`);

  // 路由: /health, /<domain>/<rpc>
  if (path === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', clients: Object.keys(clients), uptime: process.uptime() }));
    return;
  }

  // 解析 /<domain>/<rpc>
  const m = path.match(/^\/([^/]+)\/([^/]+)$/);
  if (!m) {
    res.writeHead(400, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'invalid path' }));
    return;
  }
  const [, domain, rpc] = m;
  const client = clients[domain];
  if (!client) {
    res.writeHead(503, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: `domain ${domain} not loaded`, available: Object.keys(clients) }));
    return;
  }
  if (typeof client[rpc] !== 'function') {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: `rpc ${rpc} not found on ${domain}` }));
    return;
  }

  // 读 body
  let body = {};
  if (req.method === 'POST') {
    const chunks = [];
    req.on('data', c => chunks.push(c));
    await new Promise(r => req.on('end', r));
    try { body = JSON.parse(Buffer.concat(chunks).toString() || '{}'); } catch {}
  }

  // gRPC 调用 (5s timeout)
  const deadline = new Date(Date.now() + 5000);
  client[rpc](body, { deadline }, (err, response) => {
    if (err) {
      res.writeHead(502, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: err.message, code: err.code, domain, rpc }));
      return;
    }
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, domain, rpc, response, latency_ms: Date.now() }));
  });
});

server.listen(8084, '127.0.0.1', () => {
  console.log('[PROXY] rgs-proxy listening on 127.0.0.1:8084');
  console.log(`[PROXY] clients loaded: ${Object.keys(clients).join(', ') || '(none, RGS binary not running)'}`);
});
