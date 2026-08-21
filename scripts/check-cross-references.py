#!/usr/bin/env python3
"""cypher
CREATE
  (file:File {name: "check-cross-references.py", type: "file", language: "python"}),
  (load_registry:Function {name: "load_allowed_missing_ids", type: "function", signature: "() -> set[str] | None"}),
  (document:Class {name: "Document", type: "class", signature: "Document(document_id: str, path: str, text: str)"}),
  (extract_id:Function {name: "document_id_from_header", type: "function", signature: "(path: str, content: str) -> str"}),
  (load_docs:Function {name: "load_docs", type: "function", signature: "() -> list[Document]"}),
  (sections:Function {name: "top_level_sections", type: "function", signature: "(text: str) -> set[str]"}),
  (section_sort:Function {name: "section_sort_key", type: "function", signature: "(section: str) -> tuple[int, ...]"}),
  (sql:Function {name: "sql_blocks", type: "function", signature: "(text: str) -> str"}),
  (tables:Function {name: "parse_tables", type: "function", signature: "(text: str) -> dict"}),
  (all_tables:Function {name: "all_tables", type: "function", signature: "(docs: list[Document]) -> dict"}),
  (check:Function {name: "check", type: "function", signature: "(docs: list[Document], allowed_missing_ids: set[str]) -> list[str]"}),
  (main:Function {name: "main", type: "function", signature: "() -> int"}),
  (registry_path:Variable {name: "REGISTRY_PATH", type: "variable"}),
  (document_id_pattern:Variable {name: "DOCUMENT_ID_PATTERN", type: "variable"}),
  (filename_document_id_pattern:Variable {name: "FILENAME_DOCUMENT_ID_PATTERN", type: "variable"}),
  (document_id_field_pattern:Variable {name: "DOCUMENT_ID_FIELD_PATTERN", type: "variable"}),
  (sql_reserved:Variable {name: "SQL_RESERVED", type: "variable"}),
  (file)-[:CONTAINS]->(load_registry),
  (file)-[:CONTAINS]->(document),
  (file)-[:CONTAINS]->(extract_id),
  (file)-[:CONTAINS]->(load_docs),
  (file)-[:CONTAINS]->(sections),
  (file)-[:CONTAINS]->(section_sort),
  (file)-[:CONTAINS]->(sql),
  (file)-[:CONTAINS]->(tables),
  (file)-[:CONTAINS]->(all_tables),
  (file)-[:CONTAINS]->(check),
  (file)-[:CONTAINS]->(main),
  (file)-[:CONTAINS]->(registry_path),
  (file)-[:CONTAINS]->(document_id_pattern),
  (file)-[:CONTAINS]->(filename_document_id_pattern),
  (file)-[:CONTAINS]->(document_id_field_pattern),
  (file)-[:CONTAINS]->(sql_reserved),
  (load_registry)-[:USES]->(registry_path),
  (extract_id)-[:USES]->(filename_document_id_pattern),
  (extract_id)-[:USES]->(document_id_field_pattern),
  (load_docs)-[:CALLS]->(extract_id),
  (check)-[:USES]->(document_id_pattern),
  (check)-[:USES]->(sql_reserved),
  (all_tables)-[:CALLS]->(sql),
  (all_tables)-[:CALLS]->(tables),
  (check)-[:CALLS]->(all_tables),
  (check)-[:CALLS]->(sql),
  (check)-[:CALLS]->(tables),
  (check)-[:CALLS]->(sections),
  (check)-[:CALLS]->(section_sort),
  (main)-[:CALLS]->(load_registry),
  (main)-[:CALLS]->(load_docs),
  (main)-[:CALLS]->(check);
"""
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
import tomllib
from dataclasses import dataclass

REGISTRY_PATH = "docs/document-registry.toml"
DOCUMENT_ID_PATTERN = r"RGS-[A-Z]+(?:-[A-Z]+)*-\d+(?:-ADD\d+)?(?![A-Za-z0-9-])"
FILENAME_DOCUMENT_ID_PATTERN = r"RGS-[A-Z]+(?:-[A-Z]+)*-\d+(?![A-Za-z0-9-])"
DOCUMENT_ID_FIELD_PATTERN = re.compile(
    r"^\|\s*(?:文档|决策|ADR)(?:编号|ID)\s*\|\s*(.*?)\s*\|",
    re.M | re.I,
)

SQL_RESERVED = {"DESC", "ASC", "NULLS", "LAST", "FIRST", "WHERE", "AND", "OR", "NOT", "IS", "NULL"}


def load_allowed_missing_ids():
    """读取可受控引用的计划文档；前缀通配不再掩盖拼写或编号错误。"""
    try:
        with open(REGISTRY_PATH, "rb") as registry_file:
            registry = tomllib.load(registry_file)
    except (FileNotFoundError, tomllib.TOMLDecodeError) as exc:
        print(f"  [FAIL] 无法读取文档登记册 {REGISTRY_PATH}: {exc}", file=sys.stderr)
        return None
    return {
        entry["id"]
        for entry in registry.get("planned_document", [])
        if entry.get("allow_reference") is True and entry.get("status") == "planned"
    }


@dataclass(frozen=True)
class Document:
    document_id: str
    path: str
    text: str


def document_id_from_header(path: str, content: str) -> str:
    """以文件名基准ID核对文档头，并保留头中声明的 -ADDn 补遗后缀。"""
    filename_match = re.search(FILENAME_DOCUMENT_ID_PATTERN, os.path.basename(path))
    if not filename_match:
        raise ValueError(f"{path}: 文件名缺少可解析的正式文档编号")
    filename_id = filename_match.group(0)
    field_match = DOCUMENT_ID_FIELD_PATTERN.search(content[:1600])
    if not field_match:
        raise ValueError(f"{path}: 文档头缺少可解析的正式文档/决策编号字段")
    field = field_match.group(1)
    declared_match = re.search(DOCUMENT_ID_PATTERN, field)
    document_id = declared_match.group(0) if declared_match else filename_id
    if re.sub(r"-ADD\d+$", "", document_id) != filename_id:
        raise ValueError(f"{path}: 文件名编号 {filename_id} 与文档头编号 {field} 不一致")
    if document_id == filename_id and filename_id.removeprefix("RGS-") not in field:
        raise ValueError(f"{path}: 文档头编号 {field} 与文件名编号 {filename_id} 不一致")
    return document_id


def load_docs() -> list[Document]:
    docs: list[Document] = []
    for path in sorted(glob.glob("docs/**/RGS-*.md", recursive=True)):
        text = open(path, encoding="utf-8").read()
        docs.append(Document(document_id_from_header(path, text), path, text))
    return docs


def top_level_sections(text):
    """文档中所有 Markdown 标题的章节号（含 §0、二级与三级章节）。"""
    return set(re.findall(r"^#{1,6}\s+(?:§\s*)?(\d+(?:\.\d+)*)\.?\s", text, re.M))


def section_sort_key(section: str) -> tuple[int, ...]:
    return tuple(int(part) for part in section.split("."))


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


def all_tables(docs: list[Document]):
    """全仓库 表名 -> (列集合, 定义所在文档)。同名表在多文档定义时以首次出现为准。"""
    registry = {}
    for doc in docs:
        for name, cols in parse_tables(sql_blocks(doc.text)).items():
            registry.setdefault(name, (cols, doc.path))
    return registry


def check(docs: list[Document], allowed_missing_ids: set[str]) -> list[str]:
    issues = []
    global_tables = all_tables(docs)
    docs_by_id: dict[str, list[Document]] = collections.defaultdict(list)
    for doc in docs:
        docs_by_id[doc.document_id].append(doc)

    # 5e 跨文档 REFERENCES：引用别的文档定义的表时，列名必须真实存在。
    # 这是引发2026-08-17审核的原始bug类型（RGS-DTL-025 引用 accounts(account_id)，
    # 而 accounts 表定义在 RGS-DTL-001 且并无 account_id 这一列），5c 只查本文档内定义的表，
    # 覆盖不到，故单列一项。
    for doc in docs:
        for m in re.finditer(r"REFERENCES (\w+)\s*\((\w+)\)", sql_blocks(doc.text)):
            table, col = m.group(1), m.group(2)
            if table not in global_tables:
                issues.append(f"{doc.path}: REFERENCES 了全仓库均未定义的表 {table}")
                continue
            cols, owner = global_tables[table]
            if col not in cols:
                have = ", ".join(sorted(cols))
                issues.append(
                    f"{doc.path}: REFERENCES {table}({col})，但 {table}（定义于 {owner}）无此列；现有列：{have}"
                )

    for doc in docs:
        did, text = doc.document_id, doc.text
        # 5a 文档编号死引用
        # 负向前瞻排除 RGS-REQ-0xx / RGS-BAS-00n 等模板占位符。
        for ref in sorted(set(re.findall(DOCUMENT_ID_PATTERN, text))):
            if ref not in docs_by_id and ref not in allowed_missing_ids:
                issues.append(f"{doc.path}: 引用了不存在的文档 {ref}")

        # 5b 跨文档章节引用
        for ref, sec in sorted(set(re.findall(rf"({DOCUMENT_ID_PATTERN})\s*§\s*(\d+(?:\.\d+)*)", text))):
            targets = docs_by_id.get(ref, [])
            if not targets:
                continue
            secs = set().union(*(top_level_sections(target.text) for target in targets))
            if secs and sec not in secs:
                have = ",".join(sorted(secs, key=section_sort_key))
                issues.append(f"{doc.path}: 引用 {ref}§{sec}，但该文档只有 §{have}")

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
                    issues.append(f"{doc.path}: 索引引用了 {table} 中不存在的列 {col}")

        # 5d protobuf 字段编号重复
        for m in re.finditer(r"message (\w+) \{(.*?)\n\}", text, re.S):
            nums = re.findall(r"=\s*(\d+)\s*;", m.group(2))
            dup = sorted({n for n, c in collections.Counter(nums).items() if c > 1}, key=int)
            if dup:
                issues.append(f"{doc.path}: message {m.group(1)} 字段编号重复 {dup}")

    return issues


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    if not os.path.isdir("docs"):
        print("  [FAIL] 未找到 docs/ 目录", file=sys.stderr)
        return 1
    allowed_missing_ids = load_allowed_missing_ids()
    if allowed_missing_ids is None:
        return 1
    try:
        docs = load_docs()
    except ValueError as exc:
        print(f"  [FAIL] {exc}", file=sys.stderr)
        return 1
    issues = check(docs, allowed_missing_ids)
    if issues:
        for line in issues:
            print(f"  [FAIL] {line}")
        return 1
    print("  跨文档引用（文档编号/章节号/SQL列名/proto字段编号）全部有效")
    return 0


if __name__ == "__main__":
    sys.exit(main())
