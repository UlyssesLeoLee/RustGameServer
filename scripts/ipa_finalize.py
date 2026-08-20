#!/usr/bin/env python3
"""
彻底修复：每份文档按 IPA 共通フレーム标准章节结构重建。
- 删除从"## 1."到下一个二级标题之间的所有内容
- 重新插入标准章节：1.1 目的、1.2 适用范围、1.3 关联文档、1.4 记述规则、1.5 字段级映射说明、1.6 命名约定
- 保留 1.6 之后的所有内容（2. 测试策略 等）
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 保留"## 1. 前言"标题（不动），替换"## 1. 前言" 到 "## 2." 之间的全部
# 实际是：找 "## 1. 前言" 之后的第一个 "## 2." 位置

STANDARD_SECTION_1 = """## 1.1 目的（目的 / Purpose）

__PURPOSE_TEXT__

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

__RELATED_DOCS__

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

## 1.5 字段级映射说明

本版本（0.2）的核心升级是**字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + §X.Y + 表/图/字段"。

**映射规则**：
- 每个测试模块对应 1 个或多个父文档的物理/实现级章节
- 每条用例精确引用其父文档的具体字段（如 DDL 字段、gRPC 方法字段、状态机迁移名）
- 模块汇总表（§2.2）给出该文档验证的字段清单与覆盖率目标

**V 模型强化对应**：本文档对应该主题父基本设计书与详细设计书，构成"V 字"右侧的 TL-1/2/3 单元素验证。

## 1.6 命名约定（命名規約 / Naming Convention）

- 用例 ID：`TST-{UT|IT|ST}-XX-NNN`（XX 为主题编号 00-07）
- 试验级别标注：UT 无标注 / IT 用 [TL-2/3/4/5] / ST 用 [TL-6/7/8/E2E]
- 覆盖类型：N=正常 / A=异常 / B=边界 / P=属性不变条件 / S=状态机非法迁移
- 运行时机：`cargo test --workspace`（主干 CI 必跑，QA-006 ≤ 15 min 约束内）

"""

def fix_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')

    # 找"## 1. 前言"
    m = re.search(r'\n## 1\.\s*前言', c)
    if not m:
        return False, '无 ## 1. 前言'

    # 找该节结束（## 2. 第一个）
    end_m = re.search(r'\n## 2\.', c[m.end():])
    if not end_m:
        return False, '无 ## 2.'
    end_idx = m.end() + end_m.start()

    # 提取"## 1. 前言"和"## 2."之间的原内容
    old_section = c[m.end():end_idx]

    # 从原内容提取"## 1.1 目的"段落（保留目的说明）
    purpose_m = re.search(r'## 1\.1\s+目的[^\n]*\n+(.*?)(?=\n## 1\.[2-9]|\n## 2\.)', old_section, re.DOTALL)
    purpose_text = purpose_m.group(1).strip() if purpose_m else 'TL-1/2/3 单元/集成/系统试验层级，对应主题域内父文档。'

    # 从原内容提取"## 1.3 关联文档"表格
    rel_m = re.search(r'## 1\.3\s+关联文档[^\n]*\n+(.*?)(?=\n## 1\.[4-9]|\n## 2\.)', old_section, re.DOTALL)
    related_docs = rel_m.group(1).strip() if rel_m else '| 文档编号 | 文档名 | 与本文档关系 |\n|---|---|---|\n| （见各文档编号） | | |'

    # 重建标准 1.x 节
    new_section = STANDARD_SECTION_1.replace('__PURPOSE_TEXT__', purpose_text).replace('__RELATED_DOCS__', related_docs)

    # 替换
    c = c[:m.end()] + new_section + c[end_idx:]
    tst_path.write_text(c, encoding='utf-8')
    return True, f'已重写 §1.x ({len(new_section)} chars)'

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = fix_doc(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: fixed += 1
    sys.stdout.write(f'\n=== §1.x 标准化：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
