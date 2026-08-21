#!/usr/bin/env bash
# Cypher structural manifest (executable Cypher; comment-delimited for Bash).
# CREATE
#   (file:File {name: "check-docs-consistency.sh", type: "file", language: "bash"}),
#   (main:Logic {name: "main", type: "logic"}),
#   (formalArc:Logic {name: "formal_arc_declaration_check", type: "logic"}),
#   (adrRegistry:Logic {name: "adr_registry_check", type: "logic"}),
#   (domainIds:Logic {name: "domain_tbd_rsk_check", type: "logic"}),
#   (readmeLinks:Logic {name: "readme_link_check", type: "logic"}),
#   (crossReferences:Logic {name: "cross_reference_check", type: "logic"}),
#   (docsDir:Variable {name: "DOCS_DIR", type: "variable"}),
#   (appendixD:Variable {name: "APPENDIX_D", type: "variable"}),
#   (readme:Variable {name: "README", type: "variable"}),
#   (declaredArcs:Variable {name: "declared_arcs", type: "variable"}),
#   (registeredArcs:Variable {name: "registered_arcs", type: "variable"}),
#   (adrPending:Variable {name: "adr_pending", type: "variable"}),
#   (crossReferencePython:Variable {name: "cross_reference_python", type: "variable"}),
#   (fail:Variable {name: "fail", type: "variable"}),
#   (file)-[:CONTAINS]->(main),
#   (file)-[:CONTAINS]->(formalArc),
#   (file)-[:CONTAINS]->(adrRegistry),
#   (file)-[:CONTAINS]->(domainIds),
#   (file)-[:CONTAINS]->(readmeLinks),
#   (file)-[:CONTAINS]->(crossReferences),
#   (file)-[:CONTAINS]->(docsDir),
#   (file)-[:CONTAINS]->(appendixD),
#   (file)-[:CONTAINS]->(readme),
#   (file)-[:CONTAINS]->(declaredArcs),
#   (file)-[:CONTAINS]->(registeredArcs),
#   (file)-[:CONTAINS]->(adrPending),
#   (file)-[:CONTAINS]->(crossReferencePython),
#   (file)-[:CONTAINS]->(fail),
#   (main)-[:CALLS]->(formalArc),
#   (main)-[:CALLS]->(adrRegistry),
#   (main)-[:CALLS]->(domainIds),
#   (main)-[:CALLS]->(readmeLinks),
#   (main)-[:CALLS]->(crossReferences),
#   (formalArc)-[:USES]->(docsDir),
#   (formalArc)-[:USES]->(declaredArcs),
#   (formalArc)-[:USES]->(fail),
#   (adrRegistry)-[:USES]->(appendixD),
#   (adrRegistry)-[:USES]->(declaredArcs),
#   (adrRegistry)-[:USES]->(registeredArcs),
#   (adrRegistry)-[:USES]->(adrPending),
#   (adrRegistry)-[:USES]->(fail),
#   (domainIds)-[:USES]->(docsDir),
#   (domainIds)-[:USES]->(appendixD),
#   (domainIds)-[:USES]->(fail),
#   (readmeLinks)-[:USES]->(readme),
#   (readmeLinks)-[:USES]->(fail),
#   (crossReferences)-[:USES]->(crossReferencePython),
#   (crossReferences)-[:USES]->(fail);
# END CYPHER
# 文档体系一致性检查（处置RSK-PAT-001／TBD-PAT-002，落地于GitHub Actions）
#
# 检查项：
#   1. 从需求定义书的标题提取正式ARC声明；测试样例和普通正文引用不构成声明
#   2. 每个ARC-018及以后的正式声明均在附件D§3有ADR登记行（**仅登记行，不校验ADR是否已制定**；
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
README="docs/README.md"

fail=0

echo "=== 1. 正式ARC声明来源检查 ==="
# 只读取RGS-REQ正文中的标题。这样ARC-099等测试输入不会被误判为架构方针；
# ARC编号不是连续序列，保留编号须在附件D的编号管理说明中显式声明。
mapfile -t declared_arcs < <(
  grep -rhE --include='RGS-REQ-*.md' \
    '^(#{1,6}[[:space:]].*(架构设计方针|架构方针).*ARC-[0-9]{3}|#{2,6}[[:space:]][0-9]+(\.[0-9]+)*[[:space:]]+ARC-[0-9]{3}[：:])' \
    "$DOCS_DIR" 2>/dev/null \
    | grep -oE 'ARC-[0-9]{3}([^0-9-]|$)' \
    | grep -oE 'ARC-[0-9]{3}' \
    | sort -u
)
if [ "${#declared_arcs[@]}" -eq 0 ]; then
  echo "  [FAIL] 未检出任何正式ARC声明"
  fail=1
else
  echo "  已提取 ${#declared_arcs[@]} 项正式声明：${declared_arcs[*]}"
fi

echo "=== 2. ADR**登记**完整性检查（ARC-018起，ADR-025治理框架落地对象） ==="
# 注意：本项只校验"附件D§3中是否存在对应行"，**不**校验该ADR是否真的写出来了。
# 二者是不同的事——见本项末尾输出的制定进度统计。历史上本项曾在44/44 ADR全部
# "未制定"的情况下报告"全部通过"，属于给出虚假保证，2026-08-17修正为同时输出进度。
if [ -f "$APPENDIX_D" ]; then
  mapfile -t registered_arcs < <(
    awk -F '|' '$2 ~ /^ ADR-/ { print $4 }' "$APPENDIX_D" \
      | grep -oE 'ARC-[0-9]{3}([^0-9-]|$)' \
      | grep -oE 'ARC-[0-9]{3}' \
      | sort -u
  )
  for arc in "${declared_arcs[@]}"; do
    arc_number="${arc#ARC-}"
    if (( 10#$arc_number >= 18 )); then
      if ! printf '%s\n' "${registered_arcs[@]}" | grep -Fxq "$arc"; then
        echo "  [FAIL] $arc 未在附件D §3找到对应ADR记录行"
        fail=1
      fi
    fi
  done
  echo "  已核对全部正式声明中ARC-018及以后的ADR登记行存在性"

  # 制定进度：登记 ≠ 制定。此处如实统计，不并入fail（未制定是已知待办，非一致性缺陷），
  # 但必须显式打印，避免"全部检查通过"被误读为"ADR体系已完备"。
  # 2026-08-17起ADR分四态：已制定 / 决策显然·不单独立ADR / 待具名人类审批 / 未制定（见附件D§3政策说明）。
  adr_total=$(grep -cE "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" || true)
  adr_done=$(grep -E "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" | grep -c "已制定" || true)
  adr_obvious=$(grep -E "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" | grep -c "决策显然" || true)
  adr_pending=$(grep -E "^\| ADR-[0-9]{4} \|" "$APPENDIX_D" | grep -c "待具名人类审批" || true)
  adr_pending_undrafted=$(awk '/^\| ADR-[0-9]{4} \|/ && /待具名人类审批/ && $0 !~ /已制定/ { count++ } END { print count + 0 }' "$APPENDIX_D")
  adr_drafted_pending=$((adr_pending - adr_pending_undrafted))
  adr_todo=$((adr_total - adr_done - adr_obvious - adr_pending_undrafted))
  echo "  [进度] ADR：已制定 ${adr_done} ／ 决策显然不立ADR ${adr_obvious} ／ 待具名人类审批（已制定 ${adr_drafted_pending}／未制定 ${adr_pending_undrafted}）／ 未处理 ${adr_todo}（合计 ${adr_total}）"
  # 已制定的ADR必须真的存在对应文件，否则状态列在说谎
  adr_missing=0
  while IFS= read -r rel; do
    [ -e "docs/$rel" ] || { echo "  [FAIL] 附件D§3标记为已制定，但文件不存在：$rel"; adr_missing=1; fail=1; }
  done < <(grep -oE "\(\.\./08-[^)]+\.md\)" "$APPENDIX_D" | sed -E 's/^\(\.\.\/(.*)\)$/\1/' | sort -u)
  [ "$adr_missing" -eq 0 ] && echo "  [OK] 标记为已制定的ADR均有对应文件"
  if [ "$adr_pending" -gt 0 ]; then
    echo "  [WARN] 仍有 ${adr_pending} 项ADR待具名人类审批；不得作为生产基线。"
  fi
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
    # Markdown 目标可以带标题、片段或查询串；文件存在性只校验路径本身。
    link="${link%% *}"
    link="${link%%#*}"
    link="${link%%\?*}"
    [ -z "$link" ] && continue

    # 跳过外部链接与纯锚点。
    case "$link" in
      *://*|mailto:*|tel:*|\#*) continue ;;
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
  cross_reference_python="python3"
elif command -v python >/dev/null 2>&1; then
  cross_reference_python="python"
else
  echo "  [FAIL] 未找到可执行的 python3 或 python，无法跳过严格跨文档引用检查"
  fail=1
fi
if [ -n "${cross_reference_python:-}" ]; then
  if ! "$cross_reference_python" scripts/check-cross-references.py; then
    fail=1
  fi
fi

echo ""
if [ "$fail" -eq 0 ]; then
  echo "全部检查通过。"
  echo "（注意：本脚本校验的是文档间的机械一致性，不校验设计内容本身的正确性，"
  echo "  也不代表文档或决策已获批准——须以各文档状态及具名审批记录为准。）"
else
  echo "存在问题，见上方[FAIL]标记。"
fi
exit "$fail"
