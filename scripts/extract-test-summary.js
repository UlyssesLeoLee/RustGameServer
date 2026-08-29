// extract-test-summary.js
// 用途:从 cargo test 输出 log 提取测试统计, 写 JSON
// 用法: node extract-test-summary.js <log-file> <out-json>
const fs = require('fs');
const path = require('path');

const [, , logFile, outJson] = process.argv;
if (!logFile || !outJson) {
    console.error('usage: node extract-test-summary.js <log-file> <out-json>');
    process.exit(1);
}

const o = fs.readFileSync(logFile, 'utf8');

// 找所有 "test result: ok. N passed; M failed" 或 "test result: FAILED. M failed"
// 多 binary: 每 binary 一行
const lines = o.split('\n');
const results = [];
let currentCrate = '';

for (const ln of lines) {
    // 找 binary 标题: "Running tests/it_xxx.rs (target/debug/.../binary-name-XXXX.exe)"
    const runMatch = ln.match(/^\s*Running\s+(?:unittests\s+)?(?:(.+?)\s+)?\(?([^\s()]+)?$/);
    if (runMatch) {
        // 用 binary name 推测 crate
        const binary = runMatch[2] || runMatch[1];
        if (binary && binary.includes('/')) {
            const parts = binary.split('-');
            if (parts.length > 1) currentCrate = parts[0];
        }
    }
    // 找 test result 行
    const okMatch = ln.match(/test result: ok\.\s+(\d+)\s+passed(?:;\s+(\d+)\s+failed)?(?:;\s+(\d+)\s+ignored)?/);
    if (okMatch) {
        results.push({
            crate: currentCrate || 'unknown',
            passed: parseInt(okMatch[1]),
            failed: parseInt(okMatch[2] || '0'),
            ignored: parseInt(okMatch[3] || '0'),
            status: 'ok',
        });
        continue;
    }
    const failMatch = ln.match(/test result: FAILED\.\s+(\d+)\s+passed;\s+(\d+)\s+failed(?:;\s+(\d+)\s+ignored)?/);
    if (failMatch) {
        results.push({
            crate: currentCrate || 'unknown',
            passed: parseInt(failMatch[1]),
            failed: parseInt(failMatch[2]),
            ignored: parseInt(failMatch[3] || '0'),
            status: 'FAILED',
        });
    }
}

const totalPassed = results.reduce((s, r) => s + r.passed, 0);
const totalFailed = results.reduce((s, r) => s + r.failed, 0);
const totalIgnored = results.reduce((s, r) => s + r.ignored, 0);
const passRate = (totalPassed + totalFailed) > 0
    ? (totalPassed * 100 / (totalPassed + totalFailed)).toFixed(2)
    : '0.00';

const summary = {
    total: {
        passed: totalPassed,
        failed: totalFailed,
        ignored: totalIgnored,
        pass_rate_pct: parseFloat(passRate),
        binary_count: results.length,
    },
    per_binary: results,
    extracted_at: new Date().toISOString(),
};

fs.writeFileSync(outJson, JSON.stringify(summary, null, 2));
console.log(`  extracted: ${results.length} binaries, ${totalPassed} passed / ${totalFailed} failed / ${totalIgnored} ignored (${passRate}%)`);
