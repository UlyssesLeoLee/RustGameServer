#!/usr/bin/env python3
"""
修复脚本第一轮的副作用：
1. 删除重复的"## 1.2 适用范围"小节
2. 添加目次小节
3. 添加"1.4 记述规则"小节
4. 添加强度用语
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

KISOKU_TEMPLATE = """
## 1.4 记述规则（記述規則 / Notation Rules）

### 1.4.1 强度用语（强度表現 / Strength of Expression）

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语：

| 中文表述 | 日文表述 | 英文 | 强度 | 含义 |
|---|---|---|---|---|
| **必须** | 必ず / 必須 | MUST | 强 | 必要条件。未满足则不予验收 |
| **应当** | すべき / 推奨 | SHOULD | 中 | 推荐条件。未满足时必须记录理由并取得批准 |
| **不得** | してはならない / 禁止 | MUST NOT | 强 | 禁止事项。违反即为设计缺陷 |
| **可以** | してもよい / 任意 | MAY | 弱 | 任意条件。是否实现不影响验收 |

### 1.4.2 优先级符号

| 符号 | 中文 | 日文 | 含义 |
|---|---|---|---|
| ◎ | 必须 | 必須 | 商用上线前必须实现 |
| ○ | 推荐 | 推奨 | 商用上线前应当实现 |
| △ | 任意 | 任意 | 上线后追加实现 |
| × | 范围外 | 範囲外 | 本次范围外 |

### 1.4.3 标识符体系

本文档遵循 RGS-REQ-001 §1.5.3 既定标识符体系：
- `RGS-TST-XX-NNN` 测试用例编号
- `RGS-{REQ|BAS|DTL}-NNN` 父文档编号
- `RGS-ADR-NNNN` 架构决策记录编号
- `NFR-<区分>-NNN` 非功能需求编号
- `AC-NNN` / `VF-NNN` / `FT-NNN` 验收/验证/故障注入编号
- `BZ-NNN` 业务规则编号
- `ST-NNN` 状态机编号

### 1.4.4 引用约定

- 全部引用以编号（如 `RGS-REQ-006`）而非文件路径
- 同一编号在本文档中首次出现时附全称，后续仅用编号

"""

MOKUJI_CONTENT = """
1. 前言
   1.1 目的
   1.2 适用范围
   1.3 关联文档
   1.4 记述规则
2. 测试策略
3. 测试用例
4. 追溯性矩阵
5. 测试执行计划
6. 通过判定基准
7. 风险与未决事项

注：本文档实际章节以文中二级标题为准。

"""

def dedupe_section(c, sec_title):
    """删除重复的章节，保留第一个"""
    pattern = re.escape(sec_title)
    matches = list(re.finditer(pattern, c))
    if len(matches) <= 1:
        return c
    # 保留第一个，删除第二个及之后
    # 从后往前删，避免索引偏移
    for m in reversed(matches[1:]):
        # 找该章节到下一个 ## 之间的范围
        next_h = c.find('\n## ', m.end())
        if next_h == -1:
            next_h = len(c)
        c = c[:m.start()] + c[next_h:]
    return c

def fix_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    changes = []

    # 1. 删重复的"## 1.2 适用范围"（保留第一个有内容的）
    dup_pattern_12 = '## 1.2 适用范围'
    matches = list(re.finditer(dup_pattern_12, c))
    if len(matches) > 1:
        # 保留第一个，删除其他
        for m in reversed(matches[1:]):
            # 找该节结束位置（下一个 ##）
            next_h = c.find('\n## ', m.end())
            if next_h == -1:
                next_h = len(c)
            # 检查该小节是否被填充了内容（不是空行）
            section = c[m.start():next_h]
            # 如果只有标题没有表格，删掉
            if '| 范畴 |' not in section and '## 1.1 目的' not in section:
                c = c[:m.start()] + c[next_h:]
                changes.append(f'删除重复章节: {dup_pattern_12}')
            else:
                # 如果是填充的章节，保留这个，删另一个
                # 找另一个
                if m != matches[0]:
                    c = c[:m.start()] + c[next_h:]
                    changes.append(f'删除重复章节: {dup_pattern_12}')

    # 2. 删除空"## 目录"标题（替换为"## 目次（目录）" + 内容）
    if '## 目录\n' in c or '## 目录 \n' in c or c.count('## 目录') > c.count('## 目次'):
        c = c.replace('## 目录', '## 目次（目录 / Table of Contents）', 1)
        # 在"## 目次"小节后插入内容
        idx = c.find('## 目次（目录 / Table of Contents）')
        next_sec = c.find('\n##', idx + 10)
        if next_sec == -1:
            next_sec = len(c)
        # 检查是否已有目录内容
        sec_content = c[idx:next_sec]
        if '1. 前言' not in sec_content and '## 1.1 目的' not in sec_content:
            c = c[:idx] + '## 目次（目录 / Table of Contents）\n\n' + MOKUJI_CONTENT.strip() + '\n' + c[next_sec:]
            changes.append('添加目次内容')

    # 3. 添加"1.4 记述规则"（如缺失）
    if '记述规则（記述規則' not in c and '記述規則' not in c:
        # 找"## 1.3 关联文档"小节结尾
        m = re.search(r'(##\s*1\.3\s*关联文档)', c)
        if m:
            next_sec = c.find('\n##', m.end())
            if next_sec == -1:
                next_sec = len(c)
            c = c[:next_sec] + KISOKU_TEMPLATE + c[next_sec:]
            changes.append('添加"1.4 记述规则"小节')

    if changes:
        tst_path.write_text(c, encoding='utf-8')
    return changes

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            changes = fix_doc(tf)
            if changes:
                sys.stdout.write(f'  [FIX] {tf.name}:\n')
                for ch in changes:
                    sys.stdout.write(f'       - {ch}\n')
                fixed += 1
            else:
                sys.stdout.write(f'  [SKIP] {tf.name}\n')
            sys.stdout.flush()
    sys.stdout.write(f'\n=== 修复完成：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
