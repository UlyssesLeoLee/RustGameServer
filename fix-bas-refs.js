// 批量补 18 份 TST 文档头表 BAS 引用 + 追溯矩阵
const fs = require('fs');

const basMap = {
    'player':  'RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-013, RGS-BAS-022',
    'economy': 'RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-015, RGS-BAS-100',
    'social':  'RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-013, RGS-BAS-019',
    'match':   'RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-023, RGS-BAS-026',
    'admin':   'RGS-BAS-003, RGS-BAS-005, RGS-BAS-007, RGS-BAS-009, RGS-BAS-031',
    'cluster': 'RGS-BAS-009, RGS-BAS-012, RGS-BAS-022, RGS-BAS-031, RGS-BAS-037',
    'asset':   'RGS-BAS-009, RGS-BAS-022, RGS-BAS-027, RGS-BAS-036',
    'gm':      'RGS-BAS-003, RGS-BAS-009, RGS-BAS-021',
    'certgen': 'RGS-BAS-009',
};

const docs = [
    ['RGS-TST-UT-01_玩家域_单元测试设计书.md', 'player'],
    ['RGS-TST-UT-02_经济域_单元测试设计书.md', 'economy'],
    ['RGS-TST-UT-03_社交域_单元测试设计书.md', 'social'],
    ['RGS-TST-UT-04_对战域_单元测试设计书.md', 'match'],
    ['RGS-TST-UT-05_Admin域_单元测试设计书.md', 'admin'],
    ['RGS-TST-UT-06_ClusterOps域_单元测试设计书.md', 'cluster'],
    ['RGS-TST-UT-07_资产下载域_单元测试设计书.md', 'asset'],
    ['RGS-TST-UT-08_GM后台_单元测试设计书.md', 'gm'],
    ['RGS-TST-UT-09_工具集_单元测试设计书.md', 'certgen'],
    ['RGS-TST-IT-01_玩家域_集成测试设计书.md', 'player'],
    ['RGS-TST-IT-02_经济域_集成测试设计书.md', 'economy'],
    ['RGS-TST-IT-03_社交域_集成测试设计书.md', 'social'],
    ['RGS-TST-IT-04_对战域_集成测试设计书.md', 'match'],
    ['RGS-TST-IT-05_Admin域_集成测试设计书.md', 'admin'],
    ['RGS-TST-IT-06_ClusterOps域_集成测试设计书.md', 'cluster'],
    ['RGS-TST-IT-07_资产下载域_集成测试设计书.md', 'asset'],
    ['RGS-TST-IT-08_GM后台_集成测试设计书.md', 'gm'],
    ['RGS-TST-IT-09_工具集_集成测试设计书.md', 'certgen'],
];

for (const [fname, domain] of docs) {
    const p = `docs/00-基准与治理/${fname}`;
    if (!fs.existsSync(p)) {
        console.log(`[NOT FOUND] ${p}`);
        continue;
    }
    let o = fs.readFileSync(p, 'utf8');
    const basStr = basMap[domain];

    // 找"| 关联源代码文档 |" 或 "| 关联源代码 |" 行,在其下加"| 关联基本设计 |"
    const lines = o.split('\n');
    let insertAt = -1;
    for (let i = 0; i < lines.length; i++) {
        if (lines[i].includes('关联源代码文档') || lines[i].includes('关联源代码')) {
            insertAt = i + 1;
            break;
        }
    }
    if (insertAt > 0) {
        // 检查是否已有"关联基本设计"行
        const hasAlready = lines.some(l => l.includes('关联基本设计'));
        if (hasAlready) {
            console.log(`[SKIP] already has 关联基本设计: ${fname}`);
            continue;
        }
        lines.splice(insertAt, 0, `| 关联基本设计 | ${basStr} |`);
        o = lines.join('\n');
        fs.writeFileSync(p, o, 'utf8');
        console.log(`[OK] 头表已补 关联基本设计: ${fname}`);
    } else {
        console.log(`[SKIP] no 关联源代码/关联源代码文档 line: ${fname}`);
    }
}
