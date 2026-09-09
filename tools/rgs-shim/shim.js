// RGS SmartSocket Shim v0.2.0 — PoC expansion (per 9/9 13:45 JST Ulysses 拍板 A)
// 重构: cmdRegistry 模式 (每个 worker 扩自己域时, 只需加表项 + 写 handler)
// 新增: cmd 10200 (地图进入) + cmd 10400 (heartbeat → RGS HealthCheck) + cmd 11001 (role list → RGS player ListPlayers)
// 其它 cmd: 透传 stub
'use strict';

const net = require('node:net');
const http = require('node:http');

const SHIM_PORT = Number(process.env.SHIM_PORT || 9001);
const RGS_PROXY = process.env.RGS_PROXY || 'http://127.0.0.1:8084';
const SHIM_VERSION = '0.2.0';

// =============================================================================
// 1) Big-endian read/write helpers
// =============================================================================
function beU8(b, o) { return b[o]; }
function beI16(b, o) { const v = (b[o]<<8)|b[o+1]; return v & 0x8000 ? v - 0x10000 : v; }
function beU16(b, o) { return (b[o]<<8)|b[o+1]; }
function beU32(b, o) { return ((b[o]<<24)|(b[o+1]<<16)|(b[o+2]<<8)|b[o+3]) >>> 0; }
function beStr(b, o) { const len = beU32(b, o); return { v: b.slice(o+4, o+4+len).toString('utf8'), n: 4+len }; }

const wU8 = v => { const b = Buffer.alloc(1); b[0] = v & 0xff; return b; };
const wI16 = v => { const b = Buffer.alloc(2); const n = v & 0xffff; b[0] = n>>8; b[1] = n & 0xff; return b; };
const wU16 = v => { const b = Buffer.alloc(2); b[0] = (v>>8) & 0xff; b[1] = v & 0xff; return b; };
const wU32 = v => { const b = Buffer.alloc(4); b[0]=(v>>>24)&0xff; b[1]=(v>>>16)&0xff; b[2]=(v>>>8)&0xff; b[3]=v&0xff; return b; };
const wStr = s => { const sb = Buffer.from(s, 'utf8'); return Buffer.concat([wU32(sb.length), sb]); };

// =============================================================================
// 2) Frame parser/builder
// =============================================================================
function parseFrame(buf) {
  if (buf.length < 6) return null;
  const len = beU32(buf, 0);
  const cmd = beU16(buf, 4);
  const payloadLen = len - 2;
  if (buf.length < 6 + payloadLen) return null;
  return { cmd, payload: buf.slice(6, 6 + payloadLen) };
}
function buildFrame(cmd, payload) {
  return Buffer.concat([wU32(2 + payload.length), wU16(cmd), payload]);
}

// =============================================================================
// 3) RGS gRPC call helper
// =============================================================================
function rgsCall(domain, rpc, body) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify(body || {});
    const u = new URL(`${RGS_PROXY}/${domain}/${rpc}`);
    const req = http.request({
      method: 'POST', hostname: u.hostname, port: u.port || 80, path: u.pathname,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(data) },
    }, (res) => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        try { resolve(JSON.parse(Buffer.concat(chunks).toString('utf8'))); }
        catch (e) { reject(e); }
      });
    });
    req.on('error', reject);
    req.setTimeout(5000, () => { req.destroy(); reject(new Error('TIMEOUT')); });
    req.write(data); req.end();
  });
}

// =============================================================================
// 4) Cmd handlers (per 9/9 13:45 JST v0.2 扩展)
// =============================================================================
// 每个 handler signature: async (payload, ctx) => { cmd, payload } (响应 cmd + payload)
// ctx: { socket, direction, raw }

// ---- Login group (10101-10103, proto_101) ----
async function handleRegisterCli(payload) {
  let o = 0;
  const sex = beU8(payload, o); o += 1;
  const { v: name, n: n1 } = beStr(payload, o); o += n1;
  const career = beI16(payload, o); o += 2;
  const { v: playform, n: n2 } = beStr(payload, o);
  console.log(`[shim] 10101 register: sex=${sex} name=${name} career=${career} playform=${playform}`);

  // 调 RGS player.GetPlayer 拿真数据
  const player = await rgsCall('player', 'GetPlayer', { id: '11111111-1111-1111-1111-111111111111' });
  const ok = player.ok;
  const displayName = player.response?.display_name || 'MavisHero';
  const uuid = player.response?.id?.id || '11111111-1111-1111-1111-111111111111';
  const rid = parseInt(uuid.slice(0, 8), 16) >>> 0;

  // srv: code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32
  return { cmd: 10101, payload: Buffer.concat([
    wU8(ok ? 0 : 1),
    wStr(ok ? 'OK (via RGS player.GetPlayer)' : 'RGS 不可达: ' + (player.error || 'unknown')),
    wU32(rid), wStr('rgs-uat-1'), wStr(displayName), wU32(Math.floor(Date.now()/1000))
  ])};
}

async function handleEnterServerCli(payload) {
  let o = 0;
  const rid = beU32(payload, o); o += 4;
  const { v: srvId, n } = beStr(payload, o);
  console.log(`[shim] 10102/10103 enter_server: rid=0x${rid.toString(16)} srv_id=${srvId}`);
  // srv: code:u8, msg:str, timestamp:u32, world_lev:u16
  return { cmd: 10102, payload: Buffer.concat([
    wU8(0), wStr('OK (RGS server ready)'),
    wU32(Math.floor(Date.now()/1000)), wU16(52)
  ])};
}

// ---- Map group (10200, proto_102) ----
async function handleMapEnterCli(payload) {
  let o = 0;
  const battleId = beU32(payload, o); o += 4;
  const id = beU32(payload, o); o += 4;
  const code = beI16(payload, o);
  console.log(`[shim] 10200 map_enter: battle_id=${battleId} id=${id} code=${code}`);
  // srv: code:u8, msg:str
  return { cmd: 10200, payload: Buffer.concat([wU8(0), wStr('OK (RGS map)')])};
}

// ---- Heartbeat (10400, 自定义) ----
async function handleHeartbeatCli(payload) {
  const t0 = Date.now();
  // 并发调 5 域 HealthCheck
  const results = await Promise.allSettled([
    rgsCall('player', 'HealthCheck'),
    rgsCall('economy', 'HealthCheck'),
    rgsCall('match', 'HealthCheck'),
    rgsCall('social', 'HealthCheck'),
    rgsCall('admin', 'HealthCheck'),
  ]);
  const okCount = results.filter(r => r.status === 'fulfilled' && r.value.ok).length;
  const dt = Date.now() - t0;
  console.log(`[shim] 10400 heartbeat: ${okCount}/5 域 OK in ${dt}ms`);
  // srv: code:u8, msg:str, ok_count:u8, total:u8
  return { cmd: 10400, payload: Buffer.concat([
    wU8(0), wStr(`OK ${okCount}/5 RGS 域 in ${dt}ms`),
    wU8(okCount), wU8(5)
  ])};
}

// ---- Role list (11001, 调 RGS player ListPlayers — 假设有, 没则降级) ----
async function handleRoleListCli(payload) {
  console.log(`[shim] 11001 role_list: querying RGS player`);
  // 先试 ListPlayers (可能 RGS 没实装, 降级用 GetPlayer)
  let players = await rgsCall('player', 'ListPlayers', { limit: 5 });
  if (!players.ok) {
    // 降级: 拿单个 player (MavisHero)
    const single = await rgsCall('player', 'GetPlayer', { id: '11111111-1111-1111-1111-111111111111' });
    players = single.ok ? { ok: true, response: { players: [single.response] } } : { ok: false };
  }
  const list = players.response?.players || (players.response ? [players.response] : []);
  console.log(`[shim] 11001 role_list: ${list.length} players from RGS`);
  // srv: code:u8, msg:str, count:u8, [name:str, level:u8 (fake from uuid hash), ...]
  const items = list.slice(0, 5).map(p => {
    const dn = p.display_name || '?';
    // 模拟 level from uuid hash
    const level = ((p.id?.id || '').split('').reduce((a, c) => a + c.charCodeAt(0), 0) % 100) + 1;
    return Buffer.concat([wStr(dn), wU8(level)]);
  });
  return { cmd: 11001, payload: Buffer.concat([
    wU8(0), wStr(`OK (RGS) ${list.length} players`),
    wU8(list.length), ...items
  ])};
}

// =============================================================================
// 5) Cmd registry (worker 加新 cmd 只需 +表项 + 写 handler)
// =============================================================================
const CMD_REGISTRY = {
  10101: handleRegisterCli,      // proto_101: register
  10102: handleEnterServerCli,   // proto_101: enter_server
  10103: handleEnterServerCli,   // proto_101: enter_server alias
  10200: handleMapEnterCli,      // proto_102: map_enter (战斗地图)
  10400: handleHeartbeatCli,     // 自定义: heartbeat → RGS 5 域 HealthCheck
  11001: handleRoleListCli,      // 自定义: role list → RGS player ListPlayers
  // TODO worker 扩: 20000-29999 (战斗), 30000-39999 (聊天/好友), 40000-49999 (任务/工会/排行)
};

async function dispatch(cmd, payload) {
  const handler = CMD_REGISTRY[cmd];
  if (!handler) {
    console.log(`[shim] ${cmd}: stub (no handler registered)`);
    return { cmd, payload: Buffer.alloc(0) };
  }
  return await handler(payload);
}

// =============================================================================
// 6) TCP server
// =============================================================================
const server = net.createServer((socket) => {
  const remote = `${socket.remoteAddress}:${socket.remotePort}`;
  console.log(`[shim] + client ${remote}`);
  let buf = Buffer.alloc(0);
  let frameCount = 0;

  socket.on('data', (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    while (true) {
      if (buf.length < 6) break;
      const len = beU32(buf, 0);
      const totalLen = 4 + len;
      if (buf.length < totalLen) break;
      const f = parseFrame(buf);
      buf = buf.slice(totalLen);
      if (!f) break;
      frameCount++;
      console.log(`[shim] ${remote} frame #${frameCount}: cmd=${f.cmd} payload_len=${f.payload.length}`);
      dispatch(f.cmd, f.payload).then(resp => {
        const out = buildFrame(resp.cmd, resp.payload);
        socket.write(out);
        console.log(`[shim] ${remote} → cmd=${resp.cmd} payload_len=${resp.payload.length}`);
      }).catch(err => {
        console.error(`[shim] ${remote} dispatch error: ${err.message}`);
        socket.write(buildFrame(f.cmd, Buffer.alloc(0)));
      });
    }
  });
  socket.on('close', () => console.log(`[shim] - client ${remote} (${frameCount} frames)`));
  socket.on('error', err => console.error(`[shim] ! ${remote} error: ${err.message}`));
});

server.listen(SHIM_PORT, '0.0.0.0', () => {
  console.log(`[shim] RGS SmartSocket shim v${SHIM_VERSION} listening on 0.0.0.0:${SHIM_PORT}`);
  console.log(`[shim] RGS proxy: ${RGS_PROXY}`);
  console.log(`[shim] Registered cmds: ${Object.keys(CMD_REGISTRY).join(', ')}`);
  console.log(`[shim] Total: ${Object.keys(CMD_REGISTRY).length} cmds (514 unique in zsyz_server, ${514 - Object.keys(CMD_REGISTRY).length} TODO)`);
});
