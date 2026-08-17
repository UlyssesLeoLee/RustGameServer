#!/usr/bin/env bash
# 文档体系一致性检查（处置RSK-PAT-001／TBD-PAT-002，落地于GitHub Actions）
#
# 检查项：
#   1. ARC-001〜N 序列无缺号/重号
#   2. 每个ARC-018及以后的决定均在附件D§3有ADR登记行（**仅登记行，不校验ADR是否已制定**；
#      另输出ADR实际制定进度，避免"全部通过"被误读为ADR体系完备）
#   3. 领域文档正文中出现的 TBD-<域>-nnn／RSK-<域>-nnn 均已在附件D登记主编号
#   4. docs/README.md 中的相对路径链接均指向实际存在的文件
#   5. 跨文档引用有效性（文档编号/章节号/SQL列名/proto字段编号），见 check-cross-references.py
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

echo "=== 2. ADR**登记**完整性检查（ARC-018起，ADR-025治理框架落地对象） ==="
# 注意：本项只校验"附件D§3中是否存在对应行"，**不**校验该ADR是否真的写出来了。
# 二者是不同的事——见本项末尾输出的制定进度统计。历史上本项曾在44/44 ADR全部
# "未制定"的情况下报告"全部通过"，属于给出虚假保证，2026-08-17修正为同时输出进度。
if [ -f "$APPENDIX_D" ]; then
  for n in $(seq 18 "$max_arc"); do
    padded=$(printf "%03d" "$n")
    if grep -qE "ARC-$padded\b" "$DOCS_DIR"/*/*.md 2>/dev/null; then
      if ! grep -qE "\| ARC-$padded \|" "$APPENDIX_D"; then
        echo "  [FAIL] ARC-$padded 未在附件D §3找到对应ADR记录行"
        fail=1
      fi
    fi
  done
  echo "  已核对 ARC-018〜$(printf '%03d' "$max_arc") 的ADR登记行存在性"

  # 制定进度：登记 ≠ 制定。此处如实统计，不并入fail（未制定是已知待办，非一致性缺陷），
  # 但必须显式打印，避免"全部检查通过"被误读为"ADR体系已完备"。
  # 2026-08-17起ADR分三态：已制定 / 决策显然·不单独立ADR / 未制定（见附件D§3政策说明）。
  adr_total=$(grep -cE "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" || true)
  adr_done=$(grep -E "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" | grep -c "已制定" || true)
  adr_obvious=$(grep -E "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" | grep -c "决策显然" || true)
  adr_todo=$((adr_total - adr_done - adr_obvious))
  echo "  [进度] ADR：已制定 ${adr_done} ／ 决策显然不立ADR ${adr_obvious} ／ 未处理 ${adr_todo}（合计 ${adr_total}）"
  # 已制定的ADR必须真的存在对应文件，否则状态列在说谎
  adr_missing=0
  while IFS= read -r rel; do
    [ -e "docs/$rel" ] || { echo "  [FAIL] 附件D§3标记为已制定，但文件不存在：$rel"; adr_missing=1; fail=1; }
  done < <(grep -oE "\(\.\./08-[^)]+\.md\)" "$APPENDIX_D" | sed -E 's/^\(\.\.\/(.*)\)$/\1/' | sort -u)
  [ "$adr_missing" -eq 0 ] && echo "  [OK] 标记为已制定的ADR均有对应文件"
  if [ "$adr_todo" -gt 0 ]; then
    echo "  [WARN] 仍有 ${adr_todo} 项ARC既未制定ADR、也未标注决策显然，属未处理状态。"
  fi
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

echo "=== 5. 跨文档引用有效性检查（文档编号/章节号/SQL列名/proto字段编号） ==="
if command -v python3 >/dev/null 2>&1; then
  if ! python3 scripts/check-cross-references.py; then
    fail=1
  fi
else
  echo "  [跳过] 未找到python3"
fi

echo ""
if [ "$fail" -eq 0 ]; then
  echo "全部检查通过。"
  echo "（注意：本脚本校验的是文档间的机械一致性，不校验设计内容本身的正确性，"
  echo "  也不代表文档已获批准——截至目前全部文档仍为0.x待评审状态。）"
else
  echo "存在问题，见上方[FAIL]标记。"
fi
exit "$fail"
