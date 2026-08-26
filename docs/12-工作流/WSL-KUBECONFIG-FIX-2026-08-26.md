# WSL2 KUBECONFIG 权限修复 SOP (2026-08-26)

**问题**: k3s API server 在 6443 端口正常运行(curl /healthz 返回 401 Unauthorized = API up),但当前 WSL user (leo19) 无法读 `/etc/rancher/k3s/k3s.yaml`(root 600),所有 `kubectl get` 都失败。

**解决**: 在 WSL Ubuntu terminal 内手动执行 2 条命令(Mavis agent 无法提权,需 Ulysses 本人跑):

## 步骤

### 1. 打开 WSL Ubuntu terminal

- Windows 搜索 "Ubuntu" → 启动 WSL
- 或 PowerShell: `wsl -d Ubuntu`

### 2. 执行修复(2 条命令)

```bash
# 加 docker 组(持续方案 — leo19 已在 docker 组,先确认)
id

# 改 k3s.yaml 权限(需要 sudo 密码)
sudo chmod 644 /etc/rancher/k3s/k3s.yaml
sudo systemctl daemon-reload  # 不必要,chmod 立即生效

# 验证
ls -la /etc/rancher/k3s/k3s.yaml   # 期望: -rw-r--r-- ... k3s.yaml
cat /etc/rancher/k3s/k3s.yaml | head -5   # 应能读
```

### 3. 跑 kubectl

```bash
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get nodes -o wide
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get pods -A
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get ns
```

### 4. (可选) 永久化(每次新 WSL session 不用重设 KUBECONFIG)

```bash
echo "export KUBECONFIG=/etc/rancher/k3s/k3s.yaml" >> ~/.bashrc
source ~/.bashrc
```

## 验证脚本

Mavis agent 准备好等用户授权后跑:

```bash
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get nodes -o wide
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get pods -A
KUBECONFIG=/etc/rancher/k3s/k3s.yaml /usr/local/bin/k3s kubectl get ns
```

## 修复后

预计可看到:
- nodes: `ulyssespc` 1 个节点 Ready
- pods: 5 域 deployment (player / economy / match / social / admin) + infra (traefik 已 disable)
- ns: default / kube-system / kube-public / rgs(可能已建)

## 状态摘要 (11:48 JST)

- WSL Ubuntu: running
- k3s service: active (PID 180, since 11:39:22)
- k3s API server: listening 6443 (curl /healthz = 401)
- 5 域 deployment: 未知(权限阻塞)
- k3s log: 持续 cgroup task 恢复错误(WSL 重启副作用,可忽略)
