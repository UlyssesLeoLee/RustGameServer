// tools/h5_e2e/mock_server.mjs
//
// Stand-in for the Rust network-gateway WebSocket endpoint while ULYS-2.1 / 2.2 /
// 2.3 are still `todo`. Speaks the exact [游戏A] binary protocol that
// [游戏A]_client_h5 (Cocos Creator H5) expects on `ws://host:<port>/websocket`.
//
// Handlers:
//   cmd=1199 (heartbeat, empty payload) → reply [cmd=1199][u32 time]
//   cmd=1110 (login stub)               → reply [cmd=1110][code:u8, msg:str,
//                                                       roles:array<{name:str}>,
//                                                       least_career:u8]
//   default                             → reply with same cmd, empty payload
//                                         ([游戏A]_server `unknown_command` analog)
//
// Run:
//   node tools/h5_e2e/mock_server.mjs
// or:
//   PORT=8000 node tools/h5_e2e/mock_server.mjs
//
// Default port is 18000, NOT 8000, because on the test host port 8000 is held by
// Microsoft Manager.exe (PID 6572) and binding fails with EACCES. Once
// ULYS-2.2 lands in network-gateway, point everything at its chosen port.
//
// Env:
//   PORT      = WS port       (default 18000; set 8000 to match [游戏A]_server's
//                              web_conn.erl 8000 once that port is free)
//   LOG_FRAMES=1              (default) — log every frame; set to 0 to silence
//
// Per issue ULYS-6 (任务 D), tracking issue ULYS-2 (01a092ae-2faa-799b-b07e-ae6a9b80c166).
// 2026-09-12 D-Boy.

import { WebSocketServer } from 'ws';
import {
  encodeFrame,
  decodeFrames,
  buildHeartbeatFrame,
  parseHeartbeatReply,
  buildLoginReply,
  CMD_HEARTBEAT,
  CMD_LOGIN,
} from './[游戏A]_protocol.js';

const PORT = Number(process.env.PORT || 18000);
const LOG_FRAMES = process.env.LOG_FRAMES !== '0';
const HOST = process.env.HOST || '0.0.0.0';

function hex(b) {
  const hex = Buffer.from(b).toString('hex');
  if (hex.length === 0) return '(empty)';
  return hex.match(/.{1,2}/g).join(' ');
}

function log(...args) {
  if (LOG_FRAMES) console.log('[mock]', ...args);
}

const wss = new WebSocketServer({ port: PORT, host: HOST, path: '/websocket' });

wss.on('connection', (ws, req) => {
  const remote = `${req.socket.remoteAddress}:${req.socket.remotePort}`;
  log(`+ connect ${remote} (path=${req.url})`);
  let buf = Buffer.alloc(0);
  let frameCount = 0;

  ws.on('message', (data, isBinary) => {
    if (!isBinary) {
      log(`  ${remote} non-binary msg (ignored):`, data.toString());
      return;
    }
    buf = Buffer.concat([buf, Buffer.from(data)]);
    const { frames, remaining } = decodeFrames(buf);
    buf = remaining;

    for (const f of frames) {
      frameCount++;
      log(`  ${remote} frame #${frameCount}  cmd=${f.cmd}  payload_len=${f.payload.length}  hex=${hex(f.payload)}`);

      let reply = null;
      try {
        if (f.cmd === CMD_HEARTBEAT) {
          // empty payload from client → reply with timestamp
          const now = Math.floor(Date.now() / 1000) >>> 0;
          const ts = Buffer.alloc(4);
          ts.writeUInt32BE(now, 0);
          reply = encodeFrame(CMD_HEARTBEAT, ts);
          log(`  ${remote} → heartbeat reply time=${now} hex=${hex(ts)}`);
        } else if (f.cmd === CMD_LOGIN) {
          // login stub: always succeed
          reply = buildLoginReply({
            code: 0,
            msg: 'OK (mock_server; switch to RGS network-gateway after ULYS-2.1/2.2/2.3)',
            roles: ['mock-alice', 'mock-bob'],
            least_career: 1,
          });
          log(`  ${remote} → login reply (stub)`);
        } else {
          // unknown: same cmd, empty payload ([游戏A]_server `unknown_command` analog)
          reply = encodeFrame(f.cmd, Buffer.alloc(0));
          log(`  ${remote} → unknown cmd=${f.cmd} reply (empty)`);
        }
      } catch (e) {
        log(`  ${remote} handler error:`, e.message);
        reply = encodeFrame(f.cmd, Buffer.alloc(0));
      }

      if (reply) {
        try {
          ws.send(reply);
        } catch (e) {
          log(`  ${remote} send error:`, e.message);
        }
      }
    }
  });

  ws.on('close', () => log(`- close  ${remote} (${frameCount} frames)`));
  ws.on('error', (e) => log(`! error  ${remote}: ${e.message}`));
});

wss.on('listening', () => {
  console.log(`[mock] [游戏A] mock server listening on ws://${HOST}:${PORT}/websocket`);
  console.log(`[mock] handlers: cmd=${CMD_HEARTBEAT} heartbeat (→ u32 timestamp), cmd=${CMD_LOGIN} login (→ stub OK)`);
  console.log(`[mock] all other cmds: echo same cmd, empty payload`);
  console.log(`[mock] ready for E2E (D3 puppeteer) and D4 raw-hex heartbeat test`);
});

wss.on('error', (e) => {
  console.error('[mock] server error:', e.message);
  process.exit(1);
});

// graceful shutdown
process.on('SIGINT', () => {
  console.log('[mock] SIGINT, closing…');
  wss.close(() => process.exit(0));
});