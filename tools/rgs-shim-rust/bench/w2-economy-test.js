// w2 economy test: 验证 handlers_w2 对 20000-29999 cmd 字节级正确
// per 2026-09-09 19:39 JST Mavis 派工 w2 worker

const net = require('net');

const HOST = '127.0.0.1';
const PORT = 9002;

// ============================================================================
// 字节级编码 (per zsyz_server protocol:pack)
// ============================================================================

function packString(s) {
  const buf = Buffer.from(s, 'utf8');
  const out = Buffer.alloc(4 + buf.length);
  out.writeUInt32BE(buf.length, 0);
  buf.copy(out, 4);
  return out;
}

function makeFrame(cmd, payload = null) {
  const p = payload || Buffer.alloc(0);
  const len = 2 + p.length;
  const buf = Buffer.alloc(6 + p.length);
  buf.writeUInt32BE(len, 0);
  buf.writeUInt16BE(cmd, 4);
  if (p.length > 0) p.copy(buf, 6);
  return buf;
}

function sendFrame(cmd, payload, timeoutMs = 5000) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection(PORT, HOST);
    let buf = Buffer.alloc(0);
    let timer;
    sock.on('connect', () => sock.write(makeFrame(cmd, payload)));
    sock.on('data', (chunk) => {
      buf = Buffer.concat([buf, chunk]);
      if (buf.length >= 6) {
        const totalLen = 4 + buf.readUInt32BE(0);
        if (buf.length >= totalLen) {
          clearTimeout(timer);
          sock.end();
          const payloadLen = buf.readUInt32BE(0) - 2;
          resolve({
            len: buf.readUInt32BE(0),
            cmd: buf.readUInt16BE(4),
            payload: buf.slice(6, 6 + payloadLen),
          });
        }
      }
    });
    sock.on('error', (e) => { clearTimeout(timer); reject(e); });
    timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, timeoutMs);
  });
}

// ============================================================================
// 测试用例 (w2 economy 域 110+ cmd 抽样验证)
// ============================================================================

const tests = [];
function test(name, fn) { tests.push({ name, fn }); }

// T1: 20001 (cli={}, srv={code:u8, msg:str}) - 验证简单 code+msg 模式
test('20001 combat_start (code+msg)', async () => {
  const r = await sendFrame(20001, Buffer.alloc(0));
  if (r.cmd !== 20001) throw new Error(`cmd mismatch: ${r.cmd}`);
  // 期望: code:u8 = 0, msg:str = "OK"
  const code = r.payload.readUInt8(0);
  const msgLen = r.payload.readUInt32BE(1);
  const msg = r.payload.toString('utf8', 5, 5 + msgLen);
  if (code !== 0) throw new Error(`code != 0: ${code}`);
  if (msg !== 'OK') throw new Error(`msg != OK: ${msg}`);
});

// T2: 20030 (cli={}, srv={is_in_combat:u8}) - 验证单字节响应
test('20030 combat_in_combat (single u8)', async () => {
  const r = await sendFrame(20030, Buffer.alloc(0));
  if (r.cmd !== 20030) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 1) throw new Error(`payload len != 1: ${r.payload.length}`);
  if (r.payload[0] !== 0) throw new Error(`is_in_combat != 0: ${r.payload[0]}`);
});

// T3: 20026 (cli={}, srv={drama_id:u32})
test('20026 combat_drama (drama_id u32)', async () => {
  const r = await sendFrame(20026, Buffer.alloc(0));
  if (r.cmd !== 20026) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});

// T4: 20019 (cli={}, srv={}) - 空响应
test('20019 empty response', async () => {
  const r = await sendFrame(20019, Buffer.alloc(0));
  if (r.cmd !== 20019) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 0) throw new Error(`payload not empty: ${r.payload.length}`);
});

// T5: 20223 (cli={}, srv={flag:u8})
test('20223 arena flag (single u8)', async () => {
  const r = await sendFrame(20223, Buffer.alloc(0));
  if (r.cmd !== 20223) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 1) throw new Error(`payload len != 1: ${r.payload.length}`);
});

// T6: 21000 (cli={}, srv={end_time:u32, first_gift:u32})
test('21000 welfare_end (2 u32)', async () => {
  const r = await sendFrame(21000, Buffer.alloc(0));
  if (r.cmd !== 21000) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 8) throw new Error(`payload len != 8: ${r.payload.length}`);
});

// T7: 21100 (cli={}, srv={status_list:u16 list of u8})
test('21100 checkin_status (u16 count + [])', async () => {
  const r = await sendFrame(21100, Buffer.alloc(0));
  if (r.cmd !== 21100) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
  const count = r.payload.readUInt16BE(0);
  if (count !== 0) throw new Error(`count != 0: ${count}`);
});

// T8: 20033 (cli={}, srv={result, def_name, def_guild_name, def_lev, def_face_id, replay_id})
test('20033 combat_replay_view (6 fields)', async () => {
  const r = await sendFrame(20033, Buffer.alloc(0));
  if (r.cmd !== 20033) throw new Error(`cmd mismatch: ${r.cmd}`);
  // 期望: result:u8 + 2 str (空=4 bytes each) + 3 u32 = 1+4+4+4+4+4 = 21 bytes
  if (r.payload.length < 1) throw new Error(`payload too short: ${r.payload.length}`);
});

// T9: 20063 (cli={}, srv={type_list count + [u16]})
test('20063 combat_type_list (u16 count)', async () => {
  const r = await sendFrame(20063, Buffer.alloc(0));
  if (r.cmd !== 20063) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
});

// T10: 20014 cli={target_id:u32, target_srv_id:str} → srv={code, msg}
test('20014 with cli payload', async () => {
  const cliPayload = Buffer.concat([
    Buffer.from([0x11, 0x22, 0x33, 0x44]),  // target_id
    packString('rgs-uat-1'),                  // target_srv_id
  ]);
  const r = await sendFrame(20014, cliPayload);
  if (r.cmd !== 20014) throw new Error(`cmd mismatch: ${r.cmd}`);
  const code = r.payload.readUInt8(0);
  if (code !== 0) throw new Error(`code != 0: ${code}`);
});

// T11: 性能测试 - 50 并发 × 1 cmd, 验证 rps >= 100 (单 cmd 轻量)
test('perf: 20001 50并发 rps', async () => {
  const N = 50;
  const start = Date.now();
  const promises = [];
  for (let i = 0; i < N; i++) {
    promises.push(sendFrame(20001, Buffer.alloc(0)));
  }
  await Promise.all(promises);
  const dt = (Date.now() - start) / 1000;
  const rps = N / dt;
  console.log(`    20001: ${N} reqs in ${dt.toFixed(2)}s = ${rps.toFixed(0)} rps`);
  if (rps < 100) throw new Error(`rps too low: ${rps.toFixed(0)} < 100`);
});

// T12: 大量 cmd 抽样 (跨域)
test('cmd 抽样 20002/20004/20200/20500/21500/23500', async () => {
  const cmds = [20002, 20004, 20200, 20500, 21500, 23500];
  for (const cmd of cmds) {
    const r = await sendFrame(cmd, Buffer.alloc(0));
    if (r.cmd !== cmd) throw new Error(`${cmd}: cmd mismatch ${r.cmd}`);
  }
});

// T13: 未知 cmd 在 20000-29999 (没有 explicit handler) - fallback
test('fallback 25400 (无 explicit handler)', async () => {
  const r = await sendFrame(25400, Buffer.alloc(0));
  if (r.cmd !== 25400) throw new Error(`cmd mismatch: ${r.cmd}`);
  // fallback: code:u8 + msg:str = 0 + "OK" = 1+4+2 = 7 bytes
  if (r.payload.length < 5) throw new Error(`payload too short: ${r.payload.length}`);
});

// ============================================================================
// 运行
// ============================================================================

(async () => {
  let pass = 0, fail = 0;
  for (const t of tests) {
    try {
      await t.fn();
      console.log(`  PASS  ${t.name}`);
      pass++;
    } catch (e) {
      console.log(`  FAIL  ${t.name}: ${e.message}`);
      fail++;
    }
  }
  console.log(`\n  ${pass}/${pass + fail} passed`);
  process.exit(fail === 0 ? 0 : 1);
})();
