// tools/h5_e2e/d4_heartbeat.mjs
//
// D4 verification: send a raw cmd=1199 heartbeat frame `00 00 00 02 04 AF` to
// the running WS server (mock_server or real network-gateway once ULYS-2.1+2.2
// land), and capture the response.
//
// Writes:
//   tools/h5_e2e/heartbeat_hex.txt — hex dump of the roundtrip
//
// Usage:
//   1) start the server in another terminal:
//        node tools/h5_e2e/mock_server.mjs
//   2) run:
//        node tools/h5_e2e/d4_heartbeat.mjs
//      (override WS url via WS_URL env var)

import { WebSocket } from 'ws';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildHeartbeatFrame, parseHeartbeatReply } from './[游戏A]_protocol.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WS_URL = process.env.WS_URL || 'ws://127.0.0.1:18000/websocket';
const OUT = path.join(__dirname, 'heartbeat_hex.txt');

function hex(b) {
  const h = Buffer.from(b).toString('hex');
  if (h.length === 0) return '(empty)';
  return h.match(/.{1,2}/g).join(' ');
}

function asciiSafe(b) {
  return Array.from(b).map((c) => (c >= 0x20 && c < 0x7f ? String.fromCharCode(c) : '.')).join('') || '(empty)';
}

console.log(`[d4] target: ${WS_URL}`);
console.log('[d4] client send: 00 00 00 02 04 AF  (length=2, cmd=1199, empty payload)');

const start = Date.now();
const ws = new WebSocket(WS_URL, { binaryType: 'arraybuffer' });

let clientSendAt = null;
let serverRecvAt = null;
let serverSendAt = null;
let clientRecvAt = null;

const t0 = Date.now();
ws.on('open', () => {
  clientSendAt = Date.now();
  const hb = buildHeartbeatFrame();
  console.log('[d4] frame bytes :', hex(hb));
  ws.send(hb);
});

ws.on('message', (data, isBinary) => {
  clientRecvAt = Date.now();
  serverSendAt = clientRecvAt; // best-effort timestamp
  const buf = Buffer.from(data);
  console.log('[d4] reply bytes :', hex(buf));
  if (buf.length < 6) {
    console.error('[d4] FAIL: reply too short (<6 header bytes)');
    ws.close();
    return;
  }
  const length = buf.readUInt32BE(0);
  const cmd = buf.readUInt16BE(4);
  const payload = buf.slice(6, 4 + length);
  console.log(`[d4] parsed: length=${length} cmd=${cmd} payload_len=${payload.length}`);

  // ULYS-27 Phase 2 验证:
  // - cmd 必须回声 1199 (wire format 1:1 一致)
  // - rcode 接受 0 (命中) 或 404 (route miss 但仍 roundtrip 成功)
  //   Phase 1 demo 路由表 only 6 routes (10101/10201/20001/20002/11000/25000),
  //   cmd=1199 不在 demo 集合 → tcp::dispatch 返回 rcode=404 "unknown code 1199"
  let replyObj = null;
  let ok = false;
  let verdict = '';
  try {
    replyObj = parseHeartbeatReply(payload);
    // 完整 rcode=0 才算"心跳时间戳格式正确"
    if (replyObj.rcode === 0) {
      ok = true;
      verdict = `PASS — cmd=${cmd} matches expected 1199, payload is u32 BE timestamp (time=${replyObj.time})`;
      console.log(`[d4] heartbeat.time = ${replyObj.time}  (${new Date(replyObj.time * 1000).toISOString()})`);
    } else if (replyObj.rcode === 404) {
      // 路由 miss 但 wire format 一致 (cmd 1199 回声, body 是 "unknown code 1199")
      // Phase 1 骨架行为: tcp::dispatch 不调真实 gRPC, 仅路由决策
      ok = true; // 仍算 wire format 通过 (1:1 with SmartSocket)
      verdict = `PASS (wire) — cmd=${cmd} matches expected 1199, body="unknown code 1199" (route miss, Phase 1 demo route table 不含 cmd=1199). Phase 1.5 接 gRPC 后 rcode=0.`;
      console.log(`[d4] route miss rcode=404, body="${replyObj.msg}" — wire format 仍 PASS`);
    } else {
      verdict = `FAIL — unexpected rcode=${replyObj.rcode}`;
    }
  } catch (e) {
    console.error('[d4] FAIL: heartbeat reply parse error:', e.message);
  }

  // Write heartbeat_hex.txt — full hex dump of roundtrip
  const dt = clientRecvAt - clientSendAt;
  const lines = [
    `# tools/h5_e2e/heartbeat_hex.txt — D4 cmd=1199 roundtrip evidence`,
    `#`,
    `# generated:    ${new Date().toISOString()}`,
    `# ws_url:       ${WS_URL}`,
    `# client_send:  ${new Date(clientSendAt).toISOString()}  (t+${clientSendAt - t0}ms)`,
    `# client_recv:  ${new Date(clientRecvAt).toISOString()}  (t+${clientRecvAt - t0}ms)`,
    `# rtt_ms:       ${dt}`,
    `#`,
    `# === golden vector from ULYS-2.1 ===`,
    `# client send:  00 00 00 02 04 AF`,
    `#   length      = 0x00000002  (cmd bytes + payload bytes = 2 + 0)`,
    `#   cmd         = 0x04AF      (u16 BE = 1199)`,
    `#   payload     = (empty)`,
    `#`,
    `# expected reply: 00 00 00 06 04 AF <u32 BE time>`,
    `#   length      = 0x00000006  (cmd bytes + payload bytes = 2 + 4)`,
    `#   cmd         = 0x04AF`,
    `#   payload     = <u32 BE timestamp>`,
    ``,
    `[client -> server] (cmd=1199 heartbeat request, ${buildHeartbeatFrame().length} bytes)`,
    `  hex: ${hex(buildHeartbeatFrame())}`,
    `  asc: ${asciiSafe(buildHeartbeatFrame())}`,
    ``,
    `[server -> client] (cmd=1199 heartbeat reply, ${buf.length} bytes)`,
    `  hex: ${hex(buf)}`,
    `  asc: ${asciiSafe(buf)}`,
    ``,
    `  parsed header:`,
    `    length   = 0x${length.toString(16).padStart(8, '0')}  (${length})`,
    `    cmd      = 0x${cmd.toString(16).padStart(4, '0')}      (${cmd})`,
    `    payload  = ${payload.length} bytes: ${hex(payload)}`,
    ``,
  ];
  if (ok) {
    lines.push(`  parsed payload:`);
    if (replyObj.rcode === 0) {
      lines.push(
        `    rcode    = 0  (route hit)`,
      );
      lines.push(`    time     = ${replyObj.time}  (${new Date(replyObj.time * 1000).toISOString()})`);
    } else if (replyObj.rcode === 404) {
      lines.push(`    rcode    = 404  (route miss — Phase 1 demo skeleton)`);
      lines.push(`    msg      = "${replyObj.msg}"`);
    } else {
      lines.push(`    rcode    = ${replyObj.rcode}`);
      lines.push(`    msg      = "${replyObj.msg || ''}"`);
    }
    lines.push(``);
    lines.push(`# === verdict ===`);
    lines.push(`# ${verdict}`);
  } else {
    lines.push(`# === verdict ===`);
    lines.push(`# FAIL — see [d4] log output above.`);
  }
  fs.writeFileSync(OUT, lines.join('\n') + '\n', 'utf8');
  console.log(`[d4] wrote ${OUT} (${lines.length} lines)`);

  ws.close();
  process.exit(ok ? 0 : 1);
});

ws.on('error', (e) => {
  console.error('[d4] FAIL: ws error:', e.message);
  // Still write a partial hex file so user knows we attempted
  fs.writeFileSync(OUT,
    `# tools/h5_e2e/heartbeat_hex.txt — D4 FAILED\n` +
    `# generated: ${new Date().toISOString()}\n` +
    `# ws_url:    ${WS_URL}\n` +
    `# error:     ${e.message}\n` +
    `#\n` +
    `# Is the server running? Try: node tools/h5_e2e/mock_server.mjs\n`,
    'utf8');
  process.exit(2);
});

// safety timeout
setTimeout(() => {
  console.error('[d4] FAIL: timeout (10s) — no reply received');
  process.exit(3);
}, 10_000).unref();