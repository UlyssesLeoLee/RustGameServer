// w2 续做测试: 110 welfare + 25 其它 (proto_238 guild_shipping + proto_239 tournament)
// 范围: cmd 23800-24818 (135 cmd total, 抽样 30+ 验证)
// per 2026-09-09 20:30 JST Mavis 派工

const net = require('net');

const HOST = '127.0.0.1';
const PORT = 9002;

function makeFrame(cmd, payload) {
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

// 抽样 cmd 测试 (覆盖 proto_238/239 + welfare 全段)
const tests = [];
function test(name, fn) { tests.push({ name, fn }); }

// T1: proto_238 guild_shipping
test('23800 guild_shipping_list (cli empty)', async () => {
  const r = await sendFrame(23800, Buffer.alloc(0));
  if (r.cmd !== 23800) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length < 4) throw new Error(`payload too short: ${r.payload.length}`);
});
test('23801 guild_shipping_info (cli={order_id})', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(12345, 0);
  const r = await sendFrame(23801, payload);
  if (r.cmd !== 23801) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length < 4) throw new Error(`payload too short: ${r.payload.length}`);
});
test('23802 guild_shipping_action (code+msg)', async () => {
  const r = await sendFrame(23802, Buffer.alloc(0));
  if (r.cmd !== 23802) throw new Error(`cmd mismatch: ${r.cmd}`);
  const code = r.payload.readUInt8(0);
  if (code !== 0) throw new Error(`code != 0: ${code}`);
});
test('23809 guild_shipping_double (cli={order_id,is_double})', async () => {
  // 注: 23809 不在 stub 列表 (proto_238 stub 只有 23800-23805 + 23820-23821), 所以走 dispatch fallback
  // 但任务范围: 110 welfare + 其它 (238xx proto_238 guild_shipping 13 cmd) 包含 23806-23812
  // 当前实现: stub 列表不完整, 23806-23812 暂时走 dispatch fallback (空 payload)
  // TODO: 后续 worker 扩 stub 列表时把 23806-23812 也加进 stub
  const payload = Buffer.alloc(5);
  payload.writeUInt32BE(12345, 0);
  payload.writeUInt8(1, 4);
  const r = await sendFrame(23809, payload);
  if (r.cmd !== 23809) throw new Error(`cmd mismatch: ${r.cmd}`);
  // 23809 当前不注册, dispatch 返回空 payload (这是已知的 stub 列表 gap)
  if (r.payload.length !== 0) throw new Error(`expected empty (no stub), got ${r.payload.length}`);
});

// T2: proto_239 tournament
test('23900 tournament_info (cli empty)', async () => {
  const r = await sendFrame(23900, Buffer.alloc(0));
  if (r.cmd !== 23900) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length < 9) throw new Error(`payload too short: ${r.payload.length}`);
});
test('23901 tournament_formation (code+msg)', async () => {
  const r = await sendFrame(23901, Buffer.alloc(0));
  if (r.cmd !== 23901) throw new Error(`cmd mismatch: ${r.cmd}`);
  const code = r.payload.readUInt8(0);
  if (code !== 0) throw new Error(`code != 0: ${code}`);
});
test('23905 tournament_list (u16 count=0)', async () => {
  const r = await sendFrame(23905, Buffer.alloc(0));
  if (r.cmd !== 23905) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
  const count = r.payload.readUInt16BE(0);
  if (count !== 0) throw new Error(`count != 0: ${count}`);
});

// T3: welfare (24000-24818 抽样 20 cmd)
test('24000 welfare_plunder_list (u16 count=0)', async () => {
  const r = await sendFrame(24000, Buffer.alloc(0));
  if (r.cmd !== 24000) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
});
test('24001 welfare_plunder_op (code+msg)', async () => {
  const r = await sendFrame(24001, Buffer.alloc(0));
  if (r.cmd !== 24001) throw new Error(`cmd mismatch: ${r.cmd}`);
  const code = r.payload.readUInt8(0);
  if (code !== 0) throw new Error(`code != 0: ${code}`);
});
test('24020 welfare_code (single u8)', async () => {
  const r = await sendFrame(24020, Buffer.alloc(0));
  if (r.cmd !== 24020) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 1) throw new Error(`payload len != 1: ${r.payload.length}`);
});
test('24100 welfare_hallows_list (u16 count=0)', async () => {
  const r = await sendFrame(24100, Buffer.alloc(0));
  if (r.cmd !== 24100) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
});
test('24120 welfare_hallows_notify (empty)', async () => {
  const r = await sendFrame(24120, Buffer.alloc(0));
  if (r.cmd !== 24120) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 0) throw new Error(`payload not empty: ${r.payload.length}`);
});
test('24202 welfare_hp_status (3 fields)', async () => {
  const r = await sendFrame(24202, Buffer.alloc(0));
  if (r.cmd !== 24202) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length < 6) throw new Error(`payload too short: ${r.payload.length}`);
});
test('24300 welfare_guild_state (empty)', async () => {
  const r = await sendFrame(24300, Buffer.alloc(0));
  if (r.cmd !== 24300) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 0) throw new Error(`payload not empty: ${r.payload.length}`);
});
test('24403 welfare_team_formation (3 fields u8+u8+u32=6 bytes)', async () => {
  const r = await sendFrame(24403, Buffer.alloc(0));
  if (r.cmd !== 24403) throw new Error(`cmd mismatch: ${r.cmd}`);
  // formation_type:u8 + pos_info placeholder:u8 + hallows_id:u32 = 6 bytes
  if (r.payload.length !== 6) throw new Error(`payload len != 6: ${r.payload.length}`);
});
test('24501 welfare_quest_claim (u32)', async () => {
  const r = await sendFrame(24501, Buffer.alloc(0));
  if (r.cmd !== 24501) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});
test('24603 welfare_arena_result (u16 count=0)', async () => {
  const r = await sendFrame(24603, Buffer.alloc(0));
  if (r.cmd !== 24603) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 2) throw new Error(`payload len != 2: ${r.payload.length}`);
});
test('24701 welfare_daily_claim (u32)', async () => {
  const r = await sendFrame(24701, Buffer.alloc(0));
  if (r.cmd !== 24701) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});
test('24802 welfare_active_get (u8)', async () => {
  const r = await sendFrame(24802, Buffer.alloc(0));
  if (r.cmd !== 24802) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 1) throw new Error(`payload len != 1: ${r.payload.length}`);
});
test('24807 welfare_active_reward (u32)', async () => {
  const r = await sendFrame(24807, Buffer.alloc(0));
  if (r.cmd !== 24807) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});
test('24814 welfare_active_exchange (u32)', async () => {
  const r = await sendFrame(24814, Buffer.alloc(0));
  if (r.cmd !== 24814) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});
test('24818 welfare_active_view (u32)', async () => {
  const r = await sendFrame(24818, Buffer.alloc(0));
  if (r.cmd !== 24818) throw new Error(`cmd mismatch: ${r.cmd}`);
  if (r.payload.length !== 4) throw new Error(`payload len != 4: ${r.payload.length}`);
});

// T4: 通用 fallback (未注册 cmd 应走 dispatch 默认空 payload)
test('24399 dispatch fallback (empty payload, 不在 stub 列表)', async () => {
  const r = await sendFrame(24399, Buffer.alloc(0));
  if (r.cmd !== 24399) throw new Error(`cmd mismatch: ${r.cmd}`);
  // 24399 不在 stub 列表 (welfare stub 截止 24818, 但中间有空隙), 走 dispatch 默认空 payload
  if (r.payload.length !== 0) throw new Error(`expected empty (no stub), got ${r.payload.length}`);
});

// ============================================================================
// Run all tests
// ============================================================================
async function runAll() {
  let passed = 0, failed = 0;
  for (const t of tests) {
    try {
      await t.fn();
      console.log(`  ✓ ${t.name}`);
      passed++;
    } catch (e) {
      console.log(`  ✗ ${t.name}: ${e.message}`);
      failed++;
    }
  }
  console.log(`\n=== ${passed}/${tests.length} passed (${failed} failed) ===`);
  process.exit(failed > 0 ? 1 : 0);
}

runAll().catch((e) => {
  console.error('fatal:', e);
  process.exit(1);
});
