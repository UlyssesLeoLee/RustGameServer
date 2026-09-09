// RGS SmartSocket Shim (Node.js, 0 third-party deps per rgs-proxy 母规范)
// Listens on TCP 9001 (zsyz SmartSocket default port), parses big-endian binary frames,
// forwards to RGS via rgs-proxy 8084, wraps responses back into zsyz frames.
//
// Per 2026-09-09 13:35 JST Ulysses 拍板:
// 真 zsyz C++ 客户端 (zsyz_client) 0 修改, 把 server URL 从 localhost:9001
// 改到 localhost:9001 (this shim 假装是 zsyz_server Erlang), 透明转发到 RGS gRPC
//
// Frame 格式 (per zsyz_client_core/frameworks/game_core/thirdparty/Libnetwork/GameTcpClient.h):
//   | len:32 (BE) | cmd:16 (BE) | payload... |
//   len = byte_size(cmd) + byte_size(payload) = 2 + payload_len
//
// 字节序: big-endian (大端, network byte order)
//
// Cmd IDs (per zsyz_server/src/proto/proto_101.erl):
//   10101 cli: register (sex:u8, name:str, career:i16, playform:str)
//   10101 srv: register response (code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32)
//   10102 cli: enter server (rid:u32, srv_id:str)
//   10102 srv: enter response (code:u8, msg:str, timestamp:u32, world_lev:u16)
//   10103 cli: same as 10102 (alias)
//   10103 srv: same as 10102

'use strict';

const net = require('node:net');
const http = require('node:http');

const SHIM_PORT = Number(process.env.SHIM_PORT || 9001);
const RGS_PROXY = process.env.RGS_PROXY || 'http://127.0.0.1:8084';
const SHIM_VERSION = '0.1.0';

// === Big-endian read/write helpers ===
function beReadU8(buf, off) { return buf[off]; }
function beReadU16(buf, off) { return (buf[off] << 8) | buf[off+1]; }
function beReadU32(buf, off) { return ((buf[off]<<24) | (buf[off+1]<<16) | (buf[off+2]<<8) | buf[off+3]) >>> 0; }
function beReadI16(buf, off) {
  const v = beReadU16(buf, off);
  return v & 0x8000 ? v - 0x10000 : v;
}
function beReadString(buf, off) {
  // String: u32 len + len bytes (no terminator)
  const len = beReadU32(buf, off);
  return buf.slice(off+4, off+4+len).toString('utf8');
}

function beWriteU8(v) { const b = Buffer.alloc(1); b[0] = v & 0xff; return b; }
function beWriteU16(v) { const b = Buffer.alloc(2); b[0] = (v>>8) & 0xff; b[1] = v & 0xff; return b; }
function beWriteU32(v) { const b = Buffer.alloc(4); b[0]=(v>>>24)&0xff; b[1]=(v>>>16)&0xff; b[2]=(v>>>8)&0xff; b[3]=v&0xff; return b; }
function beWriteI16(v) { return beWriteU16(v & 0xffff); }
function beWriteString(s) {
  const sBuf = Buffer.from(s, 'utf8');
  return Buffer.concat([beWriteU32(sBuf.length), sBuf]);
}

// === Frame helpers ===
function parseFrame(buf) {
  // buf: 6-byte header + payload
  if (buf.length < 6) return null;
  const len = beReadU32(buf, 0);
  const cmd = beReadU16(buf, 4);
  const payloadLen = len - 2; // len = 2 (cmd) + payload
  if (buf.length < 6 + payloadLen) return null;
  return { cmd, payload: buf.slice(6, 6 + payloadLen) };
}

function buildFrame(cmd, payload) {
  const len = 2 + payload.length;
  return Buffer.concat([beWriteU32(len), beWriteU16(cmd), payload]);
}

// === RGS gRPC call via rgs-proxy ===
function rgsCall(domain, rpc, body) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify(body || {});
    const u = new URL(`${RGS_PROXY}/${domain}/${rpc}`);
    const req = http.request({
      method: 'POST',
      hostname: u.hostname,
      port: u.port || 80,
      path: u.pathname,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(data) },
    }, (res) => {
      const chunks = [];
      res.on('data', (c) => chunks.push(c));
      res.on('end', () => {
        try { resolve(JSON.parse(Buffer.concat(chunks).toString('utf8'))); }
        catch (e) { reject(e); }
      });
    });
    req.on('error', reject);
    req.setTimeout(5000, () => { req.destroy(); reject(new Error('TIMEOUT')); });
    req.write(data);
    req.end();
  });
}

// === Cmd dispatch ===
async function dispatch(cmd, payload) {
  console.log(`[shim] cmd=${cmd} payload_len=${payload.length}`);

  // 10101 cli: register (sex:u8, name:str, career:i16, playform:str)
  // 10101 srv: code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32
  if (cmd === 10101) {
    let off = 0;
    const sex = beReadU8(payload, off); off += 1;
    const name = beReadString(payload, off); off += 4 + Buffer.byteLength(name, 'utf8');
    const career = beReadI16(payload, off); off += 2;
    const playform = beReadString(payload, off);
    console.log(`[shim] 10101 register: sex=${sex} name=${name} career=${career} playform=${playform}`);

    // 调 RGS player.GetPlayer 拿玩家真数据 (per 9/9 13:35 拍板)
    const player = await rgsCall('player', 'GetPlayer', { id: '11111111-1111-1111-1111-111111111111' });
    if (!player.ok) {
      console.log(`[shim] RGS player error: ${player.error}`);
      // 返 code=1 (失败) + msg
      const resp = Buffer.concat([
        beWriteU8(1), beWriteString('RGS 不可达: ' + (player.error || 'unknown')),
        beWriteU32(0), beWriteString(''), beWriteString('MavisHero'),
        beWriteU32(Math.floor(Date.now()/1000))
      ]);
      return { cmd: 10101, payload: resp };
    }
    // 成功: code=0, msg='OK', rid=player_uuid_first8, srv_id='rgs-uat-1', name=display_name
    const displayName = player.response?.display_name || 'MavisHero';
    const uuidPrefix = (player.response?.id?.id || '').slice(0, 8) || '00000000';
    const rid = parseInt(uuidPrefix, 16) >>> 0; // 8 hex → uint32
    const resp = Buffer.concat([
      beWriteU8(0), beWriteString('OK (via RGS)'),
      beWriteU32(rid), beWriteString('rgs-uat-1'),
      beWriteString(displayName), beWriteU32(Math.floor(Date.now()/1000))
    ]);
    return { cmd: 10101, payload: resp };
  }

  // 10102 cli: enter server (rid:u32, srv_id:str)
  // 10102 srv: code:u8, msg:str, timestamp:u32, world_lev:u16
  if (cmd === 10102 || cmd === 10103) {
    let off = 0;
    const rid = beReadU32(payload, off); off += 4;
    const srvId = beReadString(payload, off);
    console.log(`[shim] ${cmd} enter_server: rid=${rid} srv_id=${srvId}`);

    // 返 code=0, msg='OK', timestamp=now, world_lev=52
    const resp = Buffer.concat([
      beWriteU8(0), beWriteString('OK (RGS server ready)'),
      beWriteU32(Math.floor(Date.now()/1000)), beWriteU16(52)
    ]);
    return { cmd, payload: resp };
  }

  // 其它 cmd: 返空响应 (RGS stub)
  console.log(`[shim] ${cmd}: stub (not implemented)`);
  return { cmd, payload: Buffer.alloc(0) };
}

// === TCP server ===
const server = net.createServer((socket) => {
  const remote = `${socket.remoteAddress}:${socket.remotePort}`;
  console.log(`[shim] + client ${remote}`);
  let buf = Buffer.alloc(0);
  let cmd_count = 0;

  socket.on('data', (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    // 解析所有完整帧
    while (true) {
      if (buf.length < 6) break;
      const len = beReadU32(buf, 0);
      const totalLen = 4 + len;  // 4-byte len + len bytes (cmd+payload)
      if (buf.length < totalLen) break;
      const frame = parseFrame(buf);
      if (!frame) break;
      buf = buf.slice(totalLen);
      cmd_count++;
      console.log(`[shim] ${remote} frame #${cmd_count}: cmd=${frame.cmd} payload_len=${frame.payload.length}`);

      // 异步 dispatch
      dispatch(frame.cmd, frame.payload).then((resp) => {
        const outFrame = buildFrame(resp.cmd, resp.payload);
        socket.write(outFrame);
        console.log(`[shim] ${remote} → cmd=${resp.cmd} payload_len=${resp.payload.length}`);
      }).catch((err) => {
        console.error(`[shim] dispatch error: ${err.message}`);
        // 返空帧
        socket.write(buildFrame(frame.cmd, Buffer.alloc(0)));
      });
    }
  });

  socket.on('close', () => {
    console.log(`[shim] - client ${remote} (${cmd_count} frames)`);
  });
  socket.on('error', (err) => {
    console.error(`[shim] ! client ${remote} error: ${err.message}`);
  });
});

server.listen(SHIM_PORT, '0.0.0.0', () => {
  console.log(`[shim] RGS SmartSocket shim v${SHIM_VERSION} listening on 0.0.0.0:${SHIM_PORT}`);
  console.log(`[shim] RGS proxy: ${RGS_PROXY}`);
  console.log(`[shim] Connect: zsyz_client → tcp://localhost:${SHIM_PORT} (was 9001, same)`);
  console.log(`[shim] Supported: cmd 10101 (register), 10102/10103 (enter_server)`);
});
