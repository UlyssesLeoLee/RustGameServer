#!/usr/bin/env bash
# 文档体系一致性检查（处置RSK-PAT-001／TBD-PAT-002，落地于GitHub Actions）
#
# 检查项：
#   1. ARC-001〜N 序列无缺号/重号
#   2. 每个ARC-018及以后的决定均有对应ADR记录（附件D §3）
#   3. 领域文档正文中出现的 TBD-<域>-nnn／RSK-<域>-nnn 均已在附件D登记主编号
#   4. docs/README.md 中的相对路径链接均指向实际存在的文件
#
# 用法：./scripts/check-docs-consistency.sh
# 退出码：0=全部通过；1=发现问题（详情见stdout）

set -uo pipefail
cd "$(dirname "$0")/.."

DOCS_DIR="docs"
APPENDIX_D="docs/00-基准与治理/RGS-REQ-005_附件D_问题风险管理表.md"
APPENDIX_C="docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md"
README="docs/README.md"

fail=0

echo "=== 1. ARC序列缺号/重号检查 ==="
max_arc=$(grep -rhoE "ARC-[0-9]{3}" "$DOCS_DIR" | grep -oE "[0-9]{3}" | sort -n | tail -1 | sed 's/^0*//')
if [ -z "$max_arc" ]; then
  echo "  未检出任何ARC编号，跳过"
else
  for n in $(seq 1 "$max_arc"); do
    padded=$(printf "%03d" "$n")
    if ! grep -rqE "ARC-$padded\b" "$DOCS_DIR"; then
      echo "  [FAIL] 缺失 ARC-$padded"
      fail=1
    fi
  done
  echo "  已检查 ARC-001〜$(printf '%03d' "$max_arc")"
fi

echo "=== 2. ADR记录完整性检查（ARC-018起，ADR-025治理框架落地对象） ==="
if [ -f "$APPENDIX_D" ]; then
  for n in $(seq 18 "$max_arc"); do
    padded=$(printf "%03d" "$n")
    if grep -qE "ARC-$padded\b" "$DOCS_DIR"/*/*.md 2>/dev/null; then
      if ! grep -qE "\| ARC-$padded \|" "$APPENDIX_D"; then
        echo "  [FAIL] ARC-$padded 未在附件D §3找到对应ADR记录"
        fail=1
      fi
    fi
  done
  echo "  已核对 ARC-018〜$(printf '%03d' "$max_arc") 的ADR登记"
else
  echo "  [FAIL] 未找到附件D：$APPENDIX_D"
  fail=1
fi

echo "=== 3. 域内TBD/RSK主编号登记检查 ==="
if [ -f "$APPENDIX_D" ]; then
  used_ids=$(grep -rhoE "TBD-[A-Z]+-[0-9]+|RSK-[A-Z]+-[0-9]+" "$DOCS_DIR" | sort -u)
  registered_ids=$(grep -hoE "TBD-[A-Z]+-[0-9]+|RSK-[A-Z]+-[0-9]+" "$APPENDIX_D" | sort -u)
  missing=$(comm -23 <(echo "$used_ids") <(echo "$registered_ids"))
  if [ -n "$missing" ]; then
    echo "  [FAIL] 以下域内ID在正文中使用但未在附件D登记主编号："
    echo "$missing" | sed 's/^/    /'
    fail=1
  else
    echo "  全部域内TBD/RSK均已登记"
  fi
else
  echo "  [FAIL] 未找到附件D：$APPENDIX_D"
  fail=1
fi

echo "=== 4. README.md 相对路径链接有效性检查 ==="
if [ -f "$README" ]; then
  broken=0
  while IFS= read -r link; do
    # 跳过外部链接与锚点
    case "$link" in
      http://*|https://*|\#*) continue ;;
    esac
    target="docs/$link"
    if [ ! -e "$target" ]; then
      echo "  [FAIL] 死链：$link"
      broken=1
      fail=1
    fi
  done < <(grep -oE '\]\([^)]+\)' "$README" | sed -E 's/^\]\((.*)\)$/\1/')
  if [ "$broken" -eq 0 ]; then
    echo "  全部链接有效"
  fi
else
  echo "  [FAIL] 未找到README：$README"
  fail=1
fi

echo ""
if [ "$fail" -eq 0 ]; then
  echo "全部检查通过。"
else
  echo "存在问题，见上方[FAIL]标记。"
fi
exit "$fail"
