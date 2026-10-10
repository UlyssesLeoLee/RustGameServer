#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""RGS shim ↔ 闪烁之光 协议缺口矩阵生成器.

把三份事实源对齐, 回答"离对接好还差多少、差在哪":

  1. 真实协议源  mock_data/flash_proto_registry.json
     (由 scripts/extract_flash_proto.py 从 Erlang pack/cfg 抽取, 1696 code)
  2. shim 真 handler  tools/rgs-shim-rust/src/registry.rs        (map.insert)
  3. shim stub 表    tools/rgs-shim-rust/src/registry_stubs.rs  (map.entry().or_insert)

三种状态 (只依据代码, 不做推测):
  real     — registry.rs 里有真 handler (map.insert)
  stub     — registry_stubs.rs 里有自动生成 stub (handle_stub, 丢 payload 回 unknown)
  missing  — 真实协议里有, shim 两个表都没有

另有一类 **phantom**: shim 里有, 但真实协议注册表里查不到
(多为 shim 自造测试 cmd, 如 10400 / 11001) —— 标出来, 避免被误当业务覆盖.

用法:
  python build_shim_gap_matrix.py \
      --registry ..\\mock_data\\flash_proto_registry.json \
      --shim     D:\\RustGameServer\\tools\\rgs-shim-rust\\src \
      --out      ..\\mock_data\\flash_gap_matrix.json \
      --doc      ..\\docs\\flash-shim-gap-matrix.md
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict

# registry.rs:        map.insert(10101, CmdEntry { handler: handlers::handle_register, name: "register", ... });
RE_REAL = re.compile(
    r"map\.insert\(\s*(\d+)\s*,\s*CmdEntry\s*\{\s*handler\s*:\s*([\w:]+)\s*,\s*name\s*:\s*\"([^\"]*)\""
)
# registry_stubs.rs:  map.entry(14100).or_insert(CmdEntry { handler: handle_stub, name: "stub-14100", source: "..." });
RE_STUB = re.compile(
    r"map\.entry\(\s*(\d+)\s*\)\.or_insert\(CmdEntry\s*\{\s*handler\s*:\s*([\w:]+)\s*,\s*name\s*:\s*\"([^\"]*)\""
)
# 分组注释: // battle (103 cmd)
RE_GROUP = re.compile(r"^\s*//\s*([a-z][a-z0-9-]*)\s*\((\d+)\s*cmd\)")


def read(path: str) -> str:
    with open(path, "rb") as fh:
        raw = fh.read()
    for enc in ("utf-8", "gb18030", "latin-1"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            continue
    return raw.decode("utf-8", errors="replace")


def parse_real(path: str) -> dict:
    out = {}
    for m in RE_REAL.finditer(read(path)):
        out[int(m.group(1))] = {"handler": m.group(2), "name": m.group(3)}
    return out


def parse_stubs(path: str) -> dict:
    text = read(path)
    group = None
    out = {}
    for line in text.splitlines():
        gm = RE_GROUP.match(line)
        if gm:
            group = gm.group(1)
        m = RE_STUB.search(line)
        if m:
            out[int(m.group(1))] = {"handler": m.group(2), "name": m.group(3),
                                    "group": group or "ungrouped"}
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="RGS shim ↔ 闪烁之光 协议缺口矩阵")
    ap.add_argument("--registry", required=True, help="flash_proto_registry.json")
    ap.add_argument("--shim", required=True, help="rgs-shim-rust/src 目录")
    ap.add_argument("--out", default=None)
    ap.add_argument("--doc", default=None)
    args = ap.parse_args()

    if not os.path.isfile(args.registry):
        print(f"[FAIL] 找不到协议注册表: {args.registry}", file=sys.stderr)
        return 2
    real_path = os.path.join(args.shim, "registry.rs")
    stub_path = os.path.join(args.shim, "registry_stubs.rs")
    for p in (real_path, stub_path):
        if not os.path.isfile(p):
            print(f"[FAIL] 找不到 shim 源文件: {p}", file=sys.stderr)
            return 2

    reg = json.load(open(args.registry, encoding="utf-8"))
    declared = {r["code"]: r for r in reg["rpcs"]}
    impl_only = set(reg["coverage"]["implemented_not_declared"])
    real_codes = set(declared) | impl_only

    handlers = parse_real(real_path)
    stubs = parse_stubs(stub_path)

    rows, by_module, by_group = [], defaultdict(lambda: Counter()), defaultdict(lambda: Counter())
    status_count = Counter()

    for code in sorted(real_codes):
        r = declared.get(code)
        if code in handlers:
            status, info = "real", handlers[code]
        elif code in stubs:
            status, info = "stub", stubs[code]
        else:
            status, info = "missing", None
        status_count[status] += 1

        module = r["module"] if r else f"proto_{code // 1000}"
        group = info.get("group") if info else None
        by_module[module][status] += 1
        if group:
            by_group[group][status] += 1

        row = {
            "code": code,
            "status": status,
            "module": module,
            "title": (r or {}).get("title", ""),
            "declared_in_cfg": code in declared,
            "shim_handler": (info or {}).get("handler"),
            "shim_name": (info or {}).get("name"),
            "shim_group": group,
        }
        if r:
            row["req"] = r.get("req") or []
            row["reply"] = r.get("reply") or []
            row["req_desc"] = r.get("req_desc", "")
            row["reply_desc"] = r.get("reply_desc", "")
        rows.append(row)

    # phantom: shim 里有但真实协议查不到
    phantom = []
    for code in sorted(set(handlers) | set(stubs)):
        if code not in real_codes:
            info = handlers.get(code) or stubs.get(code)
            phantom.append({
                "code": code,
                "handler": info.get("handler"),
                "name": info.get("name"),
                "group": info.get("group"),
            })

    total = len(real_codes)
    covered = status_count["real"] + status_count["stub"]
    doc = {
        "schema": "rgs.flash.shim-gap-matrix/v1",
        "sources": {
            "protocol": os.path.abspath(args.registry),
            "shim_real": real_path,
            "shim_stubs": stub_path,
        },
        "totals": {
            "real_protocol_codes": total,
            "declared_in_cfg": len(declared),
            "implemented_only": len(impl_only),
            "shim_real_handlers": len(handlers),
            "shim_stub_entries": len(stubs),
            "covered": covered,
            "coverage_pct": round(100.0 * covered / total, 2) if total else 0,
            "real_pct": round(100.0 * status_count["real"] / total, 2) if total else 0,
            "stub_pct": round(100.0 * status_count["stub"] / total, 2) if total else 0,
            "missing": status_count["missing"],
            "phantom_cmds": len(phantom),
        },
        "by_module": {k: dict(v) for k, v in sorted(by_module.items())},
        "by_shim_group": {k: dict(v) for k, v in sorted(by_group.items())},
        "phantom": phantom,
        "entries": rows,
    }

    out_json = args.out
    if out_json:
        os.makedirs(os.path.dirname(os.path.abspath(out_json)), exist_ok=True)
        with open(out_json, "w", encoding="utf-8") as fh:
            json.dump(doc, fh, ensure_ascii=False, indent=2)
    if args.doc:
        write_doc(args.doc, doc)

    t = doc["totals"]
    print(f"[OK] 真实协议 code      = {t['real_protocol_codes']}")
    print(f"[OK] shim 真 handler    = {t['shim_real_handlers']}")
    print(f"[OK] shim stub          = {t['shim_stub_entries']}")
    print(f"[OK] 覆盖               = {t['covered']} ({t['coverage_pct']}%)  "
          f"其中真 handler {t['real_pct']}% / stub {t['stub_pct']}%")
    print(f"[OK] 缺口 (missing)     = {t['missing']}")
    print(f"[OK] phantom cmd        = {t['phantom_cmds']}")
    if out_json:
        print(f"[OK] -> {os.path.abspath(out_json)}")
    if args.doc:
        print(f"[OK] -> {os.path.abspath(args.doc)}")
    return 0


def write_doc(path: str, doc: dict) -> None:
    t = doc["totals"]
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    L = [
        "# RGS shim ↔ 闪烁之光 协议缺口矩阵\n",
        f"> schema `{doc['schema']}` · 由 `scripts/build_shim_gap_matrix.py` 生成\n",
        f"> 协议源 `{doc['sources']['protocol']}`\n",
        f"> shim 真 handler `{doc['sources']['shim_real']}`\n",
        f"> shim stub 表 `{doc['sources']['shim_stubs']}`\n",
        "## 1. 总账\n",
        "| 指标 | 值 |", "|---|---|",
        f"| 闪烁之光真实 protocol code | **{t['real_protocol_codes']}** |",
        f"| 其中 cfg 有声明 | {t['declared_in_cfg']} |",
        f"| 仅已实现未声明 | {t['implemented_only']} |",
        f"| shim 真 handler (`map.insert`) | **{t['shim_real_handlers']}** |",
        f"| shim stub (`or_insert` + `handle_stub`) | {t['shim_stub_entries']} |",
        f"| **合计覆盖** | **{t['covered']} ({t['coverage_pct']}%)** |",
        f"| ├ 真 handler 占比 | {t['real_pct']}% |",
        f"| └ stub 占比 (丢 payload 回 unknown) | {t['stub_pct']}% |",
        f"| **缺口 missing** | **{t['missing']}** |",
        f"| phantom (shim 有 / 真实协议无) | {t['phantom_cmds']} |",
        "",
        "> ⚠️ 旧 `rgs-shim/README.md` 记的 \"6/514 = 1.2%\" 已过时: 514 是当时",
        "> 只看部分文件得到的口径. 按 Erlang pack/cfg 实证, 真实 code 为 "
        f"{t['real_protocol_codes']} 个.\n",
        "## 2. 按 proto 模块分布\n",
        "| module | real | stub | missing | 小计 |", "|---|---|---|---|---|",
    ]
    for mod, c in doc["by_module"].items():
        sub = sum(c.values())
        L.append(f"| `{mod}` | {c.get('real',0)} | {c.get('stub',0)} | "
                 f"{c.get('missing',0)} | {sub} |")
    L += ["", "## 3. 按 shim 分组 (仅 stub 表有分组标记)\n",
          "| group | real | stub | missing | 小计 |", "|---|---|---|---|---|"]
    for g, c in doc["by_shim_group"].items():
        sub = sum(c.values())
        L.append(f"| `{g}` | {c.get('real',0)} | {c.get('stub',0)} | "
                 f"{c.get('missing',0)} | {sub} |")
    L += ["", "## 4. Phantom cmd (shim 有, 真实协议注册表查不到)\n"]
    if doc["phantom"]:
        L += ["| code | handler | name |", "|---|---|---|"]
        for p in doc["phantom"]:
            L.append(f"| {p['code']} | `{p['handler']}` | {p['name']} |")
        L.append("")
        L.append("> 这些多为 shim 自造测试 cmd (如 heartbeat / role_list),")
        L.append("> **不能计入业务覆盖**.\n")
    else:
        L.append("(无)\n")
    L += [
        "## 5. 补齐入口\n",
        "每条 missing 条目在 `mock_data/flash_gap_matrix.json` 的 `entries` 里,",
        "已带 `req` / `reply` 字段规格 (类型 / 字段名 / 中文说明), 可直接照着写 handler.\n",
    ]
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L))


if __name__ == "__main__":
    sys.exit(main())
