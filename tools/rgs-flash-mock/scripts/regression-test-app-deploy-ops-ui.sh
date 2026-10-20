#!/usr/bin/env bash
# rgs-flash-mock app 独立更新 + ops UI 回归自检
#
# 覆盖 README.md 已知缺口第 4 条 (app 独立更新 缺 4 用例) 与第 5 条 (ops UI 缺 6 用例)。
# 用例定义真源: docs/15-IPA-完全对齐438cmds/RGS-TEST-CASES-2026-09-07_v0.2.md
#   §3.3 app 独立更新 (L317-325) / §3.4 ops 运维 UI (L327-338)
#
# 本脚本是**真实断言**，不是 echo 式假绿：
#   - 10 个 scenario 必须齐全、无重复
#   - 每个 scenario 必须有 name / expected / assertions
#   - fixture 内嵌 assertions 逐条执行 (field 路径取值比对 equals)
#   - OPS-UI-003/004 额外扫描明文 env/secret 值 (8/27 11:06 JST 凭据硬 ban)
# 任一不满足 → 打印具体缺什么 + exit 1
#
# 退出码: 0 = 全部断言通过 / 1 = 有断言失败

set -uo pipefail

cd "$(dirname "$0")/.."

# Windows Git Bash 默认 stdout 是 GBK, 输出 ✅/❌ 会 UnicodeEncodeError
export PYTHONIOENCODING=utf-8

echo "=== rgs-flash-mock app-deploy + ops-ui 回归自检 ==="
echo "  mock 项目根: $(pwd)"
echo

python - <<'PY'
import json
import pathlib
import re
import sys

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except AttributeError:
    pass

FIXTURES = {
    "mock_data/app_deploy.json": [
        "APP-DEPLOY-001", "APP-DEPLOY-002", "APP-DEPLOY-003", "APP-DEPLOY-004",
    ],
    "mock_data/ops_ui.json": [
        "OPS-UI-001", "OPS-UI-002", "OPS-UI-003", "OPS-UI-004", "OPS-UI-005", "OPS-UI-006",
    ],
}

# 明文 secret 扫描: key 名命中敏感词, 且 value 非空且不是脱敏占位
SENSITIVE_KEY = re.compile(
    r"(?i)(password|passwd|secret|token|api[_-]?key|credential|private[_-]?key)"
)
ALLOWED_PLACEHOLDERS = {"", "REDACTED", "REDACTED-***", "<redacted>", "null", "None"}

failures = []
checks = 0


def resolve(obj, path):
    cur = obj
    for part in path.split("."):
        if isinstance(cur, list):
            cur = cur[int(part)]
        else:
            cur = cur[part]
    return cur


def scan_plaintext_secrets(node, trail=""):
    """返回 (key_path, value) 列表: 敏感 key 且 value 是明文。"""
    hits = []
    if isinstance(node, dict):
        for k, v in node.items():
            key_path = f"{trail}.{k}" if trail else k
            if SENSITIVE_KEY.search(k) and isinstance(v, str):
                if v.strip() not in ALLOWED_PLACEHOLDERS:
                    hits.append((key_path, v))
            hits.extend(scan_plaintext_secrets(v, key_path))
    elif isinstance(node, list):
        for i, v in enumerate(node):
            hits.extend(scan_plaintext_secrets(v, f"{trail}[{i}]"))
    return hits


for fixture_path, expected_ids in FIXTURES.items():
    p = pathlib.Path(fixture_path)
    if not p.exists():
        failures.append(f"fixture 缺失: {fixture_path}")
        print(f"  ❌ {fixture_path} 不存在")
        continue

    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        failures.append(f"{fixture_path} JSON 非法: {e}")
        print(f"  ❌ {fixture_path} JSON 非法: {e}")
        continue

    scenarios = data.get("scenarios")
    if not isinstance(scenarios, list):
        failures.append(f"{fixture_path} 缺 scenarios 数组")
        print(f"  ❌ {fixture_path} 缺 scenarios 数组")
        continue

    by_id = {}
    for sc in scenarios:
        sid = sc.get("scenario")
        if sid in by_id:
            failures.append(f"{fixture_path} scenario id 重复: {sid}")
            print(f"  ❌ {fixture_path} scenario id 重复: {sid}")
        by_id[sid] = sc

    for sid in expected_ids:
        checks += 1
        if sid not in by_id:
            failures.append(f"{fixture_path} 缺 scenario: {sid}")
            print(f"  ❌ {fixture_path} 缺 scenario {sid}")
            continue

        sc = by_id[sid]
        if not isinstance(sc.get("name"), str) or not sc["name"].strip():
            failures.append(f"{sid} 缺 name")
            print(f"  ❌ {sid} 缺 name")
            continue
        if not isinstance(sc.get("expected"), dict) or not sc["expected"]:
            failures.append(f"{sid} 缺 expected")
            print(f"  ❌ {sid} 缺 expected")
            continue
        assertions = sc.get("assertions")
        if not isinstance(assertions, list) or not assertions:
            failures.append(f"{sid} 缺 assertions")
            print(f"  ❌ {sid} 缺 assertions")
            continue

        ok = True
        for a in assertions:
            checks += 1
            field = a.get("field")
            want = a.get("equals")
            if not field:
                failures.append(f"{sid} 有一条 assertion 缺 field")
                print(f"  ❌ {sid} 有 assertion 缺 field")
                ok = False
                continue
            try:
                got = resolve(sc, field)
            except (KeyError, IndexError, ValueError, TypeError):
                failures.append(f"{sid} assertion 取值失败: {field}")
                print(f"  ❌ {sid} assertion 取值失败: {field}")
                ok = False
                continue
            if got != want:
                failures.append(f"{sid} assertion 不符: {field} 期望 {want!r} 实际 {got!r}")
                print(f"  ❌ {sid} {field} 期望 {want!r} 实际 {got!r}")
                ok = False

        # OPS-UI-003 (env var 永不打印) / OPS-UI-004 (REDACTED filter) 的明文扫描
        if sid in ("OPS-UI-003", "OPS-UI-004"):
            for key_path, value in scan_plaintext_secrets(sc):
                checks += 1
                failures.append(f"{sid} 明文 secret 泄漏: {key_path}")
                print(f"  ❌ {sid} 明文 secret 泄漏: {key_path} = {value[:12]}...")

        if ok:
            print(f"  ✅ {sid} ({len(assertions)} 条 assertion) {sc['name']}")

# 跨 fixture: scenario id 全局唯一
seen = {}
for fixture_path in FIXTURES:
    p = pathlib.Path(fixture_path)
    if not p.exists():
        continue
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        continue
    for sc in data.get("scenarios", []):
        sid = sc.get("scenario")
        if sid in seen:
            failures.append(f"scenario id 全局重复: {sid} ({seen[sid]} vs {fixture_path})")
            print(f"  ❌ scenario id 全局重复: {sid}")
        seen[sid] = fixture_path

total_expected = sum(len(v) for v in FIXTURES.values())
print()
print(f"  断言总数: {checks} / scenario 数: {len(seen)} (期望 {total_expected})")

if failures:
    print(f"  ❌ {len(failures)} 条断言失败")
    for f in failures:
        print(f"     - {f}")
    sys.exit(1)

print("  ✅ app-deploy 4 + ops-ui 6 = 10 scenario 全绿")
sys.exit(0)
PY

RC=$?
echo
if [ "$RC" -eq 0 ]; then
  echo "=== app-deploy + ops-ui 回归自检完成 (10 scenario 全绿) ==="
else
  echo "=== app-deploy + ops-ui 回归自检失败 (rc=$RC) ==="
fi
exit "$RC"
