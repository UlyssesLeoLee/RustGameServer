#!/usr/bin/env python3
"""
按日本 IPA 共通フレーム 2013 (SLCP-JCF2013) 标准，标准化所有 24 份 RGS-TST 测试设计书。
正文中文，IPA 专有术语保留中日双语。

IPA 共通フレーム要求的章节结构（测试文档）：
1. 表纸（文档编号、版本、父文档、依据标准、制定日、修订历史、审批栏）
2. 目次（目录）
3. 1. 前言（1.1 目的、1.2 适用范围、1.3 关联文档、1.4 记述规则）
4. 2. 测试策略（V 模型对应）
5. 3. 测试用例
6. 4. 追溯性矩阵
7. 5. 测试执行计划
8. 6. 通过判定基准
9. 7. 风险与未决事项
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 记述规则小节模板（中日双语，IPA 强度用语）
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

# 目次（目录）小节模板
MOKUJI_TEMPLATE_TEMPLATE = """
## 目次（目次 / Table of Contents）

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

注：本文档实际章节号以文中二级标题为准。

"""

def has_field(content, field):
    """检查文档元数据表是否含指定字段"""
    return re.search(rf'\|\s*{re.escape(field)}\s*\|', content) is not None

def normalize_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')

    changes = []

    # 1. 替换 "## 目录" → "## 目次（目录）"
    if '## 目录' in c and '## 目次' not in c:
        c = c.replace('## 目录', '## 目次（目录）', 1)
        changes.append('重命名"目录"为"目次"')

    # 2. 添加"目次"内容（在"## 目次（目录）"小节标题之后、下一节之前）
    if '## 目次（目录）' in c and '1.4 记述规则' not in c[:c.find('## 目次（目录）') + 1000]:
        # 在"## 目次（目录）"之后插入内容
        idx = c.find('## 目次（目录）')
        end_idx = c.find('\n##', idx + 10)
        if end_idx == -1:
            end_idx = c.find('\n---', idx + 10)
        if end_idx == -1:
            end_idx = idx + 100
        # 替换原有空白内容
        c = c[:idx] + '## 目次（目录 / Table of Contents）\n\n' + MOKUJI_TEMPLATE_TEMPLATE.lstrip('\n').lstrip('## 目次（目录）\n\n').strip() + '\n' + c[end_idx:]
        changes.append('填充目次内容')

    # 3. 添加"1.2 适用范围"小节（如果缺失）
    if not has_field(c, '适用范围'):
        # 在 "## 1. 前言" 之后插入
        idx = c.find('## 1. 前言')
        if idx == -1:
            idx = c.find('## 1.1')
        if idx > -1:
            next_sec = c.find('\n##', idx + 10)
            if next_sec == -1:
                next_sec = len(c)
            scope_sec = """

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

"""
            c = c[:next_sec] + scope_sec + c[next_sec:]
            changes.append('添加"1.2 适用范围"小节')

    # 4. 添加"1.4 记述规则"小节
    if '1.4 记述规则' not in c and '記述規則' not in c:
        # 在"## 1.3 关联文档"或"## 关联文档"之后插入
        idx = c.find('## 1.3 关联文档')
        if idx == -1:
            idx = c.find('## 关联文档')
        if idx > -1:
            next_sec = c.find('\n##', idx + 10)
            if next_sec == -1:
                next_sec = len(c)
            c = c[:next_sec] + KISOKU_TEMPLATE + c[next_sec:]
            changes.append('添加"1.4 记述规则"小节')

    # 5. 添加"依据标准"字段（如果元数据表缺失）
    if not has_field(c, '依据标准'):
        # 在"父文档"字段后插入
        m = re.search(r'(\| 父文档 \|.*?\n)', c)
        if m:
            ipa_standard = '| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』詳細設計工程 |'
            c = c[:m.end()] + ipa_standard + '\n' + c[m.end():]
            changes.append('添加"依据标准"字段')

    if changes:
        tst_path.write_text(c, encoding='utf-8')
    return changes

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        if topic_dir.name in ('00-基准与治理', '08-架构决策记录'):
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            changes = normalize_doc(tf)
            if changes:
                sys.stdout.write(f'  [OK] {tf.name}:\n')
                for ch in changes:
                    sys.stdout.write(f'       - {ch}\n')
                fixed += 1
            else:
                sys.stdout.write(f'  [SKIP] {tf.name}: 无需修改\n')
            sys.stdout.flush()
    sys.stdout.write(f'\n=== IPA 标准化完成：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
