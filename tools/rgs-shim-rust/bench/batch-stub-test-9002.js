// batch-stub-test.js — 批量测所有 766 send cmd (per proto_mate.js)
// v0.4.0 100% 协议点接受验证 (per 2026-09-09 16:25 JST Mavis 派工)
const net = require('net');
const fs = require('fs');

const HOST = '127.0.0.1';
const PORT = 9002;
const PROTO_MATE = 'E:/BaiduNetdiskDownload/闪烁之光/server分析/zsyz_client_h5/temp/quick-scripts/src/assets/Scripts/net/proto_mate.js';

// 帧构造
function makeFrame(cmd, payload = null) {
  const p = payload || Buffer.alloc(0);
  const len = 2 + p.length;
  const buf = Buffer.alloc(6 + p.length);
  buf.writeUInt32BE(len, 0);
  buf.writeUInt16BE(cmd, 4);
  if (p.length > 0) p.copy(buf, 6);
  return buf;
}
function sendFrame(cmd, payload) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection(PORT, HOST);
    let buf = Buffer.alloc(0);
    let timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, 3000);
    sock.on('connect', () => sock.write(makeFrame(cmd, payload)));
    sock.on('data', (chunk) => {
      buf = Buffer.concat([buf, chunk]);
      if (buf.length >= 6) {
        const totalLen = 4 + buf.readUInt32BE(0);
        if (buf.length >= totalLen) {
          clearTimeout(timer);
          sock.end();
          const dv = new DataView(buf.buffer, buf.byteOffset);
          resolve({ len: dv.getUint32(0, false), cmd: dv.getUint16(4, false), payload: buf.slice(6, totalLen) });
        }
      }
    });
    sock.on('error', (e) => { clearTimeout(timer); reject(e); });
  });
}

// 提取 send cmd 列表
const content = fs.readFileSync(PROTO_MATE, 'utf8');
const sendMatch = content.match(/module\.exports\.send\s*=\s*\{([\s\S]*?)\n\};/);
function extractCmds(text) {
  const cmds = new Set();
  const re = /^\s*(\d{5})\s*:/gm;
  let m;
  while ((m = re.exec(text)) !== null) cmds.add(parseInt(m[1]));
  return [...cmds].sort((a, b) => a - b);
}
const sendCmds = sendMatch ? extractCmds(sendMatch[1]) : [];

// 提取字段
function extractFields(text) {
  const result = {};
  const re = /(\d{5})\s*:\s*\[([\s\S]*?)\n\s*\]/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    const cmd = m[1];
    const fieldsBlock = m[2];
    const fields = [];
    const fre = /\{\s*s:\s*['"](\w+)['"]\s*,\s*t:\s*(\d+)\s*\}/g;
    let fm;
    while ((fm = fre.exec(fieldsBlock)) !== null) {
      fields.push({ name: fm[1], type: parseInt(fm[2]) });
    }
    result[parseInt(cmd)] = fields;
  }
  return result;
}
const sendFields = sendMatch ? extractFields(sendMatch[1]) : {};

// 类型 → 构造
function packField(type) {
  // 0=unknown, 1=bool, 2=u8, 3=i16, 4=i16, 5=i32, 6=u32, 7=string
  switch (type) {
    case 1: return Buffer.from([1]);
    case 2: return Buffer.from([0]);
    case 3: return Buffer.from([0, 0]);
    case 4: return Buffer.from([0, 0]);
    case 5: return Buffer.from([0, 0, 0, 0]);
    case 6: return Buffer.from([0, 0, 0, 0]);
    case 7: {
      const s = Buffer.from('test', 'utf8');
      return Buffer.concat([Buffer.from([0, 0, 0, s.length]), s]);
    }
    default: return Buffer.alloc(0);
  }
}
function buildPayload(fields) {
  return Buffer.concat(fields.map(f => packField(f.type)));
}

// 批量测
async function main() {
  console.log(`batch-stub-test: ${sendCmds.length} send cmd (per H5 proto_mate.js)`);
  console.log(`target: ${HOST}:${PORT} (rgs-shim-rust v0.4.0)`);
  console.log('');

  // 真 handler (10 cmd), 用 0 payload
  const realHandlers = [10101, 10102, 10103, 10200, 10215, 10300, 10301, 10302, 10309, 10315, 10400, 11001];
  // 构造 payload (按字段类型)
  const tasks = sendCmds.map(cmd => {
    const fields = sendFields[cmd] || [];
    const payload = buildPayload(fields);
    return { cmd, payload };
  });

  let pass = 0, fail = 0, total = tasks.length;
  const failCmds = [];
  const t0 = Date.now();
  // 并发 50 测
  const concurrency = 50;
  for (let i = 0; i < tasks.length; i += concurrency) {
    const batch = tasks.slice(i, i + concurrency);
    const results = await Promise.allSettled(batch.map(t => sendFrame(t.cmd, t.payload)));
    for (let j = 0; j < batch.length; j++) {
      const r = results[j];
      if (r.status === 'fulfilled' && r.value.cmd === batch[j].cmd) {
        pass++;
      } else {
        fail++;
        failCmds.push(batch[j].cmd);
      }
    }
  }
  const dt = Date.now() - t0;
  console.log('');
  console.log(`合计: ${pass}/${total} 通过, ${fail} 失败, ${dt}ms (${(pass/(dt/1000)).toFixed(0)} rps)`);
  if (fail > 0) {
    console.log('失败 cmd:', failCmds.slice(0, 20).join(', '));
  }
  process.exit(fail > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(1); });
