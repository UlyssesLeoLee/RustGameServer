#!/bin/bash
# rgs-batch 证书生成脚本 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-4, 2026-09-02 02:05 JST Mavis 接手代签)
#
# 用 crates/rgs-certgen 生成 rgs-batch 边缘 TLS 证书 + 5 域 mTLS 业务级证书
# 凭据永不打印 (per 8/27 11:06 JST 硬 ban)
#
# 使用: bash scripts/gen-certs.sh
# 依赖: 5 域 ST 证书 (per 8/27 ST 实践 commit 401ac5c) 已导出到 D:/rgs-st-mock/certs/
#       crates/rgs-certgen (workspace member)

set -euo pipefail

# 路径
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CERTS_DIR="$PROJECT_ROOT/certs/rgs-batch"
FIVE_DOMAIN_CERTS="${FIVE_DOMAIN_CERTS:-D:/rgs-st-mock/certs}"

mkdir -p "$CERTS_DIR"

# 1. 5 域 mTLS 业务级证书 (per 5 域 ST 实践 commit 401ac5c + 8/27 ST 导出 SOP)
#    从 ST worktree 已导出证书复制 (per WBS v0.2 §2.4 派生约束)
if [ ! -d "$FIVE_DOMAIN_CERTS" ]; then
    echo "[ERROR] 5 域 ST 证书未找到: $FIVE_DOMAIN_CERTS"
    echo "  请先在 ST worktree 导出: kubectl get secret {player,economy,match,social,admin}-tls -n rust-game-server -o yaml > certs/"
    exit 1
fi

cp -r "$FIVE_DOMAIN_CERTS"/*.{crt,key} "$CERTS_DIR/" 2>/dev/null || true
echo "[OK] 5 域 mTLS 证书已复制 (凭据未打印)"

# 2. rgs-batch 边缘 TLS 证书 (envoy mTLS termination)
#    用 crates/rgs-certgen 生成 (per WBS v0.2 §2.4 派生约束)
cd "$PROJECT_ROOT"
cargo run -p rgs-certgen --bin gen-certs -- \
    --service rgs-batch \
    --out-dir "$CERTS_DIR" \
    --cn "rgs-batch.local" \
    --san "rgs-batch,rgs-batch-console,rgs-batch-backend,rgs-batch-envoy" \
    --san-ip "127.0.0.1,10.43.0.1" 2>&1 | grep -v "Compiling\|Downloaded\|Checking" || true

echo "[OK] rgs-batch 边缘 TLS 证书已生成 (凭据未打印)"
echo "[DONE] certs 目录: $CERTS_DIR"
echo "[INFO] 凭据详情由 sre-inject-secrets.sh 注入到 k8s secret, 不打印"
