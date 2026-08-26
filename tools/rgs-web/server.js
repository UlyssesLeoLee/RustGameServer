// RGS Admin Web v0.3 — 真实接入 5 域 gRPC + cluster-ops via kubectl port-forward
// per RGS-WEB-PLAN-2026-08-26 v0.1 + v0.3 真实接入
// 0 npm 依赖:仅用 Node 内置 http / http2 / child_process / url / fs / path
//
// 工作流:
//   1) WSL2 内 `k3s kubectl port-forward` 暴露 5 域 gRPC 端口到 127.0.0.1:1505x
//   2) 本服务用 http2.connect('http://127.0.0.1:15051') 调 gRPC
//   3) 手写 protobuf 编码(只有 common.v1 + 5 域 GetXxx/HealthCheck 的字段)
//
// 端口映射(WSL 端,127.0.0.1):
//   player-service    gRPC 15051,  /metrics 19464
//   economy-service   gRPC 15052,  /metrics 19465
//   match-service     gRPC 15053,  /metrics 19466
//   social-service    gRPC 15054,  /metrics 19467
//   admin-service     gRPC 15055,  /metrics 19468
//   cluster-ops       gRPC 15056,  /metrics 19469

const http  = require('http');
const http2 = require('http2');
const url  = require('url');
const fs   = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PORT = process.env.RGS_WEB_PORT || 8788;

// ===== 5 域 gRPC endpoint 配置 =====
const SERVICES = {
  'player-service':  { grpc: 15051, metrics: 19464, package: 'player.v1',         service: 'PlayerService'    },
  'economy-service': { grpc: 15052, metrics: 19465, package: 'economy.v1',        service: 'EconomyService'   },
  'match-service':   { grpc: 15053, metrics: 19466, package: 'match.v1',          service: 'MatchService'     },
  'social-service':  { grpc: 15054, metrics: 19467, package: 'social.v1',         service: 'SocialService'    },
  'admin-service':   { grpc: 15055, metrics: 19468, package: 'admin.v1',          service: 'AdminService'     },
  'cluster-ops':     { grpc: 15056, metrics: 19469, package: 'cluster_ops.v1',    service: 'ClusterOpsService'},
};

// ===== Protobuf wire encoder =====
// 仅编码 5 域 proto 实际用到的字段:
const PB = {
  // varint
  encodeVarint(n) {
    n = Number(n);
    if (n < 0) n += (1n << 64n); // shouldn't happen for our types
    const out = [];
    while (n > 0x7f) { out.push((n & 0x7f) | 0x80); n = Math.floor(n / 128); }
    out.push(n & 0x7f);
    return Buffer.from(out);
  },
  // field tag (field_number << 3) | wire_type
  encodeTag(fieldNum, wireType) { return this.encodeVarint((fieldNum << 3) | wireType); },
  // string / bytes (wire type 2)
  encodeString(fieldNum, s) {
    const buf = Buffer.from(s || '', 'utf8');
    return Buffer.concat([this.encodeTag(fieldNum, 2), this.encodeVarint(buf.length), buf]);
  },
  // int32 / enum (wire type 0)
  encodeInt32(fieldNum, n) {
    return Buffer.concat([this.encodeTag(fieldNum, 0), this.encodeVarint(n | 0)]);
  },
  // int64 (wire type 0, varint)
  encodeInt64(fieldNum, n) {
    return Buffer.concat([this.encodeTag(fieldNum, 0), this.encodeVarint(BigInt(n))]);
  },
  // nested message (wire type 2)
  encodeMessage(fieldNum, msgBuf) {
    return Buffer.concat([this.encodeTag(fieldNum, 2), this.encodeVarint(msgBuf.length), msgBuf]);
  },
  // EntityId = { string id = 1; }
  encodeEntityId(id) { return this.encodeString(1, id); },
  // HealthCheckRequest = { string service = 1; }
  encodeHealthCheckRequest(service) { return this.encodeString(1, service); },
  // Player / Account / Match / Guild / AdminOp / Node 共用布局:
  //   { EntityId id=1; Status status=2; Timestamp created_at=3; string display_name=4; }
  // 字段全填默认值以保证可解码;只放 id + display_name(其他 proto 字段未读到)
  encodeEntity(en) {
    const parts = [
      this.encodeMessage(1, this.encodeEntityId(en.id || '')),
      this.encodeInt32(2, en.status | 0),
      this.encodeMessage(3, Buffer.concat([this.encodeInt64(1, en.created_at?.seconds || 0), this.encodeInt32(2, en.created_at?.nanos || 0)])),
      this.encodeString(4, en.display_name || ''),
    ];
    return Buffer.concat(parts);
  },
};

// ===== Protobuf wire decoder (通用) =====
// 返回 JS 对象 + raw 字节视图
function decodeProto(buf) {
  let pos = 0;
  const out = {};
  const readVarint = () => {
    let n = 0n, shift = 0n;
    while (pos < buf.length) {
      const b = BigInt(buf[pos++]);
      n |= (b & 0x7fn) << shift;
      if ((b & 0x80n) === 0n) return n;
      shift += 7n;
      if (shift > 63n) throw new Error('varint too long');
    }
    throw new Error('truncated varint');
  };
  const readBytes = (n) => { const s = buf.slice(pos, pos + n); pos += n; return s; };
  while (pos < buf.length) {
    const tag = Number(readVarint());
    const fieldNum = tag >>> 3;
    const wireType = tag & 7;
    if (wireType === 0) {
      out[fieldNum] = Number(readVarint());
    } else if (wireType === 2) {
      const len = Number(readVarint());
      const data = readBytes(len);
      // 判断:字段值是 message(嵌套)还是 scalar
      if (len > 0 && isLikelyMessage(data)) {
        try { out[fieldNum] = decodeProto(data); } catch { out[fieldNum] = data; }
      } else {
        // 文本字段优先:protobuf string = UTF-8 bytes;若 UTF-8 解码有效且非空则视为 string
        let s = null;
        try { s = data.toString('utf8'); } catch {}
        const isPrintable = s && /[\x20-\x7e\u00a0-\uffff]/.test(s) && !/[\x00-\x08\x0e-\x1f]/.test(s);
        if (s && isPrintable && data.length < 4096) {
          out[fieldNum] = s;
        } else {
          out[fieldNum] = data;
        }
      }
    } else if (wireType === 5) {
      out[fieldNum] = Number(buf.readBigUInt32BE(pos)); pos += 4;
    } else if (wireType === 1) {
      out[fieldNum] = buf.slice(pos, pos + 8); pos += 8;
    } else {
      throw new Error(`unsupported wire type ${wireType} at field ${fieldNum}`);
    }
  }
  return out;
}

function isLikelyMessage(buf) {
  if (buf.length < 2) return false;
  // 第 0 字节:低 3 位 = wire type, 高 5 位 = field_num
  // 0=varint, 2=length-delimited, 5=32bit
  // 简单启发:若以 length-delimited 子消息开头则递归
  const wt = buf[0] & 7;
  if (wt === 2) {
    // 计算 varint 长度,看是否"后续还有结构"
    let p = 1, len = 0, shift = 0;
    while (p < buf.length && p < 6) {
      const b = buf[p++];
      len |= (b & 0x7f) << shift;
      if ((b & 0x80) === 0) break;
      shift += 7;
    }
    // 合理 message 长度
    return (len > 0 && len < buf.length);
  }
  return false;
}

// ===== gRPC client via http2 (with mTLS) =====
// 调用: grpcCall('player-service', 'GetPlayer', pbReqBytes) → { httpStatus, msgLen, decoded, raw }
// gRPC wire format: 1 byte compression flag (0) + 4 bytes BE length + protobuf message
function grpcWrapBody(pbBody) {
  // compression flag = 0 (uncompressed), length = pbBody.length
  const header = Buffer.alloc(5);
  header[0] = 0; // no compression
  header.writeUInt32BE(pbBody.length, 1);
  return Buffer.concat([header, pbBody]);
}

const TLS_CA   = fs.readFileSync(path.join(__dirname, 'rgs-ca.pem'));
const TLS_CERT = fs.readFileSync(path.join(__dirname, 'rgs-client.crt.pem'));
const TLS_KEY  = fs.readFileSync(path.join(__dirname, 'rgs-client.key.pem'));

// SNI 映射 — 每个 server cert 的 CN/SAN (per main.rs 实际证书)
const SERVICE_SNI = {
  'player-service':  'player.service',
  'economy-service': 'economy.service',
  'match-service':   'match.service',
  'social-service':  'social.service',
  'admin-service':   'admin.service',
  'cluster-ops':     'cluster-ops.service',
};

function grpcCall(svcName, method, reqBody) {
  return new Promise((resolve, reject) => {
    const cfg = SERVICES[svcName];
    if (!cfg) return reject(new Error(`unknown service: ${svcName}`));
    const grpcPath = `/${cfg.package}.${cfg.service}/${method}`;
    const client = http2.connect(`https://127.0.0.1:${cfg.grpc}`, {
      ca: TLS_CA,
      cert: TLS_CERT,
      key: TLS_KEY,
      servername: SERVICE_SNI[svcName],
      ALPNProtocols: ['h2'],
      rejectUnauthorized: true,
    });
    let settled = false;
    const finish = (err, val) => {
      if (settled) return; settled = true;
      try { client.close(); } catch {}
      if (err) return reject(err);
      resolve(val);
    };
    client.on('error', (e) => finish(new Error(`http2 mTLS connect ${svcName}:${cfg.grpc} failed: ${e.message}`)));
    const req = client.request({
      ':method': 'POST',
      ':path': grpcPath,
      ':scheme': 'https',
      'content-type': 'application/grpc',
      'te': 'trailers',
    });
    req.on('response', (headers) => {
      const status = headers[':status'];
      // 5-byte gRPC header: [compression(1)] [length(4 BE)]
      const data = [];
      let totalLen = 0;
      let need = 5;
      let firstChunk = true;
      req.on('data', (chunk) => {
        data.push(chunk);
        totalLen += chunk.length;
      });
      req.on('end', () => {
        const all = Buffer.concat(data);
        if (all.length < 5) {
          return finish(null, { httpStatus: status, raw: all, error: 'short grpc response' });
        }
        const compressed = all[0];
        const msgLen = all.readUInt32BE(1);
        const body = all.slice(5, 5 + msgLen);
        let decoded = null;
        try { decoded = decodeProto(body); } catch (e) { decoded = { decodeError: e.message, raw: body.toString('base64') }; }
        finish(null, { httpStatus: status, msgLen, compressed, decoded, raw: body });
      });
      req.on('error', (e) => finish(e));
    });
    req.on('error', (e) => finish(e));
    // gRPC 5-byte header (compression flag 0 + length) + protobuf message
    req.end(grpcWrapBody(reqBody));
  });
}

// ===== HTTP/1.1 server: 6 API endpoints =====
function send(res, code, obj) {
  res.writeHead(code, { 'Content-Type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(obj));
}

function notFound(res, path) { send(res, 404, { error: 'not_found', path }); }

const handlers = {
  '/api/health': () => ({
    status: 'ok',
    pid: process.pid,
    rgs_web_version: '0.3.0-real-grpc',
    time: new Date().toISOString(),
    transport: 'http2 (built-in) + kubectl port-forward',
    pages: ['dashboard','servers','players','stream','config','hotupdate','operations','docs','worktrees','reports'],
  }),

  // 5 域并发 HealthCheck
  '/api/health/all': async () => {
    const t0 = Date.now();
    const targets = [
      ['player-service',  'HealthCheck'],
      ['economy-service', 'HealthCheck'],
      ['match-service',   'HealthCheck'],
      ['social-service',  'HealthCheck'],
      ['admin-service',   'HealthCheck'],
      ['cluster-ops',     'HealthCheck'],
    ];
    const results = await Promise.all(targets.map(async ([svc, m]) => {
      const t = Date.now();
      try {
        const reqBytes = PB.encodeHealthCheckRequest(svc);
        const r = await grpcCall(svc, m, reqBytes);
        const statusEnum = r.decoded?.[1]; // field 1 = Status
        const message = r.decoded?.[2];    // field 2 = message string
        return { service: svc, ok: true, status: enumToName(statusEnum), message, httpStatus: r.httpStatus, latencyMs: Date.now() - t };
      } catch (e) {
        return { service: svc, ok: false, error: e.message, latencyMs: Date.now() - t };
      }
    }));
    return { totalMs: Date.now() - t0, services: Object.fromEntries(results.map(r => [r.service, r])), summary: results };
  },

  // 5 域 endpoints 列表
  '/api/services/status': () => {
    return {
      transport: 'http://127.0.0.1:1505x (gRPC) + http://127.0.0.1:1946x (Prometheus /metrics) via kubectl port-forward',
      port_forward_source: '/tmp/rgs-pf/<svc>-<lport>.log (per WSL k3s kubectl port-forward)',
      services: Object.entries(SERVICES).map(([name, c]) => ({
        name,
        grpc: `http://127.0.0.1:${c.grpc}`,
        metrics: `http://127.0.0.1:${c.metrics}/metrics`,
        package: c.package,
        service: c.service,
        methods: ['HealthCheck', name === 'player-service' ? 'GetPlayer' : name === 'economy-service' ? 'GetAccount' : name === 'match-service' ? 'GetMatch' : name === 'social-service' ? 'GetGuild' : name === 'admin-service' ? 'GetAdminOp' : 'GetNode'],
      })),
    };
  },

  // 单个 GetPlayer (player-service)
  // 路由: GET /api/player/:id
  // 在 main router 中通过 query 解析
  // cluster-ops GetNode / 其他域 GetXxx 也用同一模式

  // 真 psql 查询(WSL 内 psql)。psql 不可用时降级 mock
  '/api/sql/query': async (params) => {
    const db = params.db || 'player_db';
    const sql = params.sql || 'SELECT 1';
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(db)) return { error: 'invalid db name' };
    // 安全:只允许 SELECT / SHOW
    if (!/^\s*(select|show)\b/i.test(sql)) {
      return { error: 'only SELECT/SHOW allowed (per ARC-008 + RGS-IMPL-100 saga 不可逆)' };
    }
    // 尝试 WSL psql
    try {
      const escaped = sql.replace(/"/g, '\\"');
      const cmd = `PGPASSWORD=ulysses_local psql -h 10.42.0.7 -U postgres -d ${db} -t -c "${escaped}" 2>&1`;
      const out = execSync(`wsl -e bash -c '${cmd.replace(/'/g, "'\\''")}'`, { encoding: 'utf8', timeout: 5000 });
      // psql -t 输出每行一条
      const rows = out.split('\n').map(s => s.trim()).filter(Boolean);
      return { source: 'wsl-psql', db, sql, rows, raw: out };
    } catch (e) {
      // psql 不可用:降级 mock + 注释
      return {
        source: 'mock-fallback',
        reason: `psql 不可用: ${e.message?.split('\n')[0] || e.message}`,
        note: 'WSL 内未安装 psql client;k3s postgres pod IP 10.42.0.7 需 client tool 才能 exec SQL。' +
              '可降级方案: kubectl exec -it postgres -- psql 或 kubectl port-forward 5432 + Node pg client',
        db, sql,
        mock_rows: [
          '| mock | 5 domain per ARC-008 |',
          '| player_db    | 6 tables |',
          '| economy_db   | 8 tables |',
          '| match_db     | 5 tables |',
          '| social_db    | 7 tables |',
          '| admin_db     | 4 tables |',
        ],
      };
    }
  },

  // cluster-ops PFAU phase — proto 实际只有 HealthCheck + GetNode,无 PFAU method
  // 降级:用 HealthCheck + GetNode 当前状态代为报告
  '/api/pfau/phase': async (params) => {
    const nodeId = params.node_id || 'ulyssespc';
    const t0 = Date.now();
    try {
      const hcReq = PB.encodeHealthCheckRequest('cluster-ops');
      const hc = await grpcCall('cluster-ops', 'HealthCheck', hcReq);
      const nodeReq = PB.encodeEntityId(nodeId);
      const node = await grpcCall('cluster-ops', 'GetNode', nodeReq);
      return {
        source: 'cluster-ops gRPC (HealthCheck + GetNode)',
        note: 'proto 实际无 PFAU phase method(per crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto);降级用 HealthCheck + GetNode 代为报告',
        pfau_phase: 'NOT_IN_PROTO',
        reason: 'PFAU 阶段需要 cluster-ops binary 真实部署 + 5 域联动激活;当前 6 域 gRPC 只读 GetXxx/HealthCheck',
        cluster_health: { status: enumToName(hc.decoded?.[1]), message: hc.decoded?.[2] || '' },
        node: {
          id: node.decoded?.[1]?.id || '(no id field)',
          status: enumToName(node.decoded?.[2]),
          display_name: node.decoded?.[4],
          raw: node.decoded,
        },
        latencyMs: Date.now() - t0,
      };
    } catch (e) {
      return { error: e.message, latencyMs: Date.now() - t0 };
    }
  },

  // 5 域 /metrics 端点(HTTP, 非 gRPC)
  // 路由: GET /api/metrics/<svc>
  '/api/metrics': async (params) => {
    const svc = params.service;
    const cfg = SERVICES[svc];
    if (!cfg) return { error: 'unknown service', service: svc, valid: Object.keys(SERVICES) };
    return new Promise((resolve) => {
      const t = Date.now();
      const req = http.request({
        hostname: '127.0.0.1', port: cfg.metrics, path: '/metrics', method: 'GET',
        headers: { 'User-Agent': 'rgs-web/0.3' },
        timeout: 3000,
      }, (res) => {
        let body = '';
        res.on('data', c => body += c);
        res.on('end', () => {
          // 简单解析:前 5KB + 关键 metric 计数
          const lines = body.split('\n');
          const samples = lines.filter(l => /^[a-z_]+\{/.test(l) || /^[a-z_]+ \d/.test(l)).slice(0, 30);
          resolve({
            service: svc, httpStatus: res.statusCode, latencyMs: Date.now() - t,
            sizeBytes: body.length, totalLines: lines.length, samples,
          });
        });
      });
      req.on('error', (e) => resolve({ service: svc, error: e.message, latencyMs: Date.now() - t, fallback: 'metrics port-forward may need to be running' }));
      req.on('timeout', () => { req.destroy(); resolve({ service: svc, error: 'timeout', latencyMs: Date.now() - t }); });
      req.end();
    });
  },

  // (保留 v0.2 的 IMPL-PLAN / Worktrees / Docs Health 端点,降级为辅助信息)
  '/api/impl-plan': () => {
    const dir = path.join(__dirname, '..', '..', 'docs', '12-工作流');
    try {
      const files = fs.readdirSync(dir).filter(f => f.startsWith('RGS-IMPL-PLAN-') && f.endsWith('.md'));
      return { plans: files.map(f => {
        const c = fs.readFileSync(path.join(dir, f), 'utf8');
        return { file: f, status: (c.match(/\| 状态\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown', size: c.length };
      }) };
    } catch (e) { return { error: e.message }; }
  },

  '/api/worktrees': () => {
    try {
      const out = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const wts = out.split('\n\n').filter(s => s.trim()).map(b => {
        const lines = b.split('\n');
        return {
          path:  lines.find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' '),
          head:  lines.find(l => l.startsWith('HEAD '))?.split(' ')[1]?.substring(0, 7),
          branch:lines.find(l => l.startsWith('branch '))?.split(' ').slice(1).join(' ')?.replace('refs/heads/', ''),
          locked: b.includes('locked'),
        };
      });
      return { worktrees: wts, total: wts.length };
    } catch (e) { return { error: e.message }; }
  },

  '/api/docs-health': () => ({
    fail: 1, warn: 1,
    fail_reason: 'RGS-DEC-NOGO-001 缺决策编号字段',
    warn_reason: '5 ADR 待具名审批',
    last_check: '2026-08-26 04:30 JST 基线',
  }),

  '/api/git-log': () => {
    try {
      const out = execSync('git log --pretty=format:"%h|%ad|%s" --date=short -n 30', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
      const commits = out.split('\n').filter(l => l.trim()).map(l => {
        const [hash, date, ...msg] = l.split('|');
        return { hash, date, message: msg.join('|') };
      });
      return { commits, total: commits.length };
    } catch (e) { return { error: e.message }; }
  },
};

// Status enum
const STATUS_NAMES = { 0: 'UNSPECIFIED', 1: 'OK', 2: 'PENDING', 3: 'FAILED', 4: 'CANCELLED' };
function enumToName(n) { return STATUS_NAMES[n] || `UNKNOWN(${n})`; }

// ===== HTTP server =====
const server = http.createServer(async (req, res) => {
  const t0 = Date.now();
  const u = url.parse(req.url, true);
  const p = u.pathname;
  const q = u.query;

  // 静态
  if (p === '/' || p === '/index.html') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return fs.createReadStream(path.join(__dirname, 'public', 'index.html')).pipe(res);
  }
  if (p.startsWith('/public/') || /\.(css|js|svg|png|ico)$/.test(p)) {
    const fp = path.join(__dirname, p);
    if (fs.existsSync(fp)) {
      const mime = { css:'text/css', js:'application/javascript', svg:'image/svg+xml', png:'image/png', ico:'image/x-icon' }[path.extname(fp).substring(1)] || 'application/octet-stream';
      res.writeHead(200, { 'Content-Type': mime });
      return fs.createReadStream(fp).pipe(res);
    }
  }

  // API
  try {
    // /api/player/:id → player-service GetPlayer
    const playerMatch = p.match(/^\/api\/player\/(.+)$/);
    if (playerMatch) {
      const id = decodeURIComponent(playerMatch[1]);
      const reqBytes = PB.encodeEntityId(id);
      const r = await grpcCall('player-service', 'GetPlayer', reqBytes);
      // 字段 1 = EntityId(id={1: id_str}), 2 = status, 3 = Timestamp, 4 = display_name
      const playerView = {
        id: r.decoded?.[1]?.id || null,
        status: enumToName(r.decoded?.[2]),
        created_at: { seconds: r.decoded?.[3]?.[1], nanos: r.decoded?.[3]?.[2] },
        display_name: r.decoded?.[4] || null,
      };
      return send(res, 200, {
        source: 'player-service gRPC GetPlayer (http2 → kubectl port-forward → 127.0.0.1:15051 → k3s pod:50051)',
        httpStatus: r.httpStatus,
        msgLen: r.msgLen,
        latencyMs: Date.now() - t0,
        player: playerView,
        raw_decoded: r.decoded,
      });
    }
    // /api/guild/:id → social-service GetGuild
    const guildMatch = p.match(/^\/api\/guild\/(.+)$/);
    if (guildMatch) {
      const id = decodeURIComponent(guildMatch[1]);
      const r = await grpcCall('social-service', 'GetGuild', PB.encodeEntityId(id));
      return send(res, 200, { source: 'social-service GetGuild', guild: { id: r.decoded?.[1]?.id, status: enumToName(r.decoded?.[2]), display_name: r.decoded?.[4] }, raw: r.decoded });
    }
    // /api/match/:id → match-service GetMatch
    const matchMatch = p.match(/^\/api\/match\/(.+)$/);
    if (matchMatch) {
      const id = decodeURIComponent(matchMatch[1]);
      const r = await grpcCall('match-service', 'GetMatch', PB.encodeEntityId(id));
      return send(res, 200, { source: 'match-service GetMatch', match: { id: r.decoded?.[1]?.id, status: enumToName(r.decoded?.[2]), display_name: r.decoded?.[4] }, raw: r.decoded });
    }
    // /api/account/:id → economy-service GetAccount
    const accMatch = p.match(/^\/api\/account\/(.+)$/);
    if (accMatch) {
      const id = decodeURIComponent(accMatch[1]);
      const r = await grpcCall('economy-service', 'GetAccount', PB.encodeEntityId(id));
      return send(res, 200, { source: 'economy-service GetAccount', account: { id: r.decoded?.[1]?.id, status: enumToName(r.decoded?.[2]), display_name: r.decoded?.[4] }, raw: r.decoded });
    }
    // /api/adminop/:id → admin-service GetAdminOp
    const admMatch = p.match(/^\/api\/adminop\/(.+)$/);
    if (admMatch) {
      const id = decodeURIComponent(admMatch[1]);
      const r = await grpcCall('admin-service', 'GetAdminOp', PB.encodeEntityId(id));
      return send(res, 200, { source: 'admin-service GetAdminOp', op: { id: r.decoded?.[1]?.id, status: enumToName(r.decoded?.[2]), display_name: r.decoded?.[4] }, raw: r.decoded });
    }
    // /api/node/:id → cluster-ops GetNode
    const nodeMatch = p.match(/^\/api\/node\/(.+)$/);
    if (nodeMatch) {
      const id = decodeURIComponent(nodeMatch[1]);
      const r = await grpcCall('cluster-ops', 'GetNode', PB.encodeEntityId(id));
      return send(res, 200, { source: 'cluster-ops GetNode', node: { id: r.decoded?.[1]?.id, status: enumToName(r.decoded?.[2]), display_name: r.decoded?.[4] }, raw: r.decoded });
    }

    // /api/metrics/:service
    const metricsMatch = p.match(/^\/api\/metrics\/(.+)$/);
    if (metricsMatch) {
      const result = await handlers['/api/metrics']({ service: metricsMatch[1] });
      return send(res, 200, result);
    }

    // 普通 handler
    const handler = handlers[p];
    if (handler) {
      const out = await handler(q);
      return send(res, 200, out);
    }
    return notFound(res, p);
  } catch (e) {
    return send(res, 500, { error: e.message, stack: e.stack?.split('\n').slice(0, 3) });
  }
});

server.listen(PORT, '127.0.0.1', () => {
  console.log(`RGS Admin Web v0.3-real-grpc 启动: http://127.0.0.1:${PORT}`);
  console.log(`  transport: Node http2 → kubectl port-forward → k3s pod gRPC`);
  console.log(`  port mapping:`);
  for (const [n, c] of Object.entries(SERVICES)) {
    console.log(`    ${n.padEnd(18)} gRPC 127.0.0.1:${c.grpc}  metrics 127.0.0.1:${c.metrics}`);
  }
  console.log(`  PID: ${process.pid}`);
});
