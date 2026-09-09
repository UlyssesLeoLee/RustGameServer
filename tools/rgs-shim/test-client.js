// RGS Shim test client (verify shim works)
// Sends cmd 10101 (register) + cmd 10102 (enter server), prints responses.

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
function parseSrv10101(p) {
  let off = 0;
  const code = p[off]; off += 1;
  const msgLen = p.readUInt32BE(off); off += 4;
  const msg = p.slice(off, off+msgLen).toString('utf8'); off += msgLen;
  const rid = p.readUInt32BE(off); off += 4;
  const sidLen = p.readUInt32BE(off); off += 4;
  const sid = p.slice(off, off+sidLen).toString('utf8'); off += sidLen;
  const nameLen = p.readUInt32BE(off); off += 4;
  const name = p.slice(off, off+nameLen).toString('utf8'); off += nameLen;
  const regTime = p.readUInt32BE(off);
  return { code, msg, rid, sid, name, regTime };
}
function parseSrv10102(p) {
  let off = 0;
  const code = p[off]; off += 1;
  const msgLen = p.readUInt32BE(off); off += 4;
  const msg = p.slice(off, off+msgLen).toString('utf8'); off += msgLen;
  const ts = p.readUInt32BE(off); off += 4;
  const worldLev = p.readUInt16BE(off);
  return { code, msg, ts, worldLev };
}

const HOST = '127.0.0.1';
const PORT = Number(process.env.SHIM_PORT || 9001);

const sock = net.createConnection(PORT, HOST, () => {
  console.log(`[test] connected to ${HOST}:${PORT}`);

  // 1) Register (10101): sex=0, name="MavisHero", career=2 (mage), playform="rgs-flash"
  const regPayload = Buffer.concat([beU8(0), beStr('MavisHero'), beI16(2), beStr('rgs-flash')]);
  sock.write(frame(10101, regPayload));
  console.log('[test] → 10101 register');

  // 2) After 500ms, enter server (10102)
  setTimeout(() => {
    const enterPayload = Buffer.concat([beU32(0x11111111), beStr('rgs-uat-1')]);
    sock.write(frame(10102, enterPayload));
    console.log('[test] → 10102 enter_server');
  }, 500);

  // 3) After 1500ms, close
  setTimeout(() => {
    console.log('[test] closing');
    sock.end();
  }, 1500);
});

let buf = Buffer.alloc(0);
let count = 0;
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
    if (f.cmd === 10101) {
      const r = parseSrv10101(f.payload);
      console.log(`[test] ← 10101: code=${r.code} msg="${r.msg}" rid=${r.rid} sid="${r.sid}" name="${r.name}" regTime=${r.regTime}`);
    } else if (f.cmd === 10102) {
      const r = parseSrv10102(f.payload);
      console.log(`[test] ← 10102: code=${r.code} msg="${r.msg}" ts=${r.ts} worldLev=${r.worldLev}`);
    } else {
      console.log(`[test] ← ${f.cmd}: payload_len=${f.payload.length}`);
    }
  }
});
sock.on('close', () => { console.log(`[test] closed (${count} frames)`); process.exit(0); });
sock.on('error', (e) => { console.error('[test] error:', e.message); process.exit(1); });
