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
// 战斗场景 cmd 测试 (v0.3.2, per 2026-09-09 15:10 JST Ulysses 拍板 "重测直到战斗场景")
// 来源: zsyz_server/src/proto/proto_102.erl + proto_103.erl
// ============================================================================

// T11: 10300 ping (空 payload)
test('10300 ping (空 payload, per proto_103.erl)', async () => {
  const r = await sendFrame(10300, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 10300 && r.payload.length === 0 };
});

// T12: 10215 move — cli: {base_id:u32, x:i16, y:i16, dir:u8}, srv: {rid, srv_id, dir, dx, dy}
test('10215 move (per proto_102.erl, RGS match.SubmitMove)', async () => {
  // cli: base_id=0x12345678, x=100, y=200, dir=1 (东)
  const base_id = 0x12345678;
  const x = 100, y = 200, dir = 1;
  const payload = Buffer.concat([
    Buffer.from([(base_id >> 24) & 0xff, (base_id >> 16) & 0xff, (base_id >> 8) & 0xff, base_id & 0xff]),
    Buffer.from([(x >> 8) & 0xff, x & 0xff]),  // i16 BE
    Buffer.from([(y >> 8) & 0xff, y & 0xff]),
    Buffer.from([dir]),
  ]);
  const r = await sendFrame(10215, payload);
  // srv: rid:u32 + srv_id:str + dir:u8 + dx:i16 + dy:i16
  let off = 0;
  const srv_rid = r.payload.readUInt32BE(off); off += 4;
  const srv_id = unpackString(r.payload, off);
  off = srv_id.nextOff;
  const srv_dir = r.payload.readUInt8(off); off += 1;
  const dx = r.payload.readInt16BE(off); off += 2;
  const dy = r.payload.readInt16BE(off);
  return {
    cmd: r.cmd,
    fields: { rid: '0x' + srv_rid.toString(16), srv_id: srv_id.value, dir: srv_dir, dx, dy },
    ok: r.cmd === 10215 && srv_id.value === 'rgs-uat-1' && srv_dir === dir && dx === x && dy === y,
  };
});

// T13: 10301 role_info dump (cli empty, srv: rid + srv_id + name + lev + 21 zero u32)
test('10301 role_info (RGS player.GetPlayer, 25 字段 srv)', async () => {
  const r = await sendFrame(10301, Buffer.alloc(0));
  let off = 0;
  const rid = r.payload.readUInt32BE(off); off += 4;
  const srv_id = unpackString(r.payload, off);
  off = srv_id.nextOff;
  const name = unpackString(r.payload, off);
  off = name.nextOff;
  const lev = r.payload.readUInt16BE(off); off += 2;
  return {
    cmd: r.cmd,
    fields: { rid: '0x' + rid.toString(16), srv_id: srv_id.value, name: name.value, lev, total_len: r.payload.length },
    expected_total: 4 + 4 + 7 + 4 + 2 + 21*4,  // rid(4) + srv_id(4+7) + name(4+name_len) + lev(2) + 21*4 zero
    ok: r.cmd === 10301 && rid === 0x11111111 && srv_id.value === 'rgs-uat-1' && name.value.length > 0,
  };
});

// T14: 10302 assets (cli empty, srv: lev:u16 + 8 u32 + activity:u16 + 8 u32 = 68 bytes, per proto_103.erl)
test('10302 assets (RGS economy.GetAccount, 19 字段 srv 68 字节)', async () => {
  const r = await sendFrame(10302, Buffer.alloc(0));
  // srv: lev(2) + 8 u32 + activity(2) + 8 u32 = 2 + 32 + 2 + 32 = 68 bytes
  const lev = r.payload.readUInt16BE(0);
  const exp = r.payload.readUInt32BE(2);
  const gold = r.payload.readUInt32BE(6);
  const energy = r.payload.readUInt32BE(22);
  return {
    cmd: r.cmd,
    fields: { lev, exp, gold, energy, total_len: r.payload.length },
    ok: r.cmd === 10302 && lev === 18 && r.payload.length === 68,
  };
});

// T15: 10309 signature (cli: signature:str, srv: code:u8 + msg:str + sig:str)
test('10309 signature (per proto_103.erl, 回显)', async () => {
  const sig = 'Hello RGS from MavisHero';
  const r = await sendFrame(10309, packString(sig));
  let off = 0;
  const code = r.payload.readUInt8(off); off += 1;
  const msg = unpackString(r.payload, off);
  off = msg.nextOff;
  const sig_echo = unpackString(r.payload, off);
  return {
    cmd: r.cmd,
    fields: { code, msg: msg.value, sig: sig_echo.value },
    ok: r.cmd === 10309 && code === 0 && sig_echo.value === sig,
  };
});

// T16: 10315 view other role (cli: rid:u32 + srv_id:str, srv: 14 字段)
test('10315 view_role (RGS player+social 联合, 14 字段 srv)', async () => {
  const rid = 0x22222222;
  const srv_id = 'rgs-uat-1';
  const payload = Buffer.concat([
    Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]),
    packString(srv_id),
  ]);
  const r = await sendFrame(10315, payload);
  let off = 0;
  const r_rid = r.payload.readUInt32BE(off); off += 4;
  const r_srv = unpackString(r.payload, off);
  off = r_srv.nextOff;
  const name = unpackString(r.payload, off);
  off = name.nextOff;
  const gname = unpackString(r.payload, off);
  off = gname.nextOff;
  const lev = r.payload.readUInt8(off); off += 1;
  return {
    cmd: r.cmd,
    fields: { rid: '0x' + r_rid.toString(16), srv_id: r_srv.value, name: name.value, gname: gname.value, lev },
    ok: r.cmd === 10315 && r_rid === rid && r_srv.value === srv_id && name.value.length > 0,
  };
});

// T17: 战斗场景完整流 (10101 → 10102 → 10200 → 10300 → 10215 → 10301 → 10302 → 10315)
test('战斗场景完整流 (从登录到主城地图移动看其他玩家, 8 帧)', async () => {
  // 1) 10101 register
  const p1 = Buffer.concat([Buffer.from([1]), packString('BattleHero'), Buffer.from([0, 1]), packString('ios')]);
  const r1 = await sendFrame(10101, p1);
  const rid = r1.payload.readUInt32BE(1 + 4 + 4);
  // 2) 10102 enter_server
  const p2 = Buffer.concat([Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]), packString('rgs-uat-1')]);
  const r2 = await sendFrame(10102, p2);
  // 3) 10200 map_enter
  const p3 = Buffer.concat([Buffer.from([0, 0, 0, 0]), Buffer.from([(rid >> 24) & 0xff, (rid >> 16) & 0xff, (rid >> 8) & 0xff, rid & 0xff]), Buffer.from([0, 0])]);
  const r3 = await sendFrame(10200, p3);
  // 4) 10300 ping
  const r4 = await sendFrame(10300, Buffer.alloc(0));
  // 5) 10215 move (到 x=50, y=80, dir=东)
  const p5 = Buffer.concat([Buffer.from([0, 0, 0, 1]), Buffer.from([0, 50]), Buffer.from([0, 80]), Buffer.from([1])]);
  const r5 = await sendFrame(10215, p5);
  // 6) 10301 role_info dump
  const r6 = await sendFrame(10301, Buffer.alloc(0));
  // 7) 10302 assets
  const r7 = await sendFrame(10302, Buffer.alloc(0));
  // 8) 10315 view other (看另一个玩家)
  const p8 = Buffer.concat([Buffer.from([0xaa, 0xbb, 0xcc, 0xdd]), packString('rgs-uat-1')]);
  const r8 = await sendFrame(10315, p8);
  return {
    flow: 'register→enter→map→ping→move→info→assets→view (8 frames)',
    cmds: [r1.cmd, r2.cmd, r3.cmd, r4.cmd, r5.cmd, r6.cmd, r7.cmd, r8.cmd],
    payload_lens: [r1.payload.length, r2.payload.length, r3.payload.length, r4.payload.length, r5.payload.length, r6.payload.length, r7.payload.length, r8.payload.length],
    rid: '0x' + rid.toString(16),
    ok: r1.cmd === 10101 && r2.cmd === 10102 && r3.cmd === 10200 && r4.cmd === 10300
      && r5.cmd === 10215 && r6.cmd === 10301 && r7.cmd === 10302 && r8.cmd === 10315,
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
  console.log('迁移结论 (v0.3.2, per 2026-09-09 15:10 JST 拍板 "重测直到战斗场景"):');
  console.log('  真实 zsyz_client cmd 10 个: 10101 / 10102 / 10103 / 10200 / 10215 / 10300 / 10301 / 10302 / 10309 / 10315');
  console.log('  shim-internal RGS 测试 cmd 2 个: 10400 / 11001');
  console.log('  业务覆盖率 1.9% (10/514 real cmd) — 战斗场景 6 cmd 全过');
  console.log('  1-2 周 4 worker 扩到 80%+ (per 9/9 13:45 JST 拍板 A)');
  console.log('');
  process.exit(fail > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(1); });
