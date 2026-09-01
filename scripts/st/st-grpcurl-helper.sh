#!/usr/bin/env bash
# st-11/12/13/14/15/16 grpcurl mTLS helper
# Args: $1=caPem $2=clientPem $3=clientKey $4=serverName $5=commonDir $6=domainDir
#       $7=commonProto $8=domainProto $9=podIP $10=methodName $11=port

set -e
CA_PEM="$1"
CLIENT_PEM="$2"
CLIENT_KEY="$3"
SERVER_NAME="$4"
COMMON_DIR="$5"
DOMAIN_DIR="$6"
COMMON_PROTO="$7"
DOMAIN_PROTO="$8"
POD_IP="$9"
METHOD="${10}"
PORT="${11:-50051}"

export PATH=~/.local/bin:$PATH

# 构造 request body 文件
echo '{"service":"'$SERVER_NAME'"}' > /tmp/rgs-mtls/req.json

grpcurl \
  -cacert "$CA_PEM" \
  -cert "$CLIENT_PEM" \
  -key "$CLIENT_KEY" \
  -servername "$SERVER_NAME" \
  -import-path "$COMMON_DIR" \
  -import-path "$DOMAIN_DIR" \
  -proto "$COMMON_PROTO" \
  -proto "$DOMAIN_PROTO" \
  -d @ \
  "${POD_IP}:${PORT}" \
  "$METHOD" < /tmp/rgs-mtls/req.json 2>&1
