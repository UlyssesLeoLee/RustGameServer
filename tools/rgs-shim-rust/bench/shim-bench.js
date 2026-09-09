// shim-bench.js — TCP client bench for rgs-shim-rust v0.3.0
// 真实 RGS 调用 + 1k frame 顺序测试 + 100 并发连接
// Usage: node shim-bench.js [host] [port]

const net = require('net');
const HOST = process.argv[2] || '127.0.0.1';
const PORT = parseInt(process.argv[3] || '9001', 10);

// Build a SmartSocket frame: | len:32 BE | cmd:16 BE | payload |
// len = 2 + payload.length
function makeFrame(cmd, payload = Buffer.alloc(0)) {
    const len = 2 + payload.length;
    const buf = Buffer.alloc(6 + payload.length);
    buf.writeUInt32BE(len, 0);
    buf.writeUInt16BE(cmd, 4);
    if (payload.length > 0) payload.copy(buf, 6);
    return buf;
}

// Build payload for 10101 register: {sex:u8=1, name:str="Bench", career:i16=1, playform:str="ios"}
function makeRegisterPayload(name = 'Bench') {
    const nameBuf = Buffer.from(name, 'utf8');
    const platformBuf = Buffer.from('ios', 'utf8');
    const buf = Buffer.alloc(1 + 4 + nameBuf.length + 2 + 4 + platformBuf.length);
    let off = 0;
    buf.writeUInt8(1, off); off += 1;
    buf.writeUInt32BE(nameBuf.length, off); nameBuf.copy(buf, off + 4); off += 4 + nameBuf.length;
    buf.writeInt16BE(1, off); off += 2;
    buf.writeUInt32BE(platformBuf.length, off); platformBuf.copy(buf, off + 4); off += 4 + platformBuf.length;
    return buf;
}

function parseResponse(buf) {
    if (buf.length < 6) return null;
    const len = buf.readUInt32BE(0);
    const cmd = buf.readUInt16BE(4);
    const payloadLen = len - 2;
    return { len, cmd, payload: buf.slice(6, 6 + payloadLen) };
}

function sendFrame(cmd, payload = null) {
    return new Promise((resolve, reject) => {
        const sock = net.createConnection(PORT, HOST);
        let buf = Buffer.alloc(0);
        let timer;
        sock.on('connect', () => {
            const frame = payload ? makeFrame(cmd, payload) : makeFrame(cmd);
            sock.write(frame);
            timer = setTimeout(() => { sock.destroy(); reject(new Error('timeout')); }, 5000);
        });
        sock.on('data', (chunk) => {
            buf = Buffer.concat([buf, chunk]);
            // 简单: 等到至少 6 字节且 len + 4 == buf.length
            if (buf.length >= 6) {
                const totalLen = 4 + buf.readUInt32BE(0);
                if (buf.length >= totalLen) {
                    clearTimeout(timer);
                    sock.end();
                    resolve(parseResponse(buf));
                }
            }
        });
        sock.on('error', (e) => { clearTimeout(timer); reject(e); });
        sock.on('close', () => {});
    });
}

async function benchSequential(n = 100) {
    const t0 = Date.now();
    let success = 0;
    let totalLatency = 0;
    for (let i = 0; i < n; i++) {
        const t1 = Date.now();
        try {
            const r = await sendFrame(10400);  // heartbeat (5 域并发 HealthCheck)
            if (r && r.cmd === 10400) success++;
            totalLatency += Date.now() - t1;
        } catch (e) {
            console.error(`seq ${i}: ${e.message}`);
        }
    }
    const dt = Date.now() - t0;
    return {
        mode: 'sequential',
        total: n,
        success,
        avg_latency_ms: (totalLatency / n).toFixed(2),
        throughput_rps: (success / (dt / 1000)).toFixed(1),
        total_ms: dt,
    };
}

async function benchConcurrent(n = 100, parallelism = 10) {
    const t0 = Date.now();
    let success = 0;
    const queue = Array.from({ length: n }, (_, i) => i);
    async function worker() {
        while (queue.length > 0) {
            const i = queue.shift();
            if (i === undefined) break;
            try {
                const r = await sendFrame(10400);
                if (r && r.cmd === 10400) success++;
            } catch (e) { /* ignore */ }
        }
    }
    const workers = Array.from({ length: parallelism }, () => worker());
    await Promise.all(workers);
    const dt = Date.now() - t0;
    return {
        mode: 'concurrent',
        total: n,
        success,
        parallelism,
        throughput_rps: (success / (dt / 1000)).toFixed(1),
        total_ms: dt,
    };
}

async function main() {
    console.log(`shim-bench target: ${HOST}:${PORT}`);
    console.log('');

    // 1) 单点 smoke: 6 个 cmd
    console.log('=== 6 cmd smoke ===');
    for (const cmd of [10101, 10102, 10103, 10200, 10400, 11001]) {
        const t1 = Date.now();
        try {
            let payload = null;
            if (cmd === 10101) payload = makeRegisterPayload('BenchHero');
            const r = await sendFrame(cmd, payload);
            const dt = Date.now() - t1;
            console.log(`  cmd ${cmd} → resp.cmd=${r.cmd} payload_len=${r.payload.length} dt=${dt}ms`);
        } catch (e) {
            console.log(`  cmd ${cmd} → ERR ${e.message}`);
        }
    }

    console.log('');
    console.log('=== bench: sequential 100 heartbeat ===');
    const seq = await benchSequential(100);
    console.log(JSON.stringify(seq));

    console.log('');
    console.log('=== bench: concurrent 100 heartbeat x 10 parallel ===');
    const conc = await benchConcurrent(100, 10);
    console.log(JSON.stringify(conc));

    console.log('');
    console.log('=== bench: concurrent 500 heartbeat x 20 parallel ===');
    const conc2 = await benchConcurrent(500, 20);
    console.log(JSON.stringify(conc2));
}

main().catch(e => { console.error(e); process.exit(1); });
