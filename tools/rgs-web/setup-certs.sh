#!/bin/bash
# setup-certs.sh — 在 rgs-web 工作目录生成 / 拉取 mTLS 客户端证书
#
# 用法:
#   1) 第一次设置(CA 私钥丢失时):用 rgs-certgen 重新生成 CA + 6 域 server cert
#      + 签发 rgs-web client cert,更新 k3s secret,重启 pods
#   2) 增量更新(只 rgs-web client cert 丢失):用现有 CA 签发新 client cert
#
# 假设:
#   - rgs-certgen.exe 在 E:/DevCache/cargo/target/debug/(per phase-0-5-step-4-gen-certs.ps1)
#   - kubectl 在 WSL 内,WSL user = leo19,kubeconfig 在 ~/.kube/config
#   - rgs-web 目录在 D:/RustGameServer-worktrees/<branch>/tools/rgs-web/
set -e

WORKTREE_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
CERTS_DIR="$WORKTREE_DIR/tools/rgs-web"
WSL_NEWC=/tmp/rgs-new-certs
NAMESPACE=rust-game-server
CERTGEN_WIN='E:\DevCache\cargo\target\debug\rgs-certgen.exe'

echo "=== rgs-web cert setup ==="
echo "WORKTREE_DIR: $WORKTREE_DIR"
echo "CERTS_DIR:    $CERTS_DIR"

# 在 WSL 内准备 rgs-new-certs 目录
wsl -e bash -c "mkdir -p $WSL_NEWC"

# Step 1: 重新生成 CA + 6 域 server cert(幂等,会覆盖)
echo ""
echo "=== Step 1: 重新生成 CA + 6 域 server cert ==="
wsl -e bash -c "/mnt/e/DevCache/cargo/target/debug/rgs-certgen.exe --output $WSL_NEWC --validity-days 730"

# Step 2: 签发 rgs-web client cert
echo ""
echo "=== Step 2: 签发 rgs-web client cert (CN=rgs-web-client, EKU=clientAuth) ==="
wsl -e bash -c "
cd $WSL_NEWC
# 写 v3 ext config
cat > client-ext.cnf <<EOF
[req]
distinguished_name = req_dn
prompt = no
[req_dn]
CN = rgs-web-client
O  = RustGameServer
[v3_client]
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
subjectAltName = DNS:rgs-web-client
EOF

# 拿 CA key(若已存在)+ 重生成
test -f ca.key.pem || { echo 'CA key not found at \$WSL_NEWC/ca.key.pem — rgs-certgen output 异常'; exit 1; }

# 签发
openssl req -new -key client.key.pem -out client.csr.pem -subj '/CN=rgs-web-client/O=RustGameServer'
openssl x509 -req -in client.csr.pem -CA ca.crt.pem -CAkey ca.key.pem -CAcreateserial -out client.crt.pem -days 730 -sha256 -extfile client-ext.cnf -extensions v3_client

# 验证
openssl verify -CAfile ca.crt.pem client.crt.pem
echo 'client cert signed OK'
"

# Step 3: 更新 k3s 7 个 secret(CA + 6 域 server tls)
echo ""
echo "=== Step 3: 更新 k3s 7 个 secret ==="
wsl -e bash -c "
export KUBECONFIG=\$HOME/.kube/config
NS=$NAMESPACE
# CA
k3s kubectl create secret generic rgs-secret-ca -n \$NS --from-file=ca.pem=$WSL_NEWC/ca.crt.pem --dry-run=client -o yaml | k3s kubectl apply -f -
# 6 域 server tls
for name in player economy match social admin; do
  k3s kubectl create secret tls rgs-secret-\${name}-tls -n \$NS --cert=$WSL_NEWC/\${name}.service.crt.pem --key=$WSL_NEWC/\${name}.service.key.pem --dry-run=client -o yaml | k3s kubectl apply -f -
done
# cluster-ops
k3s kubectl create secret tls rgs-secret-cluster-ops-tls -n \$NS --cert=$WSL_NEWC/cluster-ops.service.crt.pem --key=$WSL_NEWC/cluster-ops.service.key.pem --dry-run=client -o yaml | k3s kubectl apply -f -

# 验证
echo '--- 验证 CA 是否更新 ---'
k3s kubectl get secret -n \$NS rgs-secret-ca -o jsonpath='{.data.ca\.pem}' | base64 -d | openssl x509 -noout -fingerprint -sha256
"

# Step 4: 重启 6 域 deployment
echo ""
echo "=== Step 4: 重启 6 域 deployment (per k3s rollout restart) ==="
wsl -e bash -c "
export KUBECONFIG=\$HOME/.kube/config
NS=$NAMESPACE
for svc in player-service economy-service match-service social-service admin-service cluster-ops; do
  k3s kubectl rollout restart deployment/\$svc -n \$NS 2>&1 | head -1
done
echo 'wait for rollout...'
for svc in player-service economy-service match-service social-service admin-service cluster-ops; do
  k3s kubectl rollout status deployment/\$svc -n \$NS --timeout=60s 2>&1 | tail -1
done
"

# Step 5: 复制 cert 到 rgs-web 目录
echo ""
echo "=== Step 5: 复制 cert 到 rgs-web 目录 ==="
wsl -e bash -c "cat $WSL_NEWC/ca.crt.pem"      > "$CERTS_DIR/rgs-ca.pem"
wsl -e bash -c "cat $WSL_NEWC/client.crt.pem"  > "$CERTS_DIR/rgs-client.crt.pem"
wsl -e bash -c "cat $WSL_NEWC/client.key.pem"  > "$CERTS_DIR/rgs-client.key.pem"

ls -la "$CERTS_DIR/rgs-"*.pem

# Step 6: 重启 port-forward
echo ""
echo "=== Step 6: 重启 kubectl port-forward (per .pf-start.sh) ==="
wsl -e bash -c "bash $WORKTREE_DIR/.pf-start.sh" 2>&1 | tail -3 || wsl -e bash -c "bash /mnt/d/RustGameServer/.pf-start.sh" 2>&1 | tail -3

echo ""
echo "=== 验证(Test-NetConnection 应全 True) ==="
echo "Test-NetConnection 127.0.0.1 -Port 15051 # player-service gRPC"
echo "Test-NetConnection 127.0.0.1 -Port 15056 # cluster-ops gRPC"
echo ""
echo "=== 启动 rgs-web ==="
echo "cd $CERTS_DIR && node server.js"
echo "open http://127.0.0.1:8788"
