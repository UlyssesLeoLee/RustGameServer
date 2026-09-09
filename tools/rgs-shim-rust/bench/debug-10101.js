// debug-10101.js
const net = require('net');
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
const sock = net.createConnection(9001, '127.0.0.1');
let buf = Buffer.alloc(0);
sock.on('connect', () => {
  const p1 = Buffer.concat([Buffer.from([1]), packString('FlowHero'), Buffer.from([0, 1]), packString('android')]);
  console.log('Sending 10101, payload length:', p1.length);
  sock.write(makeFrame(10101, p1));
});
sock.on('data', (chunk) => {
  buf = Buffer.concat([buf, chunk]);
  if (buf.length >= 6) {
    const totalLen = 4 + buf.readUInt32BE(0);
    if (buf.length >= totalLen) {
      const cmd = buf.readUInt16BE(4);
      const payload = buf.slice(6, totalLen);
      console.log('Response cmd:', cmd, 'payload_len:', payload.length);
      console.log('Response payload hex (first 40 bytes):', payload.slice(0, 40).toString('hex'));
      // 解析 10101 srv: code:u8 + msg:str + rid:u32 + srv_id:str + name:str + reg_time:u32
      let off = 0;
      const code = payload.readUInt8(off); off += 1;
      const msgLen = payload.readUInt32BE(off); off += 4;
      const msg = payload.toString('utf8', off, off + msgLen); off += msgLen;
      console.log('  code:', code, 'msg:', msg, 'nextOff:', off);
      const rid = payload.readUInt32BE(off); off += 4;
      console.log('  rid:', rid.toString(16), 'nextOff:', off);
      const srvLen = payload.readUInt32BE(off); off += 4;
      const srv = payload.toString('utf8', off, off + srvLen); off += srvLen;
      console.log('  srv_id:', srv, 'nextOff:', off);
      const nameLen = payload.readUInt32BE(off); off += 4;
      const name = payload.toString('utf8', off, off + nameLen); off += nameLen;
      console.log('  name:', name, 'nextOff:', off);
      const regTime = payload.readUInt32BE(off);
      console.log('  reg_time:', regTime);
      sock.end();
    }
  }
});
sock.on('error', (e) => console.error('error:', e.message));
sock.on('close', () => process.exit(0));
