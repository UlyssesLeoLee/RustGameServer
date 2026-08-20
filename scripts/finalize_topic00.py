#!/usr/bin/env python3
"""
为主题 00（治理）的 3 份 TST 补充 ADR 决策验证小节。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs\00-基准与治理')

ADR_SECTION = '''
## 6.6 ADR 决策验证（本主题）

本主题域涉及的 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备：

| ADR 编号 | 决定项摘要 | 实现位置 | 测试位置（本文档） | 守门位置 |
|---|---|---|---|---|
| RGS-ADR-0024 | 治理闭环的重新闭合（两级 ID 体系、三层位阶、两层验收） | RGS-DTL-009 §3 ID 登记表 + §5 治理 CI | §3.1 ID 登记表 + §3.3 治理 CI 校验 | git push 时 CI 阻断 |
| RGS-ADR-0025 | 运维负荷预算（OLU，NFR-OP-010 ≤ 2 SRE） | RGS-DTL-009 §4 OLU 预算台账 | §3.2 OLU 预算台账 | CI 静态检查 + 季度复盘 |

'''

def add_adr(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '## 6.6 ADR 决策验证' in c or '## 3.10' in c and 'ADR' in c:
        return False, '已存在'
    # 找 §6.5 NFR 覆盖索引
    m = re.search(r'(##\s*6\.5\s*NFR 覆盖索引)', c)
    if not m:
        return False, '无 §6.5 锚点'
    end_m = re.search(r'\n##\s', c[m.end():])
    if not end_m:
        return False, '无结束位置'
    end_idx = m.end() + end_m.start()
    c = c[:end_idx] + ADR_SECTION + c[end_idx:]
    tst_path.write_text(c, encoding='utf-8')
    return True, '已添加'

def main():
    added = 0
    for tf in sorted(DOCS_ROOT.glob('RGS-TST-*.md')):
        ok, msg = add_adr(tf)
        sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
        sys.stdout.flush()
        if ok: added += 1
    sys.stdout.write(f'\n=== 主题 00 ADR 验证补充：added={added} ===\n')

if __name__ == '__main__':
    main()
