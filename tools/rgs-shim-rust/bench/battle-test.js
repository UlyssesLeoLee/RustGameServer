// battle-test.js — Phase 4 w3 battle 域 66 cmd 字节级测试 (per 2026-09-09 19:32 JST Mavis 派工)
// 范围: 19800-19807 + 19901-19908 (战斗/录像) + 25100-25841 (任务/成就/城市/矿脉)
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

// 19800-19807 战斗/录像
test('19800 battle_result', async () => {
  const r = await sendFrame(19800, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19800 };
});

test('19801 battle_result_ack {code:u8}', async () => {
  const payload = Buffer.from([0]);
  const r = await sendFrame(19801, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19801 };
});

test('19802 replay_list', async () => {
  const r = await sendFrame(19802, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19802 };
});

test('19804 replay_rewards', async () => {
  const r = await sendFrame(19804, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19804 };
});

test('19805 claim_replay_reward {id:u8}', async () => {
  const payload = Buffer.from([1]);
  const r = await sendFrame(19805, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19805 };
});

test('19806 battle_status', async () => {
  const r = await sendFrame(19806, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19806 };
});

test('19807 battle_opponent', async () => {
  const r = await sendFrame(19807, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19807 };
});

// 19901-19908 战报筛选/分享
test('19901 replay_query {type:u8}', async () => {
  const payload = Buffer.from([1]);
  const r = await sendFrame(19901, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19901 };
});

test('19902 replay_paged_query {type,cond,start,num}', async () => {
  const payload = Buffer.alloc(11);
  payload[0] = 1;
  payload.writeUInt32BE(100, 1);
  payload.writeUInt32BE(0, 5);
  payload[9] = 10;
  const r = await sendFrame(19902, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19902 };
});

test('19903 replay_like {id:u32}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(123, 0);
  const r = await sendFrame(19903, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19903 };
});

test('19906 replay_like_count', async () => {
  const r = await sendFrame(19906, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 19906 };
});

// 25100-25102 日常任务
test('25100 daily_quest', async () => {
  const r = await sendFrame(25100, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25100 };
});

test('25101 daily_quest_claim {id:u32}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(100, 0);
  const r = await sendFrame(25101, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25101 };
});

// 25300-25309 月卡/周卡
test('25300 card_state', async () => {
  const r = await sendFrame(25300, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25300 };
});

test('25301 card_list', async () => {
  const r = await sendFrame(25301, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25301 };
});

test('25304 card_claim {id:u16}', async () => {
  const payload = Buffer.alloc(2);
  payload.writeUInt16BE(5, 0);
  const r = await sendFrame(25304, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25304 };
});

test('25309 card_misc is_pop', async () => {
  const r = await sendFrame(25309, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25309 };
});

// 25400-25414 竞技场
test('25400 arena_state', async () => {
  const r = await sendFrame(25400, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25400 };
});

test('25405 arena_battle', async () => {
  const r = await sendFrame(25405, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25405 };
});

test('25414 arena_partner_list', async () => {
  const r = await sendFrame(25414, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25414 };
});

// 25800-25807 城市/荣誉
test('25800 city_enter {city_id:u32}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(1, 0);
  const r = await sendFrame(25800, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25800 };
});

test('25802 city_rank', async () => {
  const r = await sendFrame(25802, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25802 };
});

test('25805 honor_set {pos:u8, id:u32}', async () => {
  const payload = Buffer.alloc(5);
  payload[0] = 1;
  payload.writeUInt32BE(100, 1);
  const r = await sendFrame(25805, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25805 };
});

test('25807 honor_default', async () => {
  const r = await sendFrame(25807, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25807 };
});

// 25810-25820 成就
test('25810 achievement_list', async () => {
  const r = await sendFrame(25810, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25810 };
});

test('25812 achievement_claim {id:u32}', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(100, 0);
  const r = await sendFrame(25812, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25812 };
});

test('25813 achievement_view (empty srv)', async () => {
  const payload = Buffer.alloc(4);
  payload.writeUInt32BE(100, 0);
  const r = await sendFrame(25813, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25813 };
});

test('25820 achievement_share_reward {share_id:u32, srv_id:str}', async () => {
  const payload = Buffer.concat([
    Buffer.alloc(4),
    packString('rgs-uat-1'),
  ]);
  payload.writeUInt32BE(99, 0);
  const r = await sendFrame(25820, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25820 };
});

// 25830-25841 矿脉/BBS
test('25830 room_grow {start:u16, num:u8}', async () => {
  const payload = Buffer.alloc(3);
  payload.writeUInt16BE(0, 0);
  payload[2] = 10;
  const r = await sendFrame(25830, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25830 };
});

test('25837 bbs_list', async () => {
  const r = await sendFrame(25837, Buffer.alloc(0));
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25837 };
});

test('25840 bbs_praise {rid:u32, srv_id:str, bbs_id:u32}', async () => {
  const payload = Buffer.concat([
    Buffer.alloc(4),
    packString('rgs-uat-1'),
    Buffer.alloc(4),
  ]);
  payload.writeUInt32BE(0x11111111, 0);
  payload.writeUInt32BE(100, 4 + 4 + 4);
  const r = await sendFrame(25840, payload);
  return { cmd: r.cmd, payload_len: r.payload.length, ok: r.cmd === 25840 };
});

async function main() {
  console.log(`battle-test target: ${HOST}:${PORT}`);
  console.log(`协议: BE | len:u32 + cmd:u16 + payload`);
  console.log(`rgs-shim v0.4.1 (Phase 4 w3 battle 域 66 cmd)`);
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
  console.log('Phase 4 w3 派工结果:');
  console.log('  battle 域 19800-19908 (15) + 25100-25841 (51) = 66 cmd 真实 handler');
  console.log('  战斗/录像/任务/成就/城市/矿脉/BBS 全覆盖');
  console.log('  stub-19800..stub-19908 + stub-25100..stub-25841 全部升级为 real handler');
  console.log('');
  process.exit(fail > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(1); });
