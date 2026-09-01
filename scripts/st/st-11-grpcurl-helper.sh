#!/usr/bin/env bash
# st-11 grpcurl mTLS helper
# Args: $1=caPem $2=clientPem $3=clientKey $4=serverName $5=commonDir $6=playerDir
#       $7=commonProto $8=playerProto $9=podIP $10=methodName

set -e
CA_PEM="$1"
CLIENT_PEM="$2"
CLIENT_KEY="$3"
SERVER_NAME="$4"
COMMON_DIR="$5"
PLAYER_DIR="$6"
COMMON_PROTO="$7"
PLAYER_PROTO="$8"
POD_IP="$9"
METHOD="${10}"

export PATH=~/.local/bin:$PATH

# 构造 request body 文件
echo '{"request_id":"st-11-2026-09-01"}' > /tmp/rgs-mtls/req.json

grpcurl \
  -cacert "$CA_PEM" \
  -cert "$CLIENT_PEM" \
  -key "$CLIENT_KEY" \
  -servername "$SERVER_NAME" \
  -import-path "$COMMON_DIR" \
  -import-path "$PLAYER_DIR" \
  -proto "$COMMON_PROTO" \
  -proto "$PLAYER_PROTO" \
  -d '{"service":"player"}' \
  "${POD_IP}:50051" \
  "$METHOD" 2>&1
