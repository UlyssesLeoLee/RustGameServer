// extract-coverage.js
// 用途:从 LCOV 解析 workspace 覆盖率, 写 JSON
// 用法: node extract-coverage.js <lcov-file> <out-json>
const fs = require('fs');

const [, , lcovFile, outJson] = process.argv;
if (!lcovFile || !outJson) {
    console.error('usage: node extract-coverage.js <lcov-file> <out-json>');
    process.exit(1);
}

const o = fs.readFileSync(lcovFile, 'utf8');
const lines = o.split('\n');

// LCOV 格式: SF:<file>  DA:<line>,<count>  end_of_record
const fileStats = {}; // file -> { lines, hit }
let currentFile = null;

for (const ln of lines) {
    if (ln.startsWith('SF:')) {
        currentFile = ln.slice(3);
        fileStats[currentFile] = { lines: 0, hit: 0 };
    } else if (ln.startsWith('DA:')) {
        const m = ln.match(/^DA:(\d+),(\d+)$/);
        if (m && currentFile) {
            fileStats[currentFile].lines += 1;
            if (parseInt(m[2]) > 0) {
                fileStats[currentFile].hit += 1;
            }
        }
    } else if (ln === 'end_of_record') {
        currentFile = null;
    }
}

// 算 per-crate
const crateStats = {};
for (const [file, stat] of Object.entries(fileStats)) {
    // file path: crates/<crate>/...
    const m = file.match(/^crates\/([^/]+)\//);
    if (m) {
        const crate = m[1];
        if (!crateStats[crate]) crateStats[crate] = { lines: 0, hit: 0, files: 0 };
        crateStats[crate].lines += stat.lines;
        crateStats[crate].hit += stat.hit;
        crateStats[crate].files += 1;
    }
}

const totalLines = Object.values(fileStats).reduce((s, x) => s + x.lines, 0);
const totalHit = Object.values(fileStats).reduce((s, x) => s + x.hit, 0);
const coveragePct = totalLines > 0 ? (totalHit * 100 / totalLines).toFixed(2) : '0.00';

const perCrate = {};
for (const [c, s] of Object.entries(crateStats)) {
    perCrate[c] = {
        lines: s.lines,
        hit: s.hit,
        coverage_pct: s.lines > 0 ? parseFloat((s.hit * 100 / s.lines).toFixed(2)) : 0,
        files: s.files,
    };
}

const summary = {
    workspace: {
        total_lines: totalLines,
        hit_lines: totalHit,
        coverage_pct: parseFloat(coveragePct),
        file_count: Object.keys(fileStats).length,
    },
    per_crate: perCrate,
    extracted_at: new Date().toISOString(),
};

fs.writeFileSync(outJson, JSON.stringify(summary, null, 2));
console.log(`  workspace coverage: ${coveragePct}% (${totalHit}/${totalLines} lines, ${Object.keys(fileStats).length} files)`);
