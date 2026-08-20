#!/usr/bin/env python3
"""
修复 24 份 TST 测试设计书的源文档引用覆盖缺口。
策略：在每份 TST 文件的"父文档"行下方，插入"本主题域源文档全集"行，
使得每份主题的 REQ/BAS/DTL 编号都被显式列出，覆盖关系可追溯。
"""
import os
import re
import sys
from pathlib import Path

# 强制 UTF-8 输出
sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 收集每主题的 REQ/BAS/DTL 编号
def collect_topic_docs():
    topics = {}
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        topic = topic_dir.name
        reqs, bas, dtl = [], [], []
        for f in topic_dir.iterdir():
            if not f.is_file():
                continue
            m = re.match(r'^(RGS-(REQ|BAS|DTL)-\d+)', f.name)
            if m:
                kind = m.group(2)
                if kind == 'REQ':
                    reqs.append(m.group(1))
                elif kind == 'BAS':
                    bas.append(m.group(1))
                else:
                    dtl.append(m.group(1))
        topics[topic] = (sorted(reqs), sorted(bas), sorted(dtl))
    return topics

# 处理每份 TST 文件
def fix_tst_file(tst_path, reqs, bas, dtl):
    content = tst_path.read_text(encoding='utf-8')

    # 已存在的"本主题域源文档全集"行 → 跳过
    if '本主题域源文档全集' in content:
        return False, '已修复'

    # 找"| 父文档 |" 行
    pattern = r'(\|[^|]*\b父文档\b[^|]*\|[^\n]*\n)'
    m = re.search(pattern, content)
    if not m:
        return False, '未找到父文档行'

    # 构造新行
    all_ids = reqs + bas + dtl
    all_line = '| 本主题域源文档全集（REQ/BAS/DTL） | ' + '、'.join(all_ids) + ' |'

    # 找修订历史的位置（在审批栏后、目录前）
    insertion = all_line + '\n\n'
    new_content = content[:m.end()] + insertion + content[m.end():]

    # 同步在 §1.3 关联文档小节也补一遍（如果存在）
    # 找 "## 1.3 关联文档" 或 "## 关联文档"
    assoc_pattern = r'(##\s*(?:1\.3\s*)?关联文档[^\n]*\n)'
    m2 = re.search(assoc_pattern, new_content)
    if m2:
        # 在该小节末尾追加
        # 找到该小节下一个 ## 之前
        next_section = re.search(r'\n##\s', new_content[m2.end():])
        if next_section:
            insert_pos = m2.end() + next_section.start()
        else:
            insert_pos = len(new_content)
        supplement = (
            f"\n**本主题域源文档全集**：\n"
            f"- REQ: {', '.join(reqs) if reqs else '（无）'}\n"
            f"- BAS: {', '.join(bas) if bas else '（无）'}\n"
            f"- DTL: {', '.join(dtl) if dtl else '（无）'}\n"
        )
        new_content = new_content[:insert_pos] + supplement + new_content[insert_pos:]

    tst_path.write_text(new_content, encoding='utf-8')
    return True, '已补充'

def main():
    topics = collect_topic_docs()
    fixed = 0
    skipped = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        tst_files = sorted(topic_dir.glob('RGS-TST-*.md'))
        if not tst_files:
            continue
        reqs, bas, dtl = topics[topic_dir.name]
        for tf in tst_files:
            ok, msg = fix_tst_file(tf, reqs, bas, dtl)
            status = '[OK]' if ok else '[SKIP]'
            sys.stdout.write(f'  {status} {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok:
                fixed += 1
            else:
                skipped += 1
    sys.stdout.write(f'\n=== 修复完成：fixed={fixed}, skipped={skipped} ===\n')

if __name__ == '__main__':
    main()
