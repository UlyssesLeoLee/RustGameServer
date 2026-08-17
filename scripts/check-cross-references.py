#!/usr/bin/env python3
"""跨文档引用有效性检查（由 check-docs-consistency.sh 第5项调用）

背景：2026-08-17最终审核中，仅靠"结构性检查"（版本号同步、标题层级、链接有效性）
通过了全部检验的文档里，仍藏有实质性引用错误——例如 RGS-DTL-025 引用了
`accounts(account_id)` 这个从不存在的列名、RGS-DTL-003 引用了 RGS-DTL-025§7
但该文档只有§1〜6。这类错误穿透了当时的全部检查项，因此固化为常驻检查。

检查项：
  5a. 引用的 RGS-XXX-NNN 文档编号确实存在（RGS-REQ-0xx 这类模板占位符除外）
  5b. 形如 RGS-XXX-NNN§M 的跨文档章节引用，目标章节确实存在于目标文档
  5c. SQL 索引引用的列确实存在于对应 CREATE TABLE 中
  5d. 同一 protobuf message 内字段编号无重复
  5e. 跨文档 REFERENCES 的表与列在全仓库范围内确实存在（5c 只覆盖本文档内定义的表）

刻意不检查的：文档内部裸 §N 引用。中文技术写作中"细化RGS-BAS-002§4.2、§5.2、
§10.1"这类顿列式承前引用指向的是前文提到的目标文档而非本文档，机械判定误报率极高。
"""
import re
import sys
import glob
import os
import collections

# 尚未制定、仅在README§2"后续制定文档"表中预留编号的文档类别，引用它们不算死链
PLANNED_PREFIXES = ("RGS-IFS-", "RGS-DBS-", "RGS-TST-", "RGS-OPS-", "RGS-ADR-")

SQL_RESERVED = {"DESC", "ASC", "NULLS", "LAST", "FIRST", "WHERE", "AND", "OR", "NOT", "IS", "NULL"}


def load_docs():
    docs = {}
    for path in sorted(glob.glob("docs/**/RGS-*.md", recursive=True)):
        m = re.search(r"(RGS-[A-Z]+-\d+)", os.path.basename(path))
        if m:
            docs[m.group(1)] = open(path, encoding="utf-8").read()
    return docs


def top_level_sections(text):
    """文档的顶级章节号集合（# N. / ## N. 两种写法并存于本仓库）"""
    return set(re.findall(r"^#{1,2} (\d+)\.", text, re.M))


def sql_blocks(text):
    """只取 ```sql 围栏内的内容。行文中引述历史错误写法（如修订历史里"原写作
    REFERENCES accounts(account_id)"）不是活引用，不应被 SQL 类检查判为缺陷。"""
    return "\n".join(re.findall(r"```sql\n(.*?)```", text, re.S))


def parse_tables(text):
    """解析文本中的 CREATE TABLE，返回 表名 -> 列集合。

    逐行扫描而非单条正则：`) PARTITION BY RANGE (...)` 这类结尾使得
    非贪婪的 `.*?\\n\\)` 会越过本表继续吞掉下一张表的定义（本仓库
    transaction_ledger_template 之后的 matches 表就是这样被整张漏掉的）。
    以"行首为 )"作为表定义终止符，可靠得多。
    """
    def strip_noise(s):
        """去掉注释与字符串字面量，避免其中的括号干扰深度计数"""
        s = re.sub(r"--.*$", "", s)
        return re.sub(r"'[^']*'", "''", s)

    tables = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        m = re.match(r"CREATE TABLE (?:IF NOT EXISTS )?(\w+)\s*\(", lines[i])
        if not m:
            i += 1
            continue
        name, cols = m.group(1), set()
        head = strip_noise(lines[i])
        depth = head.count("(") - head.count(")")
        i += 1
        # depth<=0 表示单行建表（如 CREATE TABLE x (LIKE y INCLUDING ALL);），本行已闭合，
        # 不可继续向后吞行——否则会把后续整张表的定义吃掉（历史上 matches 表即因此被整张漏检）。
        while depth > 0 and i < len(lines):
            body = strip_noise(lines[i])
            c = re.match(r"\s{2,}(\w+)\s+[A-Z]", lines[i])
            if c and c.group(1).upper() not in {
                "CONSTRAINT", "PRIMARY", "UNIQUE", "CHECK", "FOREIGN", "EXCLUDE", "LIKE",
            }:
                cols.add(c.group(1))
            depth += body.count("(") - body.count(")")
            i += 1
        tables.setdefault(name, cols)
    return tables


def all_tables(docs):
    """全仓库 表名 -> (列集合, 定义所在文档)。同名表在多文档定义时以首次出现为准。"""
    registry = {}
    for did, text in docs.items():
        for name, cols in parse_tables(sql_blocks(text)).items():
            registry.setdefault(name, (cols, did))
    return registry


def check(docs):
    issues = []
    global_tables = all_tables(docs)

    # 5e 跨文档 REFERENCES：引用别的文档定义的表时，列名必须真实存在。
    # 这是引发2026-08-17审核的原始bug类型（RGS-DTL-025 引用 accounts(account_id)，
    # 而 accounts 表定义在 RGS-DTL-001 且并无 account_id 这一列），5c 只查本文档内定义的表，
    # 覆盖不到，故单列一项。
    for did, text in docs.items():
        for m in re.finditer(r"REFERENCES (\w+)\s*\((\w+)\)", sql_blocks(text)):
            table, col = m.group(1), m.group(2)
            if table not in global_tables:
                issues.append(f"{did}: REFERENCES 了全仓库均未定义的表 {table}")
                continue
            cols, owner = global_tables[table]
            if col not in cols:
                have = ", ".join(sorted(cols))
                issues.append(
                    f"{did}: REFERENCES {table}({col})，但 {table}（定义于 {owner}）无此列；现有列：{have}"
                )

    for did, text in docs.items():
        # 5a 文档编号死引用
        # 末尾 \b 用于排除 RGS-REQ-0xx / RGS-BAS-00n 这类模板占位符写法（数字后紧跟字母时不成立）
        for ref in sorted(set(re.findall(r"\bRGS-[A-Z]+-\d+\b", text))):
            if ref not in docs and not ref.startswith(PLANNED_PREFIXES):
                issues.append(f"{did}: 引用了不存在的文档 {ref}")

        # 5b 跨文档章节引用
        for ref, sec in sorted(set(re.findall(r"(RGS-(?:REQ|BAS|DTL)-\d+)\s*§\s*(\d+)", text))):
            if ref not in docs:
                continue
            secs = top_level_sections(docs[ref])
            if secs and sec not in secs:
                have = ",".join(sorted(secs, key=int))
                issues.append(f"{did}: 引用 {ref}§{sec}，但该文档只有 §{have}")

        # 5c SQL 索引列不存在于建表语句
        sql = sql_blocks(text)
        tables = parse_tables(sql)
        for m in re.finditer(r"CREATE (?:UNIQUE )?INDEX \w+\s*\n?\s*ON (\w+) \(([^)]*)\)", sql):
            table, colspec = m.group(1), m.group(2)
            if table not in tables:
                continue
            for col in re.findall(r"\w+", colspec):
                if col.upper() in SQL_RESERVED or col.isdigit():
                    continue
                if col not in tables[table]:
                    issues.append(f"{did}: 索引引用了 {table} 中不存在的列 {col}")

        # 5d protobuf 字段编号重复
        for m in re.finditer(r"message (\w+) \{(.*?)\n\}", text, re.S):
            nums = re.findall(r"=\s*(\d+)\s*;", m.group(2))
            dup = sorted({n for n, c in collections.Counter(nums).items() if c > 1}, key=int)
            if dup:
                issues.append(f"{did}: message {m.group(1)} 字段编号重复 {dup}")

    return issues


def main():
    if not os.path.isdir("docs"):
        print("  [FAIL] 未找到 docs/ 目录", file=sys.stderr)
        return 1
    issues = check(load_docs())
    if issues:
        for line in issues:
            print(f"  [FAIL] {line}")
        return 1
    print("  跨文档引用（文档编号/章节号/SQL列名/proto字段编号）全部有效")
    return 0


if __name__ == "__main__":
    sys.exit(main())
