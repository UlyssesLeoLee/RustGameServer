// 批量补缺失的"修订人"行
const fs = require('fs');

const fixList = [
    // 7 份 UT 域文档
    'docs/00-基准与治理/RGS-TST-UT-01_玩家域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-02_经济域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-03_社交域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-04_对战域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-05_Admin域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-06_ClusterOps域_单元测试设计书.md',
    'docs/00-基准与治理/RGS-TST-UT-07_资产下载域_单元测试设计书.md',
    // UT-09 工具集
    'docs/00-基准与治理/RGS-TST-UT-09_工具集_单元测试设计书.md',
    // OLD-DEBT
    'crates/cluster-ops/tests-disabled/OLD-DEBT.md',
];

const newReviser = '\n**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手\n';

for (const p of fixList) {
    if (!fs.existsSync(p)) { console.log('NOT FOUND:', p); continue; }
    let o = fs.readFileSync(p, 'utf8');
    if (o.includes('**修订人**')) {
        console.log('[SKIP] already has 修订人:', p);
        continue;
    }
    // 找到"**审批**"行后追加"**修订人**"
    // 中文逗号 '，' 不用作匹配标记
    const lines = o.split('\n');
    let insertAt = -1;
    for (let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith('**审批**') || lines[i].startsWith('**审批')) {
            insertAt = i + 1;
            break;
        }
    }
    if (insertAt < 0) {
        // 没有审批行,就在文件末尾追加
        o = o.trimEnd() + '\n\n' + newReviser.trim() + '\n';
    } else {
        lines.splice(insertAt, 0, '**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手');
        o = lines.join('\n');
    }
    fs.writeFileSync(p, o, 'utf8');
    console.log('[OK] 修订人已补:', p);
}
