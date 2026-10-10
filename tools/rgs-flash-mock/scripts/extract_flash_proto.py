#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""闪烁之光 (sszg / zsyz) 协议 + 配置抽取器 → RGS mock 规格源.

从已落地的游戏服务端包中抽取机器可读的协议注册表, 供 RGS mock 消费.

权威来源 (proto_*.erl 由 zsyz_tools/gen_proto 从 PHP 配置生成):

  1. <src>/zsyz_tools/gen_proto/cfg/proto_*.erl   协议声明 (code / log_title / req / reply)
  2. <src>/zsyz_server_mod/src/proto/proto_*.erl  mod 变体已实现 pack/unpack
  3. <src>/zsyz_server/src/proto/proto_*.erl      base 变体已实现 pack/unpack
  4. <src>/zsyz_server_mod/src/proto/rpc_cfg.erl  全量 RPC code 注册表

线格式 (从 pack/2 函数体实证提取, 不硬编码假设):

  <<(byte_size(Payload) + 2):32, Code:16, Payload/binary>>

源文件排版极不统一 (`, reply =` 换行后才跟 `[` / 空 `[]` / 内联 `[{...}]` /
`{array, name, type, "desc", [子字段]}`), 故解析基于**括号深度扫描切块**,
不使用行缩进启发式.

用法:
  python extract_flash_proto.py \
      --source D:\\rgs-flash-mock\\game-server \
      --out    ../mock_data/flash_proto_registry.json \
      --doc    ../docs/flash-game-proto-spec.md
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict

# ---------------------------------------------------------------- 词法工具

def read_text(path: str) -> str:
    """源包为 UTF-8 (已实证: gen_proto/cfg/proto_102.erl utf-8 OK / gb18030 FAIL)."""
    with open(path, "rb") as fh:
        raw = fh.read()
    for enc in ("utf-8", "gb18030", "latin-1"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            continue
    return raw.decode("utf-8", errors="replace")


def strip_comments(text: str) -> str:
    """删除 Erlang %% 行注释 (跳过字符串内的内容)."""
    out = []
    i, n, quote = 0, len(text), ""
    while i < n:
        ch = text[i]
        if quote:
            out.append(ch)
            if ch == "\\":
                if i + 1 < n:
                    out.append(text[i + 1])
                i += 2
                continue
            if ch == quote:
                quote = ""
            i += 1
            continue
        if ch in "\"'":
            quote = ch
            out.append(ch)
            i += 1
            continue
        if ch == "%" and text[i:i + 2] == "%%":
            j = text.find("\n", i)
            i = n if j < 0 else j
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def scan_balanced(text: str, open_idx: int) -> int:
    """text[open_idx] 为 ( [ { 之一; 返回配对闭括号下标, 找不到返回 -1."""
    depth = 0
    i, n, quote = open_idx, len(text), ""
    while i < n:
        ch = text[i]
        if quote:
            if ch == "\\":
                i += 2
                continue
            if ch == quote:
                quote = ""
            i += 1
            continue
        if ch in "\"'":
            quote = ch
            i += 1
            continue
        if ch == "%" and text[i:i + 2] == "%%":
            j = text.find("\n", i)
            i = n if j < 0 else j
            continue
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def split_top_level(text: str, sep: str = ",") -> list:
    """按顶层分隔符切分, 尊重引号与嵌套括号."""
    out, buf, depth, quote, esc = [], [], 0, "", False
    for ch in text:
        if esc:
            buf.append(ch)
            esc = False
            continue
        if quote:
            buf.append(ch)
            if ch == "\\":
                esc = True
            elif ch == quote:
                quote = ""
            continue
        if ch in "\"'":
            quote = ch
            buf.append(ch)
            continue
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == sep and depth == 0:
            out.append("".join(buf).strip())
            buf = []
            continue
        buf.append(ch)
    tail = "".join(buf).strip()
    if tail:
        out.append(tail)
    return out


def unquote(s: str) -> str:
    s = s.strip()
    if len(s) >= 2 and s[0] == s[-1] and s[0] in "\"'":
        return s[1:-1]
    return s


# ---------------------------------------------------------------- 字段解析

def _subfields_of(default_src: str, errors: list | None = None, ctx: str = "") -> list:
    """从 array 默认值里抠出子字段列表, 如 ?p_g_set(x, [{a,1},{b,2}]) 或 [ ... ]."""
    lb = default_src.find("[")
    if lb < 0:
        return []
    rb = scan_balanced(default_src, lb)
    if rb < 0:
        return []
    return parse_field_list(default_src[lb + 1:rb], errors, ctx + ".elem")


def parse_field(tuple_src: str, errors: list | None = None, ctx: str = "") -> dict | None:
    """{uint32, battle_id, "战斗位ID"} / {array, name, type, "desc", [子字段]}"""
    t = tuple_src.strip()
    if not (t.startswith("{") and t.endswith("}")):
        if errors is not None:
            errors.append({"ctx": ctx, "raw": t[:200], "why": "not a tuple"})
        return None
    parts = split_top_level(t[1:-1])
    if len(parts) < 2:
        if errors is not None:
            errors.append({"ctx": ctx, "raw": t[:200], "why": "fewer than 2 parts"})
        return None
    ftype, fname = unquote(parts[0]), unquote(parts[1])
    if not ftype or not fname or not re.match(r"^[A-Za-z_]\w*$", ftype):
        if errors is not None:
            errors.append({"ctx": ctx, "raw": t[:200], "why": f"bad type/name {ftype!r}/{fname!r}"})
        return None

    if ftype == "array" and len(parts) >= 3:
        field = {"type": "array", "name": fname, "elem_type": unquote(parts[2]), "desc": ""}
        if len(parts) >= 4:
            field["desc"] = unquote(parts[3])
        if len(parts) >= 5:
            field["default"] = parts[4].strip()
            sub = _subfields_of(parts[4], errors, ctx)
            if sub:
                field["elem_fields"] = sub
    else:
        field = {"type": ftype, "name": fname, "desc": ""}
        if len(parts) >= 3:
            field["desc"] = unquote(parts[2])
        if len(parts) >= 4:
            field["extra"] = [unquote(p) for p in parts[3:]]
    return field


def parse_field_list(content: str, errors: list | None = None, ctx: str = "") -> list:
    """解析 req/reply 字段列表.

    解析不了的元素**不静默丢弃**: 传入 errors 收集器后记录原文, 由 stats.unparsed
    暴露. 早期版本静默跳过, 导致 off-by-one 切片 bug (每条记录最后一个字段丢 `}`)
    长期不可见.
    """
    out = []
    for el in split_top_level(content, ","):
        el = el.strip()
        if not el or el in ("[]", "undefined"):
            continue
        f = parse_field(el)
        if f:
            out.append(f)
            continue
        # 宏包裹的字段定义, 如 ?p_g_set(mail, {array, mail, mail, "邮件列表", [...]})
        # 或 ?p_g_get(mail) —— 字段 schema 藏在宏参数内.
        if el.startswith("?"):
            lb = el.find("{")
            if lb >= 0:
                rb = scan_balanced(el, lb)
                if rb > 0:
                    f = parse_field(el[lb:rb + 1], None, ctx)
                    if f:
                        f["via_macro"] = el[:lb].strip()
                        out.append(f)
                        continue
            if errors is not None:
                # 裸 getter 宏 (?p_g_get(mail)): 本身不含字段定义,
                # schema 定义在对应的 ?p_g_set 形态里. 非解析失败.
                errors.append({"ctx": ctx, "raw": el[:200],
                               "why": "macro getter 无内联字段定义 (schema 见对应 ?p_g_set)"})
                return out
        if errors is not None:
            errors.append({"ctx": ctx, "raw": el[:200], "why": "unparseable element"})
    return out


# ---------------------------------------------------------------- #rpc{} 切块

RE_RPC_OPEN = re.compile(r"#rpc\s*\{")


def split_rpc_blocks(text: str) -> list:
    """按括号深度切出每个 #rpc{...} 的内部文本, 跳过已消费的区间避免重复计数."""
    blocks, pos, n = [], 0, len(text)
    while True:
        m = RE_RPC_OPEN.search(text, pos)
        if not m:
            break
        lb = m.end() - 1
        rb = scan_balanced(text, lb)
        if rb < 0:
            pos = m.end()
            continue
        blocks.append(text[lb + 1:rb])
        pos = rb + 1
    return blocks


RE_CODE = re.compile(r"\bcode\s*=\s*(\d+)")
RE_LOG_TITLE = re.compile(r'log_title\s*=\s*"((?:[^"\\]|\\.)*)"')
RE_SIDE = re.compile(r"\b(req|reply)(?!_)\s*=\s*")


def _find_side(block: str, side: str):
    """按 side 各自定位赋值点 (不能用 RE_SIDE.search 取首个匹配:
    对 req 先于 reply 出现的记录, 首个匹配恒为 req, 会导致 reply 永远解析为空)."""
    return re.search(r"\b" + side + r"(?!_)\s*=\s*", block)


def parse_rpc_block(block: str, errors: list | None = None) -> dict | None:
    mc = RE_CODE.search(block)
    if not mc:
        return None
    code = int(mc.group(1))
    rpc = {
        "code": code,
        "title": "",
        "req": [],
        "reply": [],
        "req_desc": "",
        "reply_desc": "",
    }
    mt = RE_LOG_TITLE.search(block)
    if mt:
        rpc["title"] = mt.group(1)

    for side in ("req", "reply"):
        md = re.search(r"\b" + side + r"_desc\s*=\s*(.+)", block)
        if md:
            rpc[side + "_desc"] = md.group(1).strip().rstrip(",")

        ms = _find_side(block, side)
        if not ms:
            continue
        k = ms.end()
        rest = block[k:]
        lead = len(rest) - len(rest.lstrip())
        head = k + lead
        ctx = f"{code}.{side}"
        if block[head:head + 1] == "[":
            rb = scan_balanced(block, head)
            if rb < 0:
                if errors is not None:
                    errors.append({"ctx": ctx, "raw": block[head:head + 120], "why": "unbalanced ["})
                continue
            rpc[side] = parse_field_list(block[head + 1:rb], errors, ctx)
        elif block[head:head + 1] == "{":
            rb = scan_balanced(block, head)
            if rb < 0:
                if errors is not None:
                    errors.append({"ctx": ctx, "raw": block[head:head + 120], "why": "unbalanced {"})
                continue
            f = parse_field(block[head:rb + 1], errors, ctx)
            rpc[side] = [f] if f else []

    return rpc


# ---------------------------------------------------------------- 模块信息

RE_MODULE = re.compile(r"-module\(([a-z0-9_]+)\)")
RE_INFO_TITLE = re.compile(r'info\(\)\s*->\s*\{\s*"((?:[^"\\]|\\.)*)"')
RE_HRL = re.compile(r'"([a-z0-9_]+\.hrl)"')


def parse_module_meta(text: str) -> dict:
    meta = {"title": "", "hrl_deps": []}
    m = RE_MODULE.search(text)
    if m:
        meta["module"] = m.group(1)
    m = RE_INFO_TITLE.search(text)
    if m:
        meta["title"] = m.group(1)
    if not meta["hrl_deps"]:
        mi = text.find("info()")
        if mi >= 0:
            me = text.find("}.", mi)
            meta["hrl_deps"] = RE_HRL.findall(text[mi:me if me > 0 else len(text)])
    return meta


def parse_cfg_file(path: str, errors: list | None = None) -> tuple[dict, list]:
    raw = read_text(path)
    meta = parse_module_meta(raw)
    rpcs = []
    for block in split_rpc_blocks(strip_comments(raw)):
        r = parse_rpc_block(block, errors)
        if r:
            r["module"] = meta.get("module", "")
            r["module_title"] = meta.get("title", "")
            rpcs.append(r)
    return meta, rpcs


# ---------------------------------------------------------------- 已实现 / 注册表

RE_PACK = re.compile(r"^pack\((\d+)\s*,\s*(cli|srv)\s*,")
RE_CFG_GET = re.compile(r"^get\((\d+)\)\s*->")
RE_FRAME = re.compile(
    r"<<\(byte_size\((\w+)\)\s*\+\s*(\d+)\):(\d+)\s*,\s*(\d+):(\d+)\s*,\s*\1/binary>>"
)


def scan_impl_dir(path: str) -> dict:
    """统计 src/proto 下 pack 已实现的 code, 并实证线格式."""
    packs: dict[int, set] = defaultdict(set)
    frames: Counter = Counter()
    files = 0
    if not os.path.isdir(path):
        return {"pack": {}, "frames": {}, "files": 0}
    for name in sorted(os.listdir(path)):
        if not name.endswith(".erl"):
            continue
        files += 1
        for line in read_text(os.path.join(path, name)).splitlines():
            m = RE_PACK.match(line)
            if m:
                packs[int(m.group(1))].add(m.group(2))
            f = RE_FRAME.search(line)
            if f:
                frames[f"{f.group(3)}/{f.group(5)}+{f.group(2)}"] += 1
    return {"pack": {str(k): sorted(v) for k, v in sorted(packs.items())},
            "frames": dict(frames), "files": files}


def scan_rpc_cfg(path: str) -> list:
    if not os.path.isfile(path):
        return []
    out = []
    for line in read_text(path).splitlines():
        m = RE_CFG_GET.match(line)
        if m:
            out.append(int(m.group(1)))
    return sorted(set(out))


# ---------------------------------------------------------------- 主流程

def build(source: str) -> dict:
    cfg_dir = os.path.join(source, "zsyz_tools", "gen_proto", "cfg")
    modules, all_rpcs, dupes, seen = [], [], [], {}
    errors: list = []
    for name in sorted(os.listdir(cfg_dir)):
        if not (name.startswith("proto_") and name.endswith(".erl")):
            continue
        meta, rpcs = parse_cfg_file(os.path.join(cfg_dir, name), errors)
        meta["file"] = f"zsyz_tools/gen_proto/cfg/{name}"
        meta["rpc_count"] = len(rpcs)
        for r in rpcs:
            if r["code"] in seen:
                dupes.append({"code": r["code"], "first": seen[r["code"]],
                              "second": meta["file"]})
            seen[r["code"]] = meta["file"]
        modules.append(meta)
        all_rpcs.extend(rpcs)
    all_rpcs.sort(key=lambda r: r["code"])

    mod_impl = scan_impl_dir(os.path.join(source, "zsyz_server_mod", "src", "proto"))
    base_impl = scan_impl_dir(os.path.join(source, "zsyz_server", "src", "proto"))
    registry = scan_rpc_cfg(os.path.join(source, "zsyz_server_mod", "src", "proto", "rpc_cfg.erl"))

    impl_codes = set(map(int, mod_impl["pack"])) | set(map(int, base_impl["pack"]))
    declared = {r["code"] for r in all_rpcs}

    shapes = Counter()
    shapes.update(mod_impl["frames"])
    shapes.update(base_impl["frames"])
    dominant = shapes.most_common(1)[0][0] if shapes else "unknown"
    # 形状形如 "32/16+2" = <len_bits>/<code_bits>+<len_add>
    _parts = dominant.split("/")
    len_bits = _parts[0] if _parts else "?"
    code_bits = _parts[1].split("+")[0] if len(_parts) > 1 else "?"
    len_add = _parts[1].split("+")[1] if len(_parts) > 1 and "+" in _parts[1] else "?"

    ftypes = Counter()
    total_fields = 0
    for r in all_rpcs:
        for side in ("req", "reply"):
            for f in r.get(side) or []:
                ftypes[f["type"]] += 1
                total_fields += 1
                for sub in f.get("elem_fields") or []:
                    ftypes["array." + sub["type"]] += 1
                    total_fields += 1

    return {
        "schema": "rgs.flash.proto-registry/v1",
        "source": {
            "package_root": source,
            "upstream": "闪烁之光 / shiyuegame (v200408, v200429)",
            "encoding": "utf-8 (实证)",
            "generator": "zsyz_tools/gen_proto/main.erl",
        },
        "framing": {
            "shape": dominant,
            "length_bits": int(len_bits) if len_bits.isdigit() else None,
            "code_bits": int(code_bits) if code_bits.isdigit() else None,
            "length_offset": int(len_add) if len_add.isdigit() else None,
            "length_covers": "code + payload",
            "byte_order": "big-endian (Erlang 默认 bit syntax)",
            "description": "<<(byte_size(Payload) + 2):32, Code:16, Payload/binary>>",
            "evidence": "从 pack/2 函数体正则统计, 非硬编码假设",
        },
        "stats": {
            "cfg_modules": len(modules),
            "declared_rpcs": len(all_rpcs),
            "unique_codes": len(declared),
            "duplicate_codes": len(dupes),
            "impl_files_mod": mod_impl["files"],
            "impl_files_base": base_impl["files"],
            "implemented_codes": len(impl_codes),
            "rpc_cfg_registered": len(registry),
            "field_types": dict(ftypes.most_common()),
            "total_fields": total_fields,
            "unparsed_elements": len([e for e in errors if e.get("why") == "unparseable element"]),
            "macro_getter_without_schema": len(errors) - len(
                [e for e in errors if e.get("why") == "unparseable element"]),
        },
        "coverage": {
            "declared_not_implemented": sorted(declared - impl_codes),
            "implemented_not_declared": sorted(impl_codes - declared),
            "declared_not_in_rpc_cfg": sorted(declared - set(registry)),
        },
        "duplicates": dupes,
        "unparsed_sample": errors[:40],
        "modules": modules,
        "rpcs": all_rpcs,
        "rpc_cfg_registry": registry,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="闪烁之光协议/配置抽取 → RGS mock")
    ap.add_argument("--source", default=r"D:\rgs-flash-mock\game-server")
    ap.add_argument("--out", default=None)
    ap.add_argument("--doc", default=None)
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    cfg_dir = os.path.join(args.source, "zsyz_tools", "gen_proto", "cfg")
    if not os.path.isdir(cfg_dir):
        print(f"[FAIL] 协议声明目录不存在: {cfg_dir}", file=sys.stderr)
        print("       请先按部署范围把 zsyz_tools 落地, 再运行本脚本.", file=sys.stderr)
        return 2

    doc = build(args.source)
    s = doc["stats"]

    out_json = args.out or os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "mock_data",
        "flash_proto_registry.json")
    os.makedirs(os.path.dirname(os.path.abspath(out_json)), exist_ok=True)
    with open(out_json, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2)
    if args.doc:
        write_doc(args.doc, doc)

    if not args.quiet:
        print(f"[OK] modules={s['cfg_modules']} rpcs={s['declared_rpcs']} "
              f"codes={s['unique_codes']} fields={s['total_fields']} "
              f"impl={s['implemented_codes']} registry={s['rpc_cfg_registered']}")
        print(f"[OK] framing={doc['framing']['shape']}  -> {os.path.abspath(out_json)}")
        if doc["duplicates"]:
            print(f"[WARN] duplicate codes: {len(doc['duplicates'])}")
        if s["unparsed_elements"]:
            print(f"[WARN] unparsed field elements: {s['unparsed_elements']} "
                  f"(见 JSON unparsed_sample) — 抽取不完整, 需人工确认")
    return 0


def write_doc(path: str, doc: dict) -> None:
    s, f, c = doc["stats"], doc["framing"], doc["coverage"]
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    L = [
        "# 闪烁之光 协议 / 配置规格 (RGS mock 规格源)\n",
        f"> 抽取脚本 `scripts/extract_flash_proto.py` · schema `{doc['schema']}`\n",
        f"> 源包 `{doc['source']['package_root']}` · 编码 {doc['source']['encoding']}\n",
        f"> 上游 {doc['source']['upstream']} · 生成器 `{doc['source']['generator']}`\n",
        "## 1. 线格式 (Framing)\n",
        f"从 {s['impl_files_mod'] + s['impl_files_base']} 个 pack 函数体**实证**提取：\n",
        f"- 形状：`{f['description']}`",
        f"- 长度字段：**{f['length_bits']} bit**，覆盖 `{f['length_covers']}`",
        f"- 命令码：**{f['code_bits']} bit**",
        f"- 长度字段覆盖 `{f['length_covers']}`（偏移 +{f['length_offset']}）",
        f"- 字节序：{f['byte_order']}\n",
        "## 2. 规模\n",
        "| 指标 | 值 |", "|---|---|",
        f"| 协议声明模块 (cfg) | {s['cfg_modules']} |",
        f"| 声明 RPC 总数 | {s['declared_rpcs']} |",
        f"| 去重 command code | {s['unique_codes']} |",
        f"| 重复 code | {s['duplicate_codes']} |",
        f"| 已实现 pack 的 code (mod + base) | {s['implemented_codes']} |",
        f"| rpc_cfg 注册 code | {s['rpc_cfg_registered']} |",
        "",
        "字段类型分布 (含 array 元素类型)：\n",
        "| type | count |", "|---|---|",
    ]
    for t, n in s["field_types"].items():
        L.append(f"| `{t}` | {n} |")
    L += [
        "",
        "## 3. 覆盖缺口\n",
        f"- 声明但**未实现** pack：{len(c['declared_not_implemented'])} 个",
        f"- 实现了但 cfg 未声明：{len(c['implemented_not_declared'])} 个",
        f"- 声明但未进 rpc_cfg：{len(c['declared_not_in_rpc_cfg'])} 个\n",
        "## 4. 模块清单\n",
        "| module | 标题 | RPC 数 | hrl 依赖 |", "|---|---|---|---|",
    ]
    for m in doc["modules"]:
        L.append(f"| `{m.get('module','')}` | {m.get('title','')} | {m['rpc_count']} | "
                 f"{', '.join(m['hrl_deps']) or '-'} |")
    L += ["", "## 5. 全量 RPC 注册表\n",
          "| code | 模块 | 标题 | req | reply |", "|---|---|---|---|---|"]
    for r in doc["rpcs"]:
        L.append(f"| {r['code']} | `{r['module']}` | {r['title']} | "
                 f"{len(r.get('req') or [])} | {len(r.get('reply') or [])} |")
    L.append("")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L))


if __name__ == "__main__":
    sys.exit(main())
