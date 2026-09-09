// w5 byte-level test for 9 admin-gm cmds (per 2026-09-09 19:32 JST)
// 14100-14104 from proto_141.erl (签到/checkin)
// 30001-30102 from proto_mate.js only (无 erl proto, srv 走空回)

const net = require('net');
const PORT = process.env.SHIM_PORT || 9005;

function makeFrame(cmd, payload) {
  const buf = Buffer.alloc(6 + payload.length);
  buf.writeUInt32BE(2 + payload.length, 0);
  buf.writeUInt16BE(cmd, 4);
  if (payload.length) payload.copy(buf, 6);
  return buf;
}

function parseFrame(buf) {
  if (buf.length < 6) return null;
  const len = buf.readUInt32BE(0);
  const cmd = buf.readUInt16BE(4);
  return { len, cmd, payload: buf.slice(6, 4 + len) };
}

function send(cmd, payload) {
  return new Promise((resolve, reject) => {
    const sock = net.connect(PORT, '127.0.0.1');
    const chunks = [];
    const timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, 3000);
    sock.on('connect', () => sock.write(makeFrame(cmd, payload)));
    sock.on('data', d => chunks.push(d));
    sock.on('end', () => {
      clearTimeout(timer);
      const all = Buffer.concat(chunks);
      const f = parseFrame(all);
      resolve(f);
    });
    sock.on('error', e => { clearTimeout(timer); reject(e); });
  });
}

(async () => {
  const tests = [
    // 14100 cli: empty → srv: {day:u8, status:u8} per proto_141.erl
    { cmd: 14100, payload: Buffer.alloc(0), expectPayload: 2, name: '14100 checkin_get' },
    // 14101 cli: empty → srv: {code:u8, msg:str, day:u8, status:u8}
    { cmd: 14101, payload: Buffer.alloc(0), expectPayloadMin: 5, name: '14101 checkin_submit' },
    // 14102 cli: empty → srv: {attr_list_len:u16, [id:u32, status:u8]*}
    { cmd: 14102, payload: Buffer.alloc(0), expectPayload: 7, name: '14102 checkin_attr_list' },
    // 14103 cli: {id:u8} → srv: {code:u8, msg:str, id:u32, status:u8}
    { cmd: 14103, payload: Buffer.from([5]), expectPayloadMin: 5, name: '14103 checkin_claim' },
    // 14104 cli: empty → srv: empty (push only)
    { cmd: 14104, payload: Buffer.alloc(0), expectPayload: 0, name: '14104 checkin_done push' },
    // 30001 cli: {id:u32, finish:u8, target_val:u32, value:u32} → srv: empty
    { cmd: 30001, payload: Buffer.alloc(14, 1), expectPayload: 0, name: '30001 gift_progress' },
    // 30002 cli: {code:u8, msg:str} → srv: empty
    { cmd: 30002, payload: Buffer.alloc(2, 0), expectPayload: 0, name: '30002 gift_err_report' },
    // 30100 cli: {flag:u8, msg:str} → srv: empty
    { cmd: 30100, payload: Buffer.alloc(2, 0), expectPayload: 0, name: '30100 gift_flag_report' },
    // 30101 cli: {code:u8} → srv: empty
    { cmd: 30101, payload: Buffer.from([1]), expectPayload: 0, name: '30101 gift_ack_1' },
    // 30102 cli: {code:u8} → srv: empty
    { cmd: 30102, payload: Buffer.from([2]), expectPayload: 0, name: '30102 gift_ack_2' },
  ];

  let pass = 0, fail = 0;
  for (const t of tests) {
    try {
      const f = await send(t.cmd, t.payload);
      const ok = f.cmd === t.cmd
        && (t.expectPayload !== undefined ? f.payload.length === t.expectPayload : f.payload.length >= t.expectPayloadMin);
      if (ok) {
        pass++;
        console.log(`  PASS  ${t.name} cmd=${f.cmd} payload=${f.payload.length}B hex=${f.payload.toString('hex')}`);
      } else {
        fail++;
        console.log(`  FAIL  ${t.name} cmd=${f.cmd} payload=${f.payload.length}B hex=${f.payload.toString('hex')} (expect ${t.expectPayload ?? '>='+t.expectPayloadMin})`);
      }
    } catch (e) {
      fail++;
      console.log(`  ERR   ${t.name} ${e.message}`);
    }
  }
  console.log(`\n=== w5: ${pass}/${pass+fail} passed (9 admin-gm cmds) ===`);
  process.exit(fail > 0 ? 1 : 0);
})();
