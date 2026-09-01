#!/bin/bash
# Extract mTLS certs from k3s secrets to ST worktree certs/
# Per AGENTS.md §2.5 L4 ST checklist
set -u

CERT_DIR="/mnt/d/rgs-st-mock/certs"
NAMESPACE="rust-game-server"
KUBECONFIG_PATH="/etc/rancher/k3s/k3s.yaml"

# Ensure 644 (k3s creates 600 on token rotation)
sudo chmod 644 "$KUBECONFIG_PATH" 2>/dev/null

mkdir -p "$CERT_DIR"

for d in player economy match social admin; do
  echo "[$d] extracting rgs-secret-${d}-tls"
  KUBECONFIG="$KUBECONFIG_PATH" kubectl get secret "rgs-secret-${d}-tls" -n "$NAMESPACE" -o yaml > "$CERT_DIR/${d}-tls.yaml"
  ls -la "$CERT_DIR/${d}-tls.yaml"
done

echo "---"
ls -la "$CERT_DIR"
