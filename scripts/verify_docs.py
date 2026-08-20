#!/usr/bin/env python3
"""cypher
CREATE
  (file:File {name: "verify_docs.py", type: "file", language: "python"}),
  (read:Function {name: "read_text", type: "function", signature: "(path: Path) -> str"}),
  (slug:Function {name: "markdown_slug", type: "function", signature: "(heading: str) -> str"}),
  (anchors:Function {name: "heading_anchors", type: "function", signature: "(content: str) -> set[str]"}),
  (target:Function {name: "link_target", type: "function", signature: "(raw_target: str) -> tuple[str, str]"}),
  (links:Function {name: "check_links", type: "function", signature: "(documents: list[Path]) -> list[str]"}),
  (declared_id:Function {name: "header_document_id", type: "function", signature: "(header: str, filename_id: str) -> str | None"}),
  (headers:Function {name: "check_headers", type: "function", signature: "(documents: list[Path]) -> list[str]"}),
  (registry:Function {name: "check_registry", type: "function", signature: "() -> list[str]"}),
  (main:Function {name: "main", type: "function", signature: "() -> int"}),
  (root:Variable {name: "ROOT", type: "variable"}),
  (docs_dir:Variable {name: "DOCS_DIR", type: "variable"}),
  (registry_path:Variable {name: "REGISTRY_PATH", type: "variable"}),
  (link_pattern:Variable {name: "LINK_PATTERN", type: "variable"}),
  (heading_pattern:Variable {name: "HEADING_PATTERN", type: "variable"}),
  (document_id_pattern:Variable {name: "DOCUMENT_ID_PATTERN", type: "variable"}),
  (filename_document_id_pattern:Variable {name: "FILENAME_DOCUMENT_ID_PATTERN", type: "variable"}),
  (header_document_id_field:Variable {name: "HEADER_DOCUMENT_ID_FIELD", type: "variable"}),
  (filename_version:Variable {name: "VERSION_IN_FILENAME", type: "variable"}),
  (header_version:Variable {name: "VERSION_IN_HEADER", type: "variable"}),
  (file)-[:CONTAINS]->(read),
  (file)-[:CONTAINS]->(slug),
  (file)-[:CONTAINS]->(anchors),
  (file)-[:CONTAINS]->(target),
  (file)-[:CONTAINS]->(links),
  (file)-[:CONTAINS]->(declared_id),
  (file)-[:CONTAINS]->(headers),
  (file)-[:CONTAINS]->(registry),
  (file)-[:CONTAINS]->(main),
  (file)-[:CONTAINS]->(root),
  (file)-[:CONTAINS]->(docs_dir),
  (file)-[:CONTAINS]->(registry_path),
  (file)-[:CONTAINS]->(link_pattern),
  (file)-[:CONTAINS]->(heading_pattern),
  (file)-[:CONTAINS]->(document_id_pattern),
  (file)-[:CONTAINS]->(filename_document_id_pattern),
  (file)-[:CONTAINS]->(header_document_id_field),
  (file)-[:CONTAINS]->(filename_version),
  (file)-[:CONTAINS]->(header_version),
  (anchors)-[:CALLS]->(slug),
  (anchors)-[:USES]->(heading_pattern),
  (links)-[:CALLS]->(read),
  (links)-[:CALLS]->(anchors),
  (links)-[:CALLS]->(target),
  (links)-[:USES]->(link_pattern),
  (headers)-[:CALLS]->(read),
  (headers)-[:CALLS]->(declared_id),
  (headers)-[:USES]->(filename_document_id_pattern),
  (headers)-[:USES]->(filename_version),
  (headers)-[:USES]->(header_version),
  (declared_id)-[:USES]->(document_id_pattern),
  (declared_id)-[:USES]->(header_document_id_field),
  (registry)-[:CALLS]->(read),
  (registry)-[:USES]->(registry_path),
  (main)-[:CALLS]->(links),
  (main)-[:CALLS]->(headers),
  (main)-[:CALLS]->(registry),
  (main)-[:USES]->(root),
  (main)-[:USES]->(docs_dir);
"""
"""RGS 文档完整性校验：链接/锚点、受控头信息、目录登记和版本命名。"""

from pathlib import Path
import re
import sys
import tomllib
import urllib.parse


ROOT = Path(__file__).resolve().parents[1]
DOCS_DIR = ROOT / "docs"
REGISTRY_PATH = DOCS_DIR / "document-registry.toml"
LINK_PATTERN = re.compile(r"(?<!!)\[([^\]]+)\]\(([^)]+)\)")
HEADING_PATTERN = re.compile(r"^\s{0,3}#{1,6}\s+(.+?)\s*#*\s*$")
DOCUMENT_ID_PATTERN = re.compile(r"(RGS-[A-Z]+(?:-[A-Z]+)*-\d+(?:-ADD\d+)?)(?![A-Za-z0-9-])")
FILENAME_DOCUMENT_ID_PATTERN = re.compile(r"(RGS-[A-Z]+(?:-[A-Z]+)*-\d+)(?![A-Za-z0-9-])")
HEADER_DOCUMENT_ID_FIELD = re.compile(
    r"^\|\s*(?:文档|决策|ADR)(?:编号|ID)\s*\|\s*(.*?)\s*\|",
    re.MULTILINE | re.IGNORECASE,
)
VERSION_IN_FILENAME = re.compile(r"_v(\d+\.\d+)\.md$", re.IGNORECASE)
VERSION_IN_HEADER = re.compile(r"\|\s*(?:文档)?版本(?:号)?\s*\|\s*v?(\d+\.\d+)", re.IGNORECASE)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="strict")


def markdown_slug(heading: str) -> str:
    """生成与 GitHub Markdown 标题锚点兼容的保守 slug。"""
    plain = re.sub(r"`([^`]*)`", r"\1", heading)
    plain = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", plain)
    plain = re.sub(r"[*_~]", "", plain).lower()
    plain = re.sub(r"[^\w\s-]", "", plain, flags=re.UNICODE)
    return re.sub(r"\s+", "-", plain).strip("-")


def heading_anchors(content: str) -> set[str]:
    anchors: set[str] = set()
    occurrences: dict[str, int] = {}
    for line in content.splitlines():
        match = HEADING_PATTERN.match(line)
        if not match:
            continue
        base = markdown_slug(match.group(1))
        if not base:
            continue
        index = occurrences.get(base, 0)
        occurrences[base] = index + 1
        anchors.add(base if index == 0 else f"{base}-{index}")
    return anchors


def link_target(raw_target: str) -> tuple[str, str]:
    """提取本地 Markdown 链接的文件部分和可选锚点。"""
    target = raw_target.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0]
    file_part, separator, fragment = target.partition("#")
    return urllib.parse.unquote(file_part), urllib.parse.unquote(fragment) if separator else ""


def check_links(documents: list[Path]) -> list[str]:
    errors: list[str] = []
    content_cache = {path.resolve(): read_text(path) for path in documents}
    anchors_cache = {path: heading_anchors(content) for path, content in content_cache.items()}
    for source, content in content_cache.items():
        for line_number, line in enumerate(content.splitlines(), 1):
            for match in LINK_PATTERN.finditer(line):
                label, raw_target = match.groups()
                parsed = urllib.parse.urlsplit(raw_target.strip())
                if parsed.scheme or raw_target.startswith("//"):
                    continue
                file_part, fragment = link_target(raw_target)
                destination = source if not file_part else (source.parent / file_part).resolve()
                if not destination.exists():
                    errors.append(f"{source.relative_to(ROOT)}:{line_number} 链接目标不存在: [{label}]({raw_target})")
                    continue
                if not fragment:
                    continue
                if not destination.is_file():
                    errors.append(f"{source.relative_to(ROOT)}:{line_number} 目录链接不能带锚点: [{label}]({raw_target})")
                    continue
                target_anchors = anchors_cache.get(destination)
                if target_anchors is None:
                    target_anchors = heading_anchors(read_text(destination))
                    anchors_cache[destination] = target_anchors
                if fragment not in target_anchors:
                    errors.append(
                        f"{source.relative_to(ROOT)}:{line_number} 锚点不存在: [{label}]({raw_target})"
                    )
    return errors


def header_document_id(header: str, filename_id: str) -> str | None:
    """以文档头受控编号字段为准；旧ADR短编号由文件名前缀规范化。"""
    field_match = HEADER_DOCUMENT_ID_FIELD.search(header)
    if not field_match:
        return None
    field = field_match.group(1)
    declared_match = DOCUMENT_ID_PATTERN.search(field)
    if declared_match:
        return declared_match.group(1)
    return filename_id if filename_id.removeprefix("RGS-") in field else None


def check_headers(documents: list[Path]) -> list[str]:
    errors: list[str] = []
    declared_ids: dict[str, list[Path]] = {}
    for path in documents:
        if not path.name.startswith("RGS-"):
            continue
        content = read_text(path)
        header = content[:1600]
        id_match = FILENAME_DOCUMENT_ID_PATTERN.search(path.name)
        if not id_match:
            errors.append(f"{path.relative_to(ROOT)} 文件名缺少正式 RGS 文档编号")
            continue
        filename_id = id_match.group(1)
        document_id = header_document_id(header, filename_id)
        if document_id is None:
            errors.append(f"{path.relative_to(ROOT)} 文档头缺少与文件名一致的受控文档编号字段")
        elif re.sub(r"-ADD\d+$", "", document_id) != filename_id:
            errors.append(
                f"{path.relative_to(ROOT)} 文件名编号 {filename_id} 与文档头编号 {document_id} 不一致"
            )
        else:
            declared_ids.setdefault(document_id, []).append(path)
        filename_version = VERSION_IN_FILENAME.search(path.name)
        header_version = VERSION_IN_HEADER.search(header)
        if filename_version:
            if not header_version:
                errors.append(f"{path.relative_to(ROOT)} 文件名带版本但文档头缺少版本字段")
            elif filename_version.group(1) != header_version.group(1):
                errors.append(
                    f"{path.relative_to(ROOT)} 文件名版本 {filename_version.group(1)} 与文档头版本 {header_version.group(1)} 不一致"
                )
        if re.search(r"^#{1,6} .*#{2,}\s+\d+\.\d+", content, re.MULTILINE):
            errors.append(f"{path.relative_to(ROOT)} 存在粘连的 Markdown 标题")
    for document_id, paths in declared_ids.items():
        if len(paths) > 1:
            relative_paths = ", ".join(str(path.relative_to(ROOT)) for path in paths)
            errors.append(f"正式文档编号 {document_id} 重复声明: {relative_paths}")
    return errors


def check_registry() -> list[str]:
    errors: list[str] = []
    try:
        with REGISTRY_PATH.open("rb") as registry_file:
            registry = tomllib.load(registry_file)
    except (FileNotFoundError, tomllib.TOMLDecodeError) as exc:
        return [f"无法读取文档登记册 {REGISTRY_PATH.relative_to(ROOT)}: {exc}"]
    readme = read_text(DOCS_DIR / "README.md")
    registered_paths: set[str] = set()
    seen_category_ids: set[str] = set()
    seen_sections: set[str] = set()
    for category in registry.get("category", []):
        category_id = str(category.get("id", "")).strip()
        path = str(category.get("path", "")).strip()
        title = str(category.get("title", "")).strip()
        section = str(category.get("readme_section", "")).strip()
        for label, value, seen in (
            ("id", category_id, seen_category_ids),
            ("path", path, registered_paths),
            ("readme_section", section, seen_sections),
        ):
            if not value:
                errors.append(f"document-registry.toml 存在没有 {label} 的 category")
            elif value in seen:
                errors.append(f"document-registry.toml category 的 {label} 重复: {value}")
            else:
                seen.add(value)
        if not title:
            errors.append("document-registry.toml 存在没有 title 的 category")
        if category_id and path and not path.startswith(f"{category_id}-"):
            errors.append(f"category id/path 不一致: {category_id} / {path}")
        if section and not re.fullmatch(r"\d+\.\d+", section):
            errors.append(f"category readme_section 格式无效: {section}")
        if path and not (DOCS_DIR / path).is_dir():
            errors.append(f"登记目录不存在: docs/{path}")
        if path and f"({path}/" not in readme:
            errors.append(f"docs/README.md 未导航登记目录: docs/{path}")
        if section and title and f"### {section} {title}" not in readme:
            errors.append(f"docs/README.md 未包含 category 标题: {section} {title}")
    actual_paths = {path.name for path in DOCS_DIR.iterdir() if path.is_dir() and re.match(r"^\d{2}-", path.name)}
    if actual_paths != registered_paths:
        missing = sorted(actual_paths - registered_paths)
        stale = sorted(registered_paths - actual_paths)
        if missing:
            errors.append(f"document-registry.toml 未登记目录: {', '.join(missing)}")
        if stale:
            errors.append(f"document-registry.toml 登记了不存在目录: {', '.join(stale)}")

    seen_planned_ids: set[str] = set()
    for entry in registry.get("planned_document", []):
        document_id = str(entry.get("id", "")).strip()
        status = str(entry.get("status", "")).strip()
        if not document_id:
            errors.append("document-registry.toml 存在没有 id 的 planned_document")
            continue
        if document_id in seen_planned_ids:
            errors.append(f"document-registry.toml planned_document id 重复: {document_id}")
        seen_planned_ids.add(document_id)
        if not re.fullmatch(r"RGS-[A-Z]+(?:-[A-Z]+)*-\d+(?:-ADD\d+)?", document_id):
            errors.append(f"planned_document id 格式无效: {document_id}")
        if status not in {"planned", "retired"}:
            errors.append(f"planned_document 状态无效: {document_id} / {status or '（空）'}")
        if not str(entry.get("title", "")).strip() or not str(entry.get("owner", "")).strip():
            errors.append(f"planned_document 缺少 title 或 owner: {document_id}")
        if status == "retired" and not str(entry.get("note", "")).strip():
            errors.append(f"retired planned_document 缺少迁移说明: {document_id}")
        if status == "retired" and entry.get("allow_reference") is True:
            errors.append(f"retired planned_document 不得 allow_reference: {document_id}")
    return errors


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    if not DOCS_DIR.is_dir():
        print("[FAIL] 未找到 docs/ 目录", file=sys.stderr)
        return 1
    documents = sorted(DOCS_DIR.rglob("*.md")) + sorted(ROOT.glob("*.md"))
    errors = check_links(documents) + check_headers(documents) + check_registry()
    print(f"[*] 已扫描 {len(documents)} 个 Markdown 文件。")
    if errors:
        for error in sorted(set(errors)):
            print(f"[FAIL] {error}")
        print(f"文档完整性检查失败：{len(set(errors))} 项阻断问题。")
        return 1
    print("[PASS] 文件链接、标题锚点、文档头、版本命名与目录登记均有效。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
