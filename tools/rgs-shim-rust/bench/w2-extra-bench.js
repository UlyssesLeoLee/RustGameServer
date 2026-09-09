// w2 续做性能测试: 50 rps >= 100 (per PHASE_4_WORKER_BRIEF.md §5 DoD)
// 抽样测试 23800/24000/24200/24800 + 福利高频 cmd
// per 2026-09-09 20:30 JST Mavis 派工

const net = require('net');

const HOST = '127.0.0.1';
const PORT = 9002;
const CONCURRENCY = 50;
const TOTAL_REQUESTS = 200;  // 50 rps target * 4 sec

function makeFrame(cmd, payload) {
  const p = payload || Buffer.alloc(0);
  const len = 2 + p.length;
  const buf = Buffer.alloc(6 + p.length);
  buf.writeUInt32BE(len, 0);
  buf.writeUInt16BE(cmd, 4);
  if (p.length > 0) p.copy(buf, 6);
  return buf;
}

function sendFrame(cmd, payload, timeoutMs = 5000) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection(PORT, HOST);
    let buf = Buffer.alloc(0);
    let timer;
    sock.on('connect', () => sock.write(makeFrame(cmd, payload)));
    sock.on('data', (chunk) => {
      buf = Buffer.concat([buf, chunk]);
      if (buf.length >= 6) {
        const totalLen = 4 + buf.readUInt32BE(0);
        if (buf.length >= totalLen) {
          clearTimeout(timer);
          sock.end();
          const payloadLen = buf.readUInt32BE(0) - 2;
          resolve({
            len: buf.readUInt32BE(0),
            cmd: buf.readUInt16BE(4),
            payload: buf.slice(6, 6 + payloadLen),
          });
        }
      }
    });
    sock.on('error', (e) => { clearTimeout(timer); reject(e); });
    timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, timeoutMs);
  });
}

async function runWorker(workerId, totalReqs, cmds) {
  let okCount = 0, errCount = 0;
  const perWorker = Math.floor(totalReqs / CONCURRENCY);
  for (let i = 0; i < perWorker; i++) {
    const cmdIdx = (workerId * perWorker + i) % cmds.length;
    const cmd = cmds[cmdIdx];
    try {
      const r = await sendFrame(cmd, Buffer.alloc(0));
      if (r.cmd === cmd && r.payload.length >= 0) {
        okCount++;
      } else {
        errCount++;
      }
    } catch (e) {
      errCount++;
    }
  }
  return { okCount, errCount };
}

async function main() {
  // 抽样 cmd (覆盖 w2 续做 110 welfare + 25 其它)
  const cmds = [
    23800, 23801, 23802, 23803, 23804, 23805, 23806, 23807, 23808, 23809, 23810, 23811, 23812,
    23900, 23901, 23902, 23903, 23904, 23905, 23906, 23907, 23908, 23909, 23910, 23911,
    24000, 24001, 24005, 24010, 24011, 24013, 24018, 24020,
    24100, 24101, 24103, 24104, 24122, 24132,
    24202, 24221, 24300, 24301, 24403, 24501, 24603, 24701, 24807, 24818,
  ];

  console.log(`w2 续做性能测试: CONCURRENCY=${CONCURRENCY}, TOTAL=${TOTAL_REQUESTS}, cmds=${cmds.length}`);
  const startTime = Date.now();

  const workers = [];
  for (let w = 0; w < CONCURRENCY; w++) {
    workers.push(runWorker(w, TOTAL_REQUESTS, cmds));
  }
  const results = await Promise.all(workers);

  const endTime = Date.now();
  const elapsed = (endTime - startTime) / 1000;
  const totalOk = results.reduce((s, r) => s + r.okCount, 0);
  const totalErr = results.reduce((s, r) => s + r.errCount, 0);
  const rps = totalOk / elapsed;
  const targetRps = 100;

  console.log(`\n=== Result ===`);
  console.log(`  Elapsed: ${elapsed.toFixed(2)}s`);
  console.log(`  Total OK: ${totalOk}`);
  console.log(`  Total ERR: ${totalErr}`);
  console.log(`  RPS: ${rps.toFixed(2)} (target >= ${targetRps})`);
  console.log(`  ${rps >= targetRps ? '✓ PASS' : '✗ FAIL'}`);

  process.exit(rps >= targetRps ? 0 : 1);
}

main().catch((e) => {
  console.error('fatal:', e);
  process.exit(1);
});
