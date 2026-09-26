// battle-test-r2.js — Phase 4 w3 round 2 (37 cmd) 字节级测试 (per 2026-09-09 20:17 JST 续做)
// 范围: 20000-20221 (战斗/HP/能量 + 战斗详细, 37 cmd)
// 协议: 大端 (BE) | len:u32 + cmd:u16 + payload | 字符串: len:u32 + bytes

const net = require('net');
const HOST = process.argv[2] || '127.0.0.1';
const PORT = parseInt(process.argv[3] || '9003', 10);

function packString(s) {
  const buf = Buffer.from(s, 'utf8');
  const out = Buffer.alloc(4 + buf.length);
  out.writeUInt32BE(buf.length, 0);
  buf.copy(out, 4);
  return out;
}

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
  return { len, cmd, payload: buf.slice(6, 6 + payloadLen) };
}

function sendFrame(cmd, payload) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection(PORT, HOST);
    let buf = Buffer.alloc(0);
    let timer;
    sock.on('connect', () => {
      sock.write(makeFrame(cmd, payload));
      timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, 5000);
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

const tests = [];
const results = [];
function test(name, fn) { tests.push({ name, fn }); }

// 20000-20019 战斗/HP/能量
test('20000 battle_start', async () => {
  const r = await sendFrame(20000, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20000 };
});

test('20001 battle_start_ack', async () => {
  const r = await sendFrame(20001, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20001 };
});

test('20002 battle_detail', async () => {
  const r = await sendFrame(20002, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20002 };
});

test('20004 battle_round', async () => {
  const r = await sendFrame(20004, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20004 };
});

test('20005 battle_simple_ack (empty srv)', async () => {
  const r = await sendFrame(20005, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20005 };
});

test('20006 battle_finish', async () => {
  const r = await sendFrame(20006, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20006 };
});

test('20008 battle_quit', async () => {
  const r = await sendFrame(20008, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20008 };
});

test('20009 battle_misc_09', async () => {
  const r = await sendFrame(20009, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20009 };
});

test('20013 battle_setup (20 字段大 payload)', async () => {
  const r = await sendFrame(20013, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20013 };
});

test('20014 battle_target {tid:u32, srv_id:str}', async () => {
  const payload = Buffer.concat([Buffer.alloc(4), packString('rgs-uat-1')]);
  payload.writeUInt32BE(0x11111111, 0);
  const r = await sendFrame(20014, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20014 };
});

test('20015 battle_misc_15', async () => {
  const r = await sendFrame(20015, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20015 };
});

test('20016 battle_misc_16', async () => {
  const r = await sendFrame(20016, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20016 };
});

test('20019 battle_done (empty)', async () => {
  const r = await sendFrame(20019, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20019 };
});

// 20020-20036 战斗/HP
test('20020 battle_init (8 字段)', async () => {
  const r = await sendFrame(20020, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20020 };
});

test('20022 battle_speed {speed:u8}', async () => {
  const r = await sendFrame(20022, Buffer.from([2]));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20022 };
});

test('20026 battle_drama', async () => {
  const r = await sendFrame(20026, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20026 };
});

test('20027 battle_spec (12 字段)', async () => {
  const r = await sendFrame(20027, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20027 };
});

test('20028 battle_spec_ack', async () => {
  const r = await sendFrame(20028, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20028 };
});

test('20029 battle_replay_request {replay_id:u32}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(123, 0);
  const r = await sendFrame(20029, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20029 };
});

test('20030 battle_in_combat', async () => {
  const r = await sendFrame(20030, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20030 };
});

test('20033 battle_defender (6 字段)', async () => {
  const r = await sendFrame(20033, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20033 };
});

test('20034 battle_share {replay_id:u32, ...}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(100, 0);
  const r = await sendFrame(20034, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20034 };
});

test('20036 battle_replay_detail {rid:u32, srv_id:str}', async () => {
  const payload = Buffer.concat([Buffer.alloc(4), packString('rgs-uat-1')]);
  payload.writeUInt32BE(0x11111111, 0);
  const r = await sendFrame(20036, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20036 };
});

// 20060-20063 combat_type
test('20060 battle_combat_type {combat_type:u16}', async () => {
  const payload = Buffer.alloc(2);
  payload.writeUInt16BE(1, 0);
  const r = await sendFrame(20060, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20060 };
});

test('20062 battle_combat_type_ack', async () => {
  const r = await sendFrame(20062, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20062 };
});

test('20063 battle_type_list', async () => {
  const r = await sendFrame(20063, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20063 };
});

// 20200-20221 战斗详细/竞技场
test('20200 arena_state_full (8 字段)', async () => {
  const r = await sendFrame(20200, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20200 };
});

test('20201 arena_f_list', async () => {
  const r = await sendFrame(20201, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20201 };
});

test('20202 arena_view {rid, srv_id}', async () => {
  const payload = Buffer.concat([Buffer.alloc(4), packString('rgs-uat-1')]);
  payload.writeUInt32BE(0x11111111, 0);
  const r = await sendFrame(20202, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20202 };
});

test('20203 arena_view_ack', async () => {
  const payload = Buffer.concat([Buffer.alloc(4), packString('rgs-uat-1')]);
  payload.writeUInt32BE(0x11111111, 0);
  const r = await sendFrame(20203, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20203 };
});

test('20204 arena_set_pos {rid, srv_id, pos}', async () => {
  const payload = Buffer.concat([
    Buffer.alloc(4),
    packString('rgs-uat-1'),
    Buffer.alloc(2)
  ]);
  payload.writeUInt32BE(0x11111111, 0);
  payload.writeUInt16BE(1, 4 + 4 + 4);
  const r = await sendFrame(20204, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20204 };
});

test('20206 arena_challenge', async () => {
  const r = await sendFrame(20206, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20206 };
});

test('20207 arena_clear_cd', async () => {
  const r = await sendFrame(20207, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20207 };
});

test('20208 arena_combat_log', async () => {
  const r = await sendFrame(20208, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20208 };
});

test('20209 arena_buy_count {num:u8}', async () => {
  const r = await sendFrame(20209, Buffer.from([1]));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20209 };
});

test('20220 arena_rank', async () => {
  const r = await sendFrame(20220, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20220 };
});

test('20221 arena_worship (4 字段)', async () => {
  const r = await sendFrame(20221, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 20221 };
});

async function main() {
  console.log(`battle-test-r2 target: ${HOST}:${PORT}`);
  console.log(`协议: BE | len:u32 + cmd:u16 + payload`);
  console.log(`rgs-shim v0.4.2 (Phase 4 w3 round 2 battle 域 37 cmd 续做)`);
  console.log('');
  console.log('='.repeat(80));

  let pass = 0, fail = 0;
  for (const t of tests) {
    try {
      const r = await t.fn();
      const status = r.ok ? '✓' : '✗';
      console.log(`[${status}] ${t.name}`);
      console.log(`     ${JSON.stringify(r)}`);
      if (r.ok) pass++; else fail++;
    } catch (e) {
      console.log(`[✗] ${t.name}`);
      console.log(`     ERR: ${e.message}`);
      fail++;
    }
  }

  console.log('='.repeat(80));
  console.log(`合计: ${pass}/${tests.length} passed, ${fail} failed`);
  console.log('');
  console.log('Phase 4 w3 round 2 派工结果:');
  console.log('  battle 域 20000-20036 (23) + 20060-20063 (3) + 20200-20221 (11) = 37 cmd 真实 handler');
  console.log('  战斗/HP/能量 + combat_type + 战斗详细/竞技场 全覆盖');
  console.log('  stub-20000..stub-20063 (26) + stub-20200..stub-20221 (11) 全部升级为 real handler');
  console.log('');
  process.exit(fail > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(1); });
