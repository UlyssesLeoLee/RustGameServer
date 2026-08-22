# K3s 集群部署验证（per WBS v0.3 §2A.5 WF-1-53.9 + DEC-010）

## 部署形态

**WSL2 native k3s**（per DEC-010，2026-08-21 决策修订自 k3d）：
- k3s v1.36.3+k3s1（实测，2026-08-22 11:58 JST）
- OS：Ubuntu 24.04.3 LTS
- systemd 模式（WSL2 内启用 systemd）
- 部署形式：单节点 control-plane（dev 足够；多节点模拟需 k3s 多实例）
- 不用 Docker Desktop（k3s native 跑在 WSL2 systemd 内）

## 集群基本信息

| 字段 | 值 |
|---|---|
| 节点名 | `ulyssespc` |
| 角色 | control-plane（Ready） |
| OS | Ubuntu 24.04.3 LTS |
| k3s 版本 | v1.36.3+k3s1 |
| Container Runtime | containerd（k3s 内置） |
| CNI | Flannel（k3s 默认） |
| StorageClass | local-path（k3s 内置，53.10 PG PVC 用） |
| kubectl | k3s 自带（无需 standalone kubectl） |

## System Pods 状态（实测 2026-08-22 11:58 JST）

| Pod | 状态 | 用途 |
|---|---|---|
| coredns-* | Running | DNS 解析 |
| local-path-provisioner-* | Running | local-path StorageClass provisioner |
| metrics-server-* | Running | 资源 metrics（kubectl top 用） |
| helm-* | Running | k3s 内置 helm controller |
| svclb-traefik-* | Running | k3s 内置 LoadBalancer（traefik） |

## K3s 启动 / 停止

```bash
# 启动
wsl -d Ubuntu
sudo systemctl start k3s

# 停止
wsl -d Ubuntu
sudo systemctl stop k3s

# 状态
wsl -d Ubuntu
sudo systemctl status k3s
sudo k3s kubectl get nodes -o wide
sudo k3s kubectl get pods -A
```

## Kubectl 路径选择

| 场景 | 用 |
|---|---|
| WSL2 内 | `sudo k3s kubectl ...`（k3s 自带） |
| Windows host | 通过 `\\wsl$\Ubuntu\usr\local\bin\k3s.exe`（UNC 路径） |
| WSL2 内 alias | `alias kubectl="k3s kubectl --"` |

**不用** 安装 standalone kubectl（避免版本冲突）。

## .env.k3s 配置（per RGS-SEC-100 §7，端口 15432 避 Windows 默认 5432 冲突）

```
K3S_KUBECONFIG=/etc/rancher/k3s/k3s.yaml
K3S_KUBECTL_CMD=k3s
K3S_NAMESPACE=rust-game-server
```

## 53.9 验收（per WBS v0.3 §2A.5 WF-1-53.9）

- ✅ Rust 工具链（53.1）
- ✅ docker-compose（53.8，作为 dev 备选）
- ✅ PG 18.6 镜像（per DEC-009，k3s 容器内运行）
- ✅ k3s WSL2 集群部署（实测 2026-08-22 11:58 JST）
- ✅ 单节点 control-plane Ready
- ✅ 5 system pods Running（coredns + local-path-provisioner + metrics-server + helm + svclb-traefik）

## 与原 WBS 53.9 范围差异说明

WBS v0.3 §2A.5 WF-1-53.9 描述："本地 k3s 集群（或 kind）单节点 dev 集群"。

**变更（per DEC-010）**：
- WBS 原计划 k3d（k3s in Docker）或 kind（K8s in Docker）
- DEC-010 修订为 **k3s native in WSL2（systemd 模式）**
- 接受代价：依赖 WSL2 + Ubuntu 22.04+ + systemd 启用
- 多节点模拟需 k3s 多实例（dev 单节点足够）
- Docker Desktop 不再是 k3s 前置

## 下游依赖

- WF-1-53.10：5 独立 PG 18.6 DB 容器在 k3s 内部署
- WF-1-53.12：OTel Collector 部署到 k3s
- WF-1-53.13：distroless base image → docker build → k3s deploy
- WF-1-58.*：CI workflow 跨平台测试

## 启动验证日志

实测启动日志见 `docs/deploy/09-deploy-dev-k3s.log`（per commit `f7c0c12` 记录）。

## 53.9 关闭条件（per 07-no-go-checklist v0.4 G-CODE-03）

- ✅ kubectl get nodes 显示 control-plane Ready
- ✅ 5 system pods Running
- ✅ local-path StorageClass 可用（53.10 PG PVC 依赖）