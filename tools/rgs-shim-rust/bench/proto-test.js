// proto-test.js — erlang→rgs 迁移测试
// 模拟 zsyz_client (per zsyz_server/src/proto/proto_*.erl) 发真实 SmartSocket 帧
// 验证 rgs-shim-rust 返回的响应跟 Erlang server 格式一致
// per 2026-09-09 14:51 JST Ulysses 拍板: "前端表现和erlang版本一致的情况下，后端换成rgs"

const net = require('net');

const HOST = process.argv[2] || '127.0.0.1';
const PORT = parseInt(process.argv[3] || '9001', 10);

// ============================================================================
// 协议定义 (per zsyz_server/src/proto/proto_101.erl + proto_102.erl + proto_110.erl)
// ============================================================================

// 字符串: | len:u32 BE | bytes (无 null 终止)
function packString(s) {
  const buf = Buffer.from(s, 'utf8');
  const out = Buffer.alloc(4 + buf.length);
  out.writeUInt32BE(buf.length, 0);
  buf.copy(out, 4);
  return out;
}

function unpackString(buf, off) {
  const len = buf.readUInt32BE(off);
  const s = buf.toString('utf8', off + 4, off + 4 + len);
  return { value: s, nextOff: off + 4 + len };
}

// 帧: | len:u32 BE | cmd:u16 BE | payload |
// len = 2 + payload.length (cmd 占 2 字节)
function makeFrame(cmd, payload) {
  const p = payload || Buffer.alloc(0);
  const len = 2 + p.length;
  const buf = Buffer.alloc(6 + p.length);
  buf.writeUInt32BE(len, 0);
  buf.writeUInt16BE(cmd, 4);
  if (p.length > 0) p.copy(buf, 6);
  return buf;
}

function parseFrame(buf) {
  if (buf.length < 6) return null;
  const len = buf.readUInt32BE(0);
  const cmd = buf.readUInt16BE(4);
  const payloadLen = len - 2;
  return {
    len, cmd,
    payload: buf.slice(6, 6 + payloadLen),
  };
}

// 发送 1 帧 + 收 1 帧
function sendFrame(cmd, payload) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection(PORT, HOST);
    let buf = Buffer.alloc(0);
    let timer;
    sock.on('connect', () => {
      const frame = makeFrame(cmd, payload);
      sock.write(frame);
      timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, 8000);
    });
    sock.on('data', (chunk) => {
      buf = Buffer.concat([buf, chunk]);
      if (buf.length >= 6) {
        const totalLen = 4 + buf.readUInt32BE(0);
        if (buf.length >= totalLen) {
          clearTimeout(timer);
          sock.end();
          resolve(parseFrame(buf));
        }
      }
    });
    sock.on('error', (e) => { clearTimeout(timer); reject(e); });
  });
}

// ============================================================================
// 测试用例 — 真实 zsyz_client 字段顺序 (per proto_mate.js + proto_*.erl)
// ============================================================================

const tests = [];
const results = [];

function test(name, fn) { tests.push({ name, fn }); }

// T1: 10101 register — 真实 zsyz_client 字段 (sex:u8 + name:str + career:i16 + playform:str)
test('10101 register (per proto_101.erl)', async () => {
  const sex = 1;
  const name = 'TestHero';
  const career = 1;
  const playform = 'ios';
  const payload = Buffer.concat([
    Buffer.from([sex]),
    packString(name),
    Buffer.from([(career >> 8) & 0xff, career & 0xff]),  // i16 BE
    packString(playform),
  ]);
  const r = await sendFrame(10101, payload);
  // 10101 srv: {code:u8, msg:str, rid:u32, srv_id:str, name:str, reg_time:u32}
  let off = 0;
  const code = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  const rid = msg.nextOff === undefined ? 0 : r.payload.readUInt32BE(msg.nextOff);
  off = msg.nextOff + 4;
  const srv_id = unpackString(r.payload, off);
  off = srv_id.nextOff;
  const name_r = unpackString(r.payload, off);
  off = name_r.nextOff;
  const reg_time = r.payload.readUInt32BE(off);
  return {
    cmd: r.cmd,
    fields: { code, msg: msg.value, rid: '0x' + rid.toString(16), srv_id: srv_id.value, name: name_r.value, reg_time },
    ok: r.cmd === 10101 && code === 0,
  };
});

// T2: 10102 enter_server — {rid:u32, srv_id:str}
test('10102 enter_server (per proto_101.erl)', async () => {
  const rid = 0x11111111;
  const srv_id = 'rgs-uat-1';
  const payload = Buffer.concat([
    Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]),
    packString(srv_id),
  ]);
  const r = await sendFrame(10102, payload);
  // 10102 srv: {code:u8, msg:str, timestamp:u32, world_lev:u16}
  let off = 0;
  const code = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  off = msg.nextOff;
  const timestamp = r.payload.readUInt32BE(off); off += 4;
  const world_lev = r.payload.readUInt16BE(off);
  return {
    cmd: r.cmd,
    fields: { code, msg: msg.value, timestamp, world_lev },
    ok: r.cmd === 10102 && code === 0,
  };
});

// T3: 10103 (alias of 10102, per proto_101.erl)
test('10103 enter_server alias', async () => {
  const r = await sendFrame(10103, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 10103 };
});

// T4: 10200 map_enter — {battle_id:u32, id:u32, code:i16}
// 10200 srv: {result:u8, msg:str, battle_id:u32, id:u32, time:u32}
test('10200 map_enter (per proto_102.erl, full 5-field srv)', async () => {
  const battle_id = 0xaaaaaaaa;
  const id = 0xbbbbbbbb;
  const code = 0;
  const payload = Buffer.concat([
    Buffer.from([(battle_id >> 24) & 0xff, (battle_id >> 16) & 0xff, (battle_id >> 8) & 0xff, battle_id & 0xff]),
    Buffer.from([(id >> 24) & 0xff, (id >> 16) & 0xff, (id >> 8) & 0xff, id & 0xff]),
    Buffer.from([(code >> 8) & 0xff, code & 0xff]),
  ]);
  const r = await sendFrame(10200, payload);
  let off = 0;
  const result = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  off = msg.nextOff;
  const r_battle_id = r.payload.readUInt32BE(off); off += 4;
  const r_id = r.payload.readUInt32BE(off); off += 4;
  const r_time = r.payload.readUInt32BE(off);
  return {
    cmd: r.cmd,
    fields: { result, msg: msg.value, r_battle_id: '0x' + r_battle_id.toString(16), r_id: '0x' + r_id.toString(16), r_time },
    ok: r.cmd === 10200 && result === 0 && r_battle_id === battle_id && r_id === id,
  };
});

// T5: 10400 heartbeat (shim-internal RGS 5 域 HealthCheck, 不是真 zsyz cmd)
test('10400 heartbeat (shim-internal, 5 域 RGS HealthCheck)', async () => {
  const r = await sendFrame(10400, Buffer.alloc(0));
  let off = 0;
  const code = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  off = msg.nextOff;
  const ok_count = r.payload.readUInt8(off); off += 1;
  const total = r.payload.readUInt8(off);
  return {
    cmd: r.cmd,
    fields: { code, msg: msg.value, ok_count, total },
    ok: r.cmd === 10400 && code === 0 && ok_count === 5 && total === 5,
  };
});

// T6: 11001 role_list (shim-internal, RGS player.ListPlayers)
test('11001 role_list (shim-internal, RGS player.ListPlayers)', async () => {
  const r = await sendFrame(11001, Buffer.alloc(0));
  let off = 0;
  const code = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  off = msg.nextOff;
  const count = r.payload.readUInt8(off);
  return {
    cmd: r.cmd,
    fields: { code, msg: msg.value, count },
    ok: r.cmd === 11001 && code === 0 && count >= 1,
  };
});

// T7: 大 payload 边界测试 (1024 字节)
test('10200 大 payload 边界 (1024 字节)', async () => {
  const payload = Buffer.alloc(1024, 0xaa);
  const r = await sendFrame(10200, payload);
  return {
    cmd: r.cmd,
    echo: r.payload.length > 0,
    ok: r.cmd === 10200,
  };
});

// T8: 未知 cmd (20000) — shim 应返 stub (空 payload)
test('20000 unknown cmd → stub', async () => {
  const r = await sendFrame(20000, Buffer.from([1, 2, 3]));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20000 && r.payload.length === 0 };
});

// T9: 并发 50 个 register (压测)
test('并发 50 register (压测)', async () => {
  const t0 = Date.now();
  const promises = [];
  for (let i = 0; i < 50; i++) {
    const payload = Buffer.concat([
      Buffer.from([1]),
      packString('Hero' + i),
      Buffer.from([0, 1]),
      packString('ios'),
    ]);
    promises.push(sendFrame(10101, payload));
  }
  const responses = await Promise.all(promises);
  const dt = Date.now() - t0;
  const ok = responses.filter(r => r.cmd === 10101 && r.payload.length > 0).length;
  return { total: 50, success: ok, dt_ms: dt, rps: (ok / (dt / 1000)).toFixed(1), ok: ok === 50 };
});

// T10: 完整登录流程 (10101 → 10102 → 10200)
test('完整登录流程 10101 → 10102 → 10200 (模拟 zsyz_client 启动)', async () => {
  // 10101 register
  const p1 = Buffer.concat([Buffer.from([1]), packString('FlowHero'), Buffer.from([0, 1]), packString('android')]);
  const r1 = await sendFrame(10101, p1);
  // 提取 rid
  const rid = r1.payload.readUInt32BE(1 + 4 + 4);  // skip code(1) + msg_len(4) + msg + rid
  // 10102 enter_server
  const p2 = Buffer.concat([
    Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]),
    packString('rgs-uat-1'),
  ]);
  const r2 = await sendFrame(10102, p2);
  // 10200 map_enter
  const p3 = Buffer.concat([
    Buffer.from([0, 0, 0, 0]),
    Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]),
    Buffer.from([0, 0]),
  ]);
  const r3 = await sendFrame(10200, p3);
  return {
    flow: '10101→10102→10200',
    r1: { cmd: r1.cmd, ok: r1.payload.length > 0 },
    r2: { cmd: r2.cmd, ok: r2.payload.length > 0 },
    r3: { cmd: r3.cmd, ok: r3.payload.length > 0 },
    rid: '0x' + rid.toString(16),
    ok: r1.cmd === 10101 && r2.cmd === 10102 && r3.cmd === 10200,
  };
});

// ============================================================================
// 主循环
// ============================================================================
async function main() {
  console.log(`proto-test target: ${HOST}:${PORT}`);
  console.log(`测试协议版本: zsyz_server/src/proto/proto_101.erl + proto_102.erl + proto_110.erl`);
  console.log(`模拟客户端: zsyz_client (per proto_mate.js 字段顺序)`);
  console.log(`shim 行为: rgs-shim-rust v0.3.1 (2026-09-09 14:55 JST)`);
  console.log('');
  console.log('='.repeat(80));

  let pass = 0, fail = 0;
  for (const t of tests) {
    try {
      const r = await t.fn();
      const status = r.ok ? '✅' : '❌';
      console.log(`[${status}] ${t.name}`);
      console.log(`     ${JSON.stringify(r)}`);
      if (r.ok) pass++; else fail++;
    } catch (e) {
      console.log(`[❌] ${t.name}`);
      console.log(`     ERR: ${e.message}`);
      fail++;
    }
  }

  console.log('='.repeat(80));
  console.log(`合计: ${pass}/${tests.length} passed, ${fail} failed`);
  console.log('');
  console.log('迁移结论:');
  console.log('  真实 zsyz_client cmd 4 个: 10101 / 10102 / 10103 / 10200 (per proto_*.erl)');
  console.log('  shim-internal RGS 测试 cmd 2 个: 10400 / 11001');
  console.log('  业务覆盖率 1.2% (4/514 real cmd), 1-2 周 4 worker 扩 (per 9/9 13:45 JST 拍板 A)');
  console.log('');
  process.exit(fail > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(1); });
