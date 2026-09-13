#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""RGS 设计链对账脚本（REQ → BAS → DTL → SPEC）

依据 RGS-PLAN-003 §0「调查方法」：
    文件名编号配对是错的 —— RGS-REQ-NNN 与 RGS-BAS-NNN 的 NNN 不一一对应。
    正确配对方法：读文档头部「父文档」行，反向索引到上位文档。

本脚本把该方法机械化，并把「解析失败」与「真实缺口」分成两个桶输出，
避免把解析失败误报为设计缺失（RGS-PLAN-003 §2.6 BAS-100 "v?" 误判即此类）。

用法：
    python scripts/reqbas_audit.py [--docs docs] [--json out.json] [--md out.md]

退出码恒为 0；本脚本只做报告，不做判定门禁（CI 落地待 RGS-PLAN-003 §4.2 拍板）。
"""

import argparse
import io
import json
import os
import re
import sys
from collections import OrderedDict, defaultdict

# 文档类型与其在 IPA 共通フレーム 层级中的上位类型
LAYER_PARENT = OrderedDict([
    ("REQ", None),      # L1/L2 需求定义书（顶层，父文档可为其它 REQ）
    ("BAS", "REQ"),     # L3 基本设计书
    ("DTL", "BAS"),     # L4 详细设计书
    ("SPEC", "DTL"),    # L5 实现规格书
])

FILENAME_RE = re.compile(r"RGS-(SPEC-DTL|REQ|BAS|DTL)-([0-9A-Za-z\-]+?)[_\.]")
# 头部表格行：| 键 | 值 |
HEADER_ROW_RE = re.compile(r"^\s*\|\s*([^|]+?)\s*\|\s*(.+?)\s*\|\s*$", re.M)
DOCNO_RE = re.compile(r"RGS-(SPEC-DTL|REQ|BAS|DTL)-([0-9]{3})")

PARENT_KEYS = ("父文档", "父文書", "父文檔", "親文档", "上位文档", "上位文書", "主文档")
NO_PARENT_MARKERS = ("无", "無", "N/A", "n/a", "—", "-", "不适用", "顶层", "本文档为")


def read_head(path, nbytes=8000):
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        return fh.read(nbytes)


def parse_header(text):
    """从文档头部表格抽取键值对（只取第一张表，遇到 '修订历史' 停止）。"""
    cut = text.find("修订历史")
    if cut == -1:
        cut = text.find("改訂履歴")
    head = text[:cut] if cut > 0 else text
    out = {}
    for key, val in HEADER_ROW_RE.findall(head):
        key = key.strip().strip("*")
        if key in ("项目", "---", "内容") or set(key) <= set("-: "):
            continue
        out.setdefault(key, val.strip())
    return out


def classify_parent(raw):
    """返回 (status, parent_docno)。

    status ∈ {"parsed", "declared-root", "unrecognized-parent-format"}
    """
    if raw is None:
        return "no-parent-row", None
    stripped = raw.strip().strip("*").strip()
    m = DOCNO_RE.search(stripped)
    if m:
        kind = "SPEC" if m.group(1) == "SPEC-DTL" else m.group(1)
        return "parsed", "%s-%s" % (kind, m.group(2))
    if any(stripped.startswith(mark) for mark in NO_PARENT_MARKERS) or not stripped:
        return "declared-root", None
    return "unrecognized-parent-format", None


def docno_from_filename(path):
    base = os.path.basename(path)
    m = FILENAME_RE.search(base)
    if not m:
        return None
    kind = "SPEC" if m.group(1) == "SPEC-DTL" else m.group(1)
    num = m.group(2)
    return "%s-%s" % (kind, num)


def collect(docs_root):
    """返回 (docs, skipped)。

    skipped 收容「文件名以 RGS-<层>- 开头、但编号无法解析」的文件。
    必须显式收容而非静默 continue —— 静默丢弃与「按文件名机械配对」
    是同一类错误（RGS-PLAN-003 §0），只是发生在更上一层。
    """
    docs = []
    skipped = []
    for dirpath, dirnames, filenames in os.walk(docs_root):
        dirnames[:] = [d for d in dirnames if d != "_archive"]
        for name in filenames:
            if not name.endswith(".md") or not name.startswith("RGS-"):
                continue
            path = os.path.join(dirpath, name)
            docno = docno_from_filename(path)
            kind = docno.split("-")[0] if docno else None
            declared = next(
                (k for k in LAYER_PARENT if name.startswith("RGS-%s-" % k)), None
            )
            if kind not in LAYER_PARENT or (declared and kind != declared):
                if declared:
                    skipped.append({
                        "path": path.replace("\\", "/"),
                        "declared_kind": declared,
                        "parsed_docno": docno,
                    })
                continue
            head = read_head(path)
            hdr = parse_header(head)
            raw_parent = None
            for key in PARENT_KEYS:
                if key in hdr:
                    raw_parent = hdr[key]
                    break
            status, parent = classify_parent(raw_parent)
            # SPEC 常把上位 DTL 写在标题/正文而非「父文档」行：按编号同构回退
            if kind == "SPEC" and status in ("no-parent-row", "unrecognized-parent-format"):
                m = DOCNO_RE.search(head)
                if m and m.group(1) == "DTL":
                    status, parent = "parsed-fallback-body", "DTL-%s" % m.group(2)
            header_docno = None
            if "文档编号" in hdr:
                m = DOCNO_RE.search(hdr["文档编号"])
                if m:
                    k = "SPEC" if m.group(1) == "SPEC-DTL" else m.group(1)
                    header_docno = "%s-%s" % (k, m.group(2))
            docs.append({
                "path": path.replace("\\", "/"),
                "docno": docno,
                "header_docno": header_docno,
                "kind": kind,
                "version": hdr.get("版本"),
                "status_field": hdr.get("状态") or hdr.get("状態"),
                "raw_parent": raw_parent,
                "parent_status": status,
                "parent": parent,
            })
    docs.sort(key=lambda d: (list(LAYER_PARENT).index(d["kind"]), d["docno"], d["path"]))
    return docs, skipped


def count_by_filename(docs_root):
    """不经任何解析、纯按文件名前缀统计，用于与解析结果核对总数。"""
    totals = dict((k, 0) for k in LAYER_PARENT)
    for dirpath, dirnames, filenames in os.walk(docs_root):
        dirnames[:] = [d for d in dirnames if d != "_archive"]
        for name in filenames:
            if not name.endswith(".md"):
                continue
            for k in LAYER_PARENT:
                if name.startswith("RGS-%s-" % k):
                    totals[k] += 1
                    break
    return totals


def build_report(docs, skipped, filename_totals):
    by_docno = defaultdict(list)
    for d in docs:
        by_docno[d["docno"]].append(d)

    duplicates = {k: [d["path"] for d in v] for k, v in by_docno.items() if len(v) > 1}
    docno_mismatch = [
        {"path": d["path"], "filename_docno": d["docno"], "header_docno": d["header_docno"]}
        for d in docs
        if d["header_docno"] and d["header_docno"] != d["docno"]
    ]

    # 反向索引：parent docno -> 子文档列表
    children = defaultdict(list)
    for d in docs:
        if d["parent"]:
            children[d["parent"]].append(d["docno"])

    parse_buckets = defaultdict(list)
    for d in docs:
        parse_buckets[d["parent_status"]].append(d["path"])

    # 链路缺口：某层文档没有任何下一层子文档
    gaps = defaultdict(list)
    for kind, child_kind in (("REQ", "BAS"), ("BAS", "DTL"), ("DTL", "SPEC")):
        for docno, group in sorted(by_docno.items()):
            if group[0]["kind"] != kind:
                continue
            kids = [c for c in children.get(docno, []) if c.startswith(child_kind + "-")]
            if not kids:
                gaps["%s_without_%s" % (kind, child_kind)].append(docno)

    # 悬空父引用：父文档编号在仓库中不存在
    dangling = sorted({
        "%s -> %s" % (d["docno"], d["parent"])
        for d in docs
        if d["parent"] and d["parent"] not in by_docno
    })

    # SPEC 层 parsed-fallback-body 启发式校验：RGS-SPEC-DTL-NNN 的回退父应为 DTL-NNN
    fallback_ok, fallback_mismatch = 0, []
    for d in docs:
        if d["parent_status"] != "parsed-fallback-body":
            continue
        own = d["docno"].split("-", 1)[1]
        if d["parent"] == "DTL-%s" % own:
            fallback_ok += 1
        else:
            fallback_mismatch.append(
                {"path": d["path"], "docno": d["docno"], "fallback_parent": d["parent"]}
            )

    parsed_totals = {k: sum(1 for d in docs if d["kind"] == k) for k in LAYER_PARENT}
    reconciliation = {
        k: {
            "by_filename": filename_totals.get(k, 0),
            "parsed": parsed_totals[k],
            "skipped": sum(1 for s in skipped if s["declared_kind"] == k),
        }
        for k in LAYER_PARENT
    }
    balanced = all(
        v["by_filename"] == v["parsed"] + v["skipped"] for v in reconciliation.values()
    )

    return {
        "totals": parsed_totals,
        "corpus_reconciliation": reconciliation,
        "corpus_balanced": balanced,
        "skipped_unmatched_filename": skipped,
        "fallback_heuristic": {
            "confirmed_same_number": fallback_ok,
            "mismatch": fallback_mismatch,
        },
        "parse_buckets": {k: sorted(v) for k, v in parse_buckets.items()},
        "parse_bucket_counts": {k: len(v) for k, v in parse_buckets.items()},
        "duplicate_docnos": duplicates,
        "docno_filename_header_mismatch": docno_mismatch,
        "dangling_parent_refs": dangling,
        "chain_gaps": {k: sorted(v) for k, v in gaps.items()},
        "children_index": {k: sorted(set(v)) for k, v in sorted(children.items())},
        "docs": docs,
    }


def render_md(rep):
    L = []
    L.append("# RGS 设计链对账结果（reqbas_audit.py）\n")
    L.append("方法：RGS-PLAN-003 §0 —— 读头部「父文档」行反向索引，不按文件名编号机械配对。\n")
    L.append("## 1. 文档总数\n")
    L.append("| 层级 | 份数 |")
    L.append("|---|---:|")
    for k, v in rep["totals"].items():
        L.append("| %s | %d |" % (k, v))
    L.append("\n## 2. 父文档解析分桶（解析失败 ≠ 设计缺口）\n")
    L.append("| 分桶 | 份数 |")
    L.append("|---|---:|")
    for k, v in sorted(rep["parse_bucket_counts"].items()):
        L.append("| %s | %d |" % (k, v))
    L.append("\n## 2.1 语料核对（按文件名前缀总数 = 已解析 + 已跳过）\n")
    L.append("| 层级 | 按文件名 | 已解析 | 已跳过 | 平衡 |")
    L.append("|---|---:|---:|---:|---|")
    for k, v in rep["corpus_reconciliation"].items():
        ok = "是" if v["by_filename"] == v["parsed"] + v["skipped"] else "否"
        L.append("| %s | %d | %d | %d | %s |" % (k, v["by_filename"], v["parsed"], v["skipped"], ok))
    L.append("\n语料整体平衡：%s\n" % ("是" if rep["corpus_balanced"] else "否"))
    if rep["skipped_unmatched_filename"]:
        L.append("\n跳过文件（文件名声明层级前缀，但编号解析失败——不计入该层份数，需人工复核）：\n")
        for s in rep["skipped_unmatched_filename"]:
            L.append("- `%s`（声明层级 %s）" % (s["path"], s["declared_kind"]))
    L.append("\n## 3. 重复文档编号\n")
    if rep["duplicate_docnos"]:
        for k, paths in sorted(rep["duplicate_docnos"].items()):
            L.append("- **%s**：" % k)
            for p in paths:
                L.append("  - `%s`" % p)
    else:
        L.append("无。")
    L.append("\n## 4. 文件名编号与头部「文档编号」不一致\n")
    if rep["docno_filename_header_mismatch"]:
        L.append("| 文件 | 文件名编号 | 头部编号 |")
        L.append("|---|---|---|")
        for m in rep["docno_filename_header_mismatch"]:
            L.append("| `%s` | %s | %s |" % (m["path"], m["filename_docno"], m["header_docno"]))
    else:
        L.append("无。")
    L.append("\n## 5. 悬空父引用（父文档编号在仓库中不存在）\n")
    L.append("\n".join("- %s" % x for x in rep["dangling_parent_refs"]) or "无。")
    L.append("\n## 6. 链路缺口候选（需人工复核，非结论）\n")
    for k, v in sorted(rep["chain_gaps"].items()):
        L.append("\n### %s（%d）\n" % (k, len(v)))
        L.append(", ".join(v) or "无。")
    return "\n".join(L) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description="RGS REQ→BAS→DTL→SPEC 设计链对账")
    ap.add_argument("--docs", default="docs", help="文档根目录（默认 docs）")
    ap.add_argument("--json", dest="json_out", help="输出 JSON 报告路径")
    ap.add_argument("--md", dest="md_out", help="输出 Markdown 报告路径")
    args = ap.parse_args(argv)

    docs, skipped = collect(args.docs)
    filename_totals = count_by_filename(args.docs)
    rep = build_report(docs, skipped, filename_totals)

    if args.json_out:
        with io.open(args.json_out, "w", encoding="utf-8") as fh:
            fh.write(json.dumps(rep, ensure_ascii=False, indent=2))
    if args.md_out:
        with io.open(args.md_out, "w", encoding="utf-8") as fh:
            fh.write(render_md(rep))
    if not args.json_out and not args.md_out:
        out = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
        out.write(render_md(rep))
        out.flush()
    return 0


if __name__ == "__main__":
    sys.exit(main())
