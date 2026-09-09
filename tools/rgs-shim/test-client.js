// RGS Shim test client v0.2 — 验证 6 cmd (10101/10102/10200/10400/11001 + 10103 alias)
const net = require('node:net');

function beU8(v) { const b = Buffer.alloc(1); b[0] = v & 0xff; return b; }
function beU16(v) { const b = Buffer.alloc(2); b[0] = (v>>8) & 0xff; b[1] = v & 0xff; return b; }
function beU32(v) { const b = Buffer.alloc(4); b[0]=(v>>>24)&0xff; b[1]=(v>>>16)&0xff; b[2]=(v>>>8)&0xff; b[3]=v&0xff; return b; }
function beI16(v) { return beU16(v & 0xffff); }
function beStr(s) { const sBuf = Buffer.from(s, 'utf8'); return Buffer.concat([beU32(sBuf.length), sBuf]); }
function frame(cmd, payload) { return Buffer.concat([beU32(2 + payload.length), beU16(cmd), payload]); }
function readFrame(buf) {
  if (buf.length < 6) return null;
  const len = buf.readUInt32BE(0);
  const totalLen = 4 + len;
  if (buf.length < totalLen) return null;
  return { cmd: buf.readUInt16BE(4), payload: buf.slice(6, totalLen) };
}
function parseStrAt(p, off) {
  const len = p.readUInt32BE(off);
  return { v: p.slice(off+4, off+4+len).toString('utf8'), n: 4+len };
}

const HOST = '127.0.0.1';
const PORT = Number(process.env.SHIM_PORT || 9001);

const sock = net.createConnection(PORT, HOST, () => {
  console.log(`[test] connected to ${HOST}:${PORT}`);
  // 1) Register
  sock.write(frame(10101, Buffer.concat([beU8(0), beStr('MavisHero'), beI16(2), beStr('rgs-flash')])));
  console.log('[test] → 10101 register');
});

let buf = Buffer.alloc(0);
let count = 0;
let step = 0;

function sendNext() {
  step++;
  if (step === 1) {
    // 200ms 后 enter server
    setTimeout(() => {
      sock.write(frame(10102, Buffer.concat([beU32(0x11111111), beStr('rgs-uat-1')])));
      console.log('[test] → 10102 enter_server');
    }, 200);
  } else if (step === 2) {
    // 200ms 后 enter_server alias
    setTimeout(() => {
      sock.write(frame(10103, Buffer.concat([beU32(0x11111111), beStr('rgs-uat-1')])));
      console.log('[test] → 10103 enter_server (alias)');
    }, 200);
  } else if (step === 3) {
    // map_enter
    setTimeout(() => {
      sock.write(frame(10200, Buffer.concat([beU32(1), beU32(100), beI16(0)])));
      console.log('[test] → 10200 map_enter');
    }, 200);
  } else if (step === 4) {
    // heartbeat
    setTimeout(() => {
      sock.write(frame(10400, Buffer.alloc(0)));
      console.log('[test] → 10400 heartbeat');
    }, 200);
  } else if (step === 5) {
    // role list
    setTimeout(() => {
      sock.write(frame(11001, Buffer.alloc(0)));
      console.log('[test] → 11001 role_list');
    }, 200);
  } else if (step === 6) {
    setTimeout(() => { console.log('[test] closing'); sock.end(); }, 500);
  }
}

sock.on('data', (chunk) => {
  buf = Buffer.concat([buf, chunk]);
  while (true) {
    if (buf.length < 6) break;
    const len = buf.readUInt32BE(0);
    const totalLen = 4 + len;
    if (buf.length < totalLen) break;
    const f = readFrame(buf);
    buf = buf.slice(totalLen);
    count++;
    let p = f.payload, o = 0;
    const code = p[o]; o += 1;
    const m1 = parseStrAt(p, o); o += m1.n;
    if (f.cmd === 10101) {
      const rid = p.readUInt32BE(o); o += 4;
      const m2 = parseStrAt(p, o); o += m2.n;
      const m3 = parseStrAt(p, o); o += m3.n;
      const regTime = p.readUInt32BE(o);
      console.log(`[test] ← 10101: code=${code} msg="${m1.v}" rid=0x${rid.toString(16)} sid="${m2.v}" name="${m3.v}" regTime=${regTime}`);
    } else if (f.cmd === 10102 || f.cmd === 10103) {
      const ts = p.readUInt32BE(o); o += 4;
      const worldLev = p.readUInt16BE(o);
      console.log(`[test] ← ${f.cmd}: code=${code} msg="${m1.v}" ts=${ts} worldLev=${worldLev}`);
    } else if (f.cmd === 10200) {
      console.log(`[test] ← 10200: code=${code} msg="${m1.v}"`);
    } else if (f.cmd === 10400) {
      const ok = p[o]; o += 1;
      const total = p[o];
      console.log(`[test] ← 10400 heartbeat: code=${code} msg="${m1.v}" ok=${ok}/${total}`);
    } else if (f.cmd === 11001) {
      const total = p[o]; o += 1;
      const players = [];
      for (let i = 0; i < total; i++) {
        const m = parseStrAt(p, o); o += m.n;
        const lv = p[o]; o += 1;
        players.push(`${m.v}(lv${lv})`);
      }
      console.log(`[test] ← 11001 role_list: code=${code} msg="${m1.v}" players=[${players.join(', ')}]`);
    } else {
      console.log(`[test] ← ${f.cmd}: code=${code} msg="${m1.v}" payload_len=${f.payload.length}`);
    }
    sendNext();
  }
});
sock.on('close', () => { console.log(`[test] closed (${count} frames)`); process.exit(0); });
sock.on('error', (e) => { console.error('[test] error:', e.message); process.exit(1); });
