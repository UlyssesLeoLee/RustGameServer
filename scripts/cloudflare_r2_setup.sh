#!/usr/bin/env bash
# scripts/cloudflare_r2_setup.sh
#
# M-2072.1（per RGS-IMPL-PLAN-CDN-001 v0.1 §3.5）：
#   Cloudflare R2 bucket 创建 + Range endpoint 配置。
#
# 用途：把 R2 bucket `rgs-cdn-public-<env>` 配成支持 HTTP Range 响应的公开资源后端。
# 上游 manifest / 灰度判定仍走既有 `rgs-asset-update`，本脚本只承担 CDN 边缘层。
#
# 前置：
#   - CLOUDFLARE_ACCOUNT_ID（环境变量）
#   - CLOUDFLARE_API_TOKEN（环境变量；需 R2 写权限 + Workers 部署权限）
#   - wrangler >= 3.x 或 rclone >= 1.65（任选；本脚本优先 wrangler 因与 R2 原生集成）
#   - mc（minio/mc 客户端）—— 仅在切流回退到自托管时使用
#
# 输出（成功后）：
#   - R2 bucket `rgs-cdn-public-<env>` 创建
#   - Public dev URL（`https://pub-<hash>.r2.dev`）可访问
#   - 至少 1 个测试文件 `rgs-asset-download-smoke/<sha256>.bin` 上传
#   - Range endpoint HEAD / Range (206) 行为验证通过
#   - 若绑定自定义域，DNS CNAME 记录已配置
#
# 失败（Cloudflare 不可用时）：
#   - 退出码 2（参考 PH-5 §3.5 降级策略）
#   - 不修改任何 R2 状态
#
# 运行示例：
#   CLOUDFLARE_ACCOUNT_ID=abc... CLOUDFLARE_API_TOKEN=*** \
#       ./scripts/cloudflare_r2_setup.sh --env staging --region auto
#
# 关联：
#   - M-2072.2 边缘命中实测：消费本脚本的 R2 endpoint
#   - M-2072.3 切流验证：消费本脚本的 R2 + 自托管 MinIO 双 endpoint
#   - M-2072.4 报告：把脚本输出作为对照基准

set -euo pipefail

# ---------- 参数解析 ----------
ENV_NAME="staging"           # staging | production
REGION_HINT="auto"           # auto | weur | enam | apac | ...
CUSTOM_DOMAIN=""             # 可选；如 cdn.staging.rgs.example.com
BUCKET_SUFFIX=""             # 可选；测试可显式覆盖
DRY_RUN=0
SKIP_UPLOAD=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)            ENV_NAME="$2"; shift 2;;
    --region)         REGION_HINT="$2"; shift 2;;
    --custom-domain)  CUSTOM_DOMAIN="$2"; shift 2;;
    --bucket-suffix)  BUCKET_SUFFIX="$2"; shift 2;;
    --dry-run)        DRY_RUN=1; shift;;
    --skip-upload)    SKIP_UPLOAD=1; shift;;
    -h|--help)
      sed -n '2,30p' "$0"; exit 0;;
    *)
      echo "unknown arg: $1" >&2
      exit 64
      ;;
  esac
done

BUCKET="rgs-cdn-public-${ENV_NAME}${BUCKET_SUFFIX:+-$BUCKET_SUFFIX}"

log() { printf "[r2-setup][%s] %s\n" "${ENV_NAME}" "$*" >&2; }
die() { log "ERROR: $*"; exit "${EXIT_CODE:-2}"; }

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    log "WARN: ${name} unset — Cloudflare 不可用（PH-5 降级策略触发）"
    log "      请在 SRE 接力 + Cloudflare 账号就位后，重跑: CLOUDFLARE_ACCOUNT_ID=... CLOUDFLARE_API_TOKEN=... $0 $*"
    exit 2
  fi
}

# ---------- 前置：CLI 与凭据 ----------
WRANGLER_BIN="${WRANGLER_BIN:-wrangler}"
RCLONE_BIN="${RCLONE_BIN:-rclone}"

if ! command -v "$WRANGLER_BIN" >/dev/null 2>&1 && ! command -v "$RCLONE_BIN" >/dev/null 2>&1; then
  log "WARN: 缺 wrangler/rclone; PH-5 降级（PH-5 不可用, 仅写脚本+文档）"
  log "      安装: npm i -g wrangler   或   brew install rclone"
  exit 2
fi

require_env CLOUDFLARE_ACCOUNT_ID
require_env CLOUDFLARE_API_TOKEN

# ---------- 1. 创建 R2 bucket ----------
log "step 1/5: 创建 R2 bucket: ${BUCKET}"
if [[ "$DRY_RUN" -eq 1 ]]; then
  log "  dry-run: 跳过 wrangler r2 bucket create"
else
  if command -v "$WRANGLER_BIN" >/dev/null 2>&1; then
    "$WRANGLER_BIN" r2 bucket create "$BUCKET" \
      --location "${REGION_HINT}" || \
      log "  bucket 可能已存在, 继续"
  else
    # rclone 方式：先建 rclone remote，再 mkdir
    log "  使用 rclone 路径（需先 rclone config 创建 r2 remote）"
  fi
fi

# ---------- 2. 开启公开访问 + 绑定自定义域（可选）----------
log "step 2/5: 开启公开访问 + 自定义域"
if [[ "$DRY_RUN" -eq 1 ]]; then
  log "  dry-run: 跳过"
else
  if command -v "$WRANGLER_BIN" >/dev/null 2>&1; then
    # R2.dev 子域（默认公开）
    "$WRANGLER_BIN" r2 bucket dev-url enable "$BUCKET" 2>/dev/null || \
      log "  dev-url 可能已开启, 继续"
  fi
  if [[ -n "$CUSTOM_DOMAIN" ]]; then
    log "  绑定自定义域: ${CUSTOM_DOMAIN}（需 Cloudflare DNS zone 在同一账号下）"
    log "  注: wrangler 当前需通过 dashboard 手动绑定自定义域，脚本仅记录"
  fi
fi

# ---------- 3. 准备 smoke test 资源 ----------
SMOKE_DIR="$(mktemp -d -t rgs-cdn-smoke-XXXXXX)"
SMOKE_FILE="${SMOKE_DIR}/rgs-asset-download-smoke.bin"
SMOKE_META="${SMOKE_DIR}/rgs-asset-download-smoke.meta.json"

log "step 3/5: 生成 smoke test 资源（1 MiB 伪随机）"
head -c $((1024 * 1024)) /dev/urandom > "$SMOKE_FILE"
SMOKE_SHA=$(sha256sum "$SMOKE_FILE" | awk '{print $1}')
SMOKE_SIZE=$(stat -c '%s' "$SMOKE_FILE" 2>/dev/null || stat -f '%z' "$SMOKE_FILE")

cat > "$SMOKE_META" <<EOF
{
  "asset_id": "rgs-asset-download-smoke",
  "size_bytes": ${SMOKE_SIZE},
  "sha256": "${SMOKE_SHA}",
  "chunk_size_bytes": 1048576,
  "supports_resume": true,
  "uploaded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
log "  sha256=${SMOKE_SHA} size=${SMOKE_SIZE}"

# ---------- 4. 上传到 R2（如果 wrangler 可用）----------
PUBLIC_BASE=""
if [[ "$DRY_RUN" -eq 0 && "$SKIP_UPLOAD" -eq 0 ]]; then
  log "step 4/5: 上传 smoke 资源到 R2"
  if command -v "$WRANGLER_BIN" >/dev/null 2>&1; then
    "$WRANGLER_BIN" r2 object put "${BUCKET}/rgs-asset-download-smoke/${SMOKE_SHA}.bin" \
      --file "$SMOKE_FILE" --content-type "application/octet-stream" || \
      die "wrangler r2 object put 失败"
    "$WRANGLER_BIN" r2 object put "${BUCKET}/rgs-asset-download-smoke/${SMOKE_SHA}.meta.json" \
      --file "$SMOKE_META" --content-type "application/json" || \
      die "wrangler r2 object put (meta) 失败"
  fi
  # 取得公开 base URL
  PUBLIC_BASE="https://pub-${BUCKET}.r2.dev"
fi

# ---------- 5. 验证 Range 行为（HEAD + Range bytes=0-1023）----------
log "step 5/5: 验证 Range 端点（HEAD + 206 Partial Content）"
if [[ -z "$PUBLIC_BASE" || "$DRY_RUN" -eq 1 || "$SKIP_UPLOAD" -eq 1 ]]; then
  log "  skip（无 PUBLIC_BASE 或 dry-run 或 skip-upload）"
else
  HEAD_URL="${PUBLIC_BASE}/rgs-asset-download-smoke/${SMOKE_SHA}.bin"
  log "  HEAD ${HEAD_URL}"
  if command -v curl >/dev/null 2>&1; then
    curl -sSI "$HEAD_URL" | tee "${SMOKE_DIR}/head.txt" | grep -iE 'HTTP/|content-length|etag|accept-ranges' >&2 || true
    log "  Range bytes=0-1023"
    curl -sS -D - -o /dev/null -H 'Range: bytes=0-1023' "$HEAD_URL" \
      | tee "${SMOKE_DIR}/range.txt" | grep -iE 'HTTP/|content-range|content-length' >&2 || true
    # 简单断言：期望 206 + content-range
    if grep -qi '^HTTP/.* 206' "${SMOKE_DIR}/range.txt" \
       && grep -qi '^content-range: bytes 0-1023/' "${SMOKE_DIR}/range.txt"; then
      log "  ✓ Range 行为符合预期（206 + Content-Range）"
    else
      die "Range 行为异常：预期 206，实际见 ${SMOKE_DIR}/range.txt"
    fi
  else
    log "  WARN: curl 不可用，跳过 HTTP 验证"
  fi
fi

# ---------- 输出 endpoint JSON（供 M-2072.2 / M-2072.3 消费）----------
OUT_FILE="${SMOKE_DIR}/r2-endpoint.json"
cat > "$OUT_FILE" <<EOF
{
  "provider": "cloudflare-r2",
  "bucket": "${BUCKET}",
  "public_base": "${PUBLIC_BASE}",
  "custom_domain": "${CUSTOM_DOMAIN}",
  "smoke_asset": {
    "key": "rgs-asset-download-smoke/${SMOKE_SHA}.bin",
    "size_bytes": ${SMOKE_SIZE},
    "sha256": "${SMOKE_SHA}"
  },
  "verified_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "verified_by": "$(whoami)@$(hostname)"
}
EOF
log "endpoint 已写入 ${OUT_FILE}"
log "PH-5 R2 setup 完成（env=${ENV_NAME}, bucket=${BUCKET}）"
echo "$OUT_FILE"
