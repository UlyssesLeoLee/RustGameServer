// w5 single-connection test (avoid shim crash from many connections)
const net = require('net');
const PORT = process.env.SHIM_PORT || 9005;

function makeFrame(cmd, payload) {
  const buf = Buffer.alloc(6 + payload.length);
  buf.writeUInt32BE(2 + payload.length, 0);
  buf.writeUInt16BE(cmd, 4);
  if (payload.length) payload.copy(buf, 6);
  return buf;
}

const tests = [
  { cmd: 14100, payload: Buffer.alloc(0), expectPayload: 2, name: '14100 checkin_get' },
  { cmd: 14101, payload: Buffer.alloc(0), expectPayloadMin: 5, name: '14101 checkin_submit' },
  { cmd: 14102, payload: Buffer.alloc(0), expectPayload: 7, name: '14102 checkin_attr_list' },
  { cmd: 14103, payload: Buffer.from([5]), expectPayloadMin: 5, name: '14103 checkin_claim' },
  { cmd: 14104, payload: Buffer.alloc(0), expectPayload: 0, name: '14104 checkin_done push' },
  { cmd: 30001, payload: Buffer.alloc(14, 1), expectPayload: 0, name: '30001 gift_progress' },
  { cmd: 30002, payload: Buffer.alloc(2, 0), expectPayload: 0, name: '30002 gift_err_report' },
  { cmd: 30100, payload: Buffer.alloc(2, 0), expectPayload: 0, name: '30100 gift_flag_report' },
  { cmd: 30101, payload: Buffer.from([1]), expectPayload: 0, name: '30101 gift_ack_1' },
  { cmd: 30102, payload: Buffer.from([2]), expectPayload: 0, name: '30102 gift_ack_2' },
];

const sock = net.connect(PORT, '127.0.0.1');
let recvBuf = Buffer.alloc(0);
let testIdx = 0;
let pass = 0, fail = 0;

function sendNext() {
  if (testIdx >= tests.length) {
    console.log(`\n=== w5 single-conn: ${pass}/${pass+fail} passed ===`);
    sock.end();
    process.exit(fail > 0 ? 1 : 0);
  }
  const t = tests[testIdx++];
  sock.write(makeFrame(t.cmd, t.payload));
  console.log(`>> ${t.name} cmd=${t.cmd}`);
}

sock.on('connect', sendNext);
sock.on('data', d => {
  recvBuf = Buffer.concat([recvBuf, d]);
  while (recvBuf.length >= 6) {
    const len = recvBuf.readUInt32BE(0);
    if (recvBuf.length < 4 + len) break;
    const cmd = recvBuf.readUInt16BE(4);
    const payload = recvBuf.slice(6, 4 + len);
    const t = tests[testIdx - 1];
    const ok = cmd === t.cmd
      && (t.expectPayload !== undefined ? payload.length === t.expectPayload : payload.length >= t.expectPayloadMin);
    if (ok) {
      pass++;
      console.log(`  PASS  cmd=${cmd} payload=${payload.length}B hex=${payload.toString('hex')}`);
    } else {
      fail++;
      console.log(`  FAIL  cmd=${cmd} payload=${payload.length}B (expect ${t.expectPayload ?? '>='+t.expectPayloadMin})`);
    }
    recvBuf = recvBuf.slice(4 + len);
    setImmediate(sendNext);
  }
});
sock.on('error', e => { console.log('error:', e.message); process.exit(2); });
setTimeout(() => { console.log('TIMEOUT'); process.exit(3); }, 10000);
