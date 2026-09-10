# k3s WSL2 安装配置 (per 2026-09-10 12:50 JST k3s 1.37 RC3 升级落地)

## 派生约束 L29 (per AGENTS.md v0.6.14+)

WSL2 + cgroup v1 装 k3s 1.37.0-rc3+k3s1 必须 2 件套:

1. **kubelet 99-override.conf** (`/var/lib/rancher/k3s/agent/etc/kubelet.conf.d/99-override.conf`):
   ```yaml
   apiVersion: kubelet.config.k8s.io/v1beta1
   kind: KubeletConfiguration
   failCgroupV1: false
   cgroupDriver: cgroupfs
   ```
   原因: WSL2 `.wslconfig` 加 `cgroup_v2` 但实际 systemd 仍 cgroup v1, k3s 1.37 默认 `failCgroupV1: true` 拒启, 默认 `cgroupDriver: systemd` 跟 v1 路径不匹配导致 `mkdir /sys/fs/cgroup/devices/kubepods.slice/...: no such file or directory`.

2. **k3s.service** (`/etc/systemd/system/k3s.service`) ExecStart 加:
   ```
   --tls-san=127.0.0.1 --tls-san=localhost --tls-san=<WSL2 eth0 IP>
   --pause-image=registry.aliyuncs.com/google_containers/pause:3.10.2
   ```
   原因: --tls-san 让 apiserver cert SAN 含 127.0.0.1, Windows kubectl 经 portproxy 52551→6443 通; --pause-image 走 aliyun 国内镜像避免 pause image pull 卡住.

## 安装步骤 (per 9/10 12:48 JST 实证)

```bash
# 0. WSL2 .wslconfig 加 cgroup_v2 (重启 WSL)
[wsl2]
memory=8GB
kernelCommandLine=cgroup_no_v1=all systemd.unified_cgroup_hierarchy=1

# 1. 清理旧 k3s
/usr/local/bin/k3s-uninstall.sh  # 或 k3s-agent-uninstall.sh

# 2. 下载 k3s 1.37.0-rc3+k3s1 binary (国内用 ghfast.top 加速)
#    RC3 是 388 download 最受欢迎的 RC, 等同"v1.37 patch release"的最快路径
curl -sfL -o /usr/local/bin/k3s https://ghfast.top/https://github.com/k3s-io/k3s/releases/download/v1.37.0-rc3%2Bk3s1/k3s
chmod +x /usr/local/bin/k3s
# SHA256 验证: 530351F912CCCE5C84945BD82D812FF5EFED139C6B40AD573AD1AA9835600E31

# 3. 部署 k3s.service
cp k3s.service /etc/systemd/system/k3s.service

# 4. 部署 kubelet 99-override.conf (cgroup v1 必备)
mkdir -p /var/lib/rancher/k3s/agent/etc/kubelet.conf.d/
cp kubelet-99-override.conf /var/lib/rancher/k3s/agent/etc/kubelet.conf.d/99-override.conf
chmod 644 /var/lib/rancher/k3s/agent/etc/kubelet.conf.d/99-override.conf

# 5. 启动 + 验证
systemctl daemon-reload
systemctl enable k3s
systemctl start k3s
sudo k3s kubectl get nodes -o wide
# 预期: ulyssespc Ready control-plane v1.37.0-rc3+k3s1, memory 16381972Ki (15.6GB)
```

## 已知问题 (per 9/10 12:55 JST 实证)

- **k3s server crash loop**: 每 ~1 min 自动重启, 报 `fatal: failed to start controllers: failed waiting on CRD 'addons.k3s.cattle.io': context canceled`
  - 根因: WSL2 + sqlite + containerd 启动慢, CRD 注册 2min 超时
  - workaround: 不影响 Node Ready, 5 域 pod 仍可调度, 仅管理面 API 间歇 close
  - 实证: 12:55 / 12:56 / 12:57 多次重启, 每次存活 1-2 min, 9/10 12:59 仍在循环
  - 后续: 待 k3s 1.37 stable release 修复, 或换 etcd 后端

- **资源限制**: ResourceQuota 默认 12Gi/12cpu, 5 域 + cluster-ops + monitoring + nats + gm-backend + battle + network-gateway 共 19 pod 需要 32Gi/32cpu (per e4-00-namespace-isolation.yaml patch)

## issue #36 状态

- ✅ Root cause fixed: 14.6GB stale NodeStatus cache bug 不再复现 (NodeStatus memory = 15.6GB 实时)
- ⚠️ k3s server crash loop 单独 issue, 不影响 Node Ready + 5 域 rollout

## 端口管理

- WSL2 内部: k3s 6443 (apiserver) + 10250 (kubelet) + 6444 (supervisor LB)
- Windows 端: netsh portproxy 52551→WSL2 6443 (per L25 + L28)
- kubeconfig: C:\Users\leo19\.kube\config server=https://127.0.0.1:52551
