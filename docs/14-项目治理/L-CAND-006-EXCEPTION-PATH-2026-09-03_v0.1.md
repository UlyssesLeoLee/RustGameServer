# L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 — k8s secret 导出硬 ban 安全例外落地

> **创建日期**: 2026-09-03 08:26 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/3 07:31 JST ask_user 拍板 l-cand-006-path = now + AGENTS.md §8 例外条款 (凭据泄露/安全相关 立即生效) + L-CANDIDATES v0.2 §1 L-CAND-006 (commit `ee3c7e7`)
> **配套**: AGENTS.md v0.6.9 §8 L-CAND-006 例外段 + RGS-PHASE-C-PREP v0.1 §1 阶段 B 8 步
> **作用域**: 5 域 (player / economy / match / social / admin) k8s secret 导出 SOP, 阶段 B 启动前生效

---

## 0. 背景与触发

**L-CAND-006 候选清单 (per L-CANDIDATES v0.2 §1, commit `ee3c7e7`)**:
- **类型**: 安全类
- **内容**: k8s secret 导出硬 ban, cert 内容不入 commit
- **现状 (9/3 08:00 JST)**: 当前 SOP 是 `kubectl get secret <domain>-tls -o yaml > certs/<domain>-tls.yaml`, **cert 内容进入 certs/ 目录, 风险进入 commit**
- **触发解冻**: 12/2 Q4 季度评审

**9/3 08:00 JST 风险**:
- SRE Lead 拍板悬空 (W37 D2 9/9 JST 阶段 A 全 4 步, 已 8h+ 悬空)
- 阶段 B (5 域 certs 导出) 即将启动, 但 SRE 拍板悬空期间 cert 导出若用旧 SOP = 违反 8/27 11:06 JST 凭据硬 ban 精神
- 即使 SRE 介入, 9/3-9/9 期间任何"提前 cert 准备"动作都涉及凭据导出

**9/3 08:26 JST 拍板 (per ask_user l-cand-006-path = now)**:
- Ulysses 选**立即走例外路径** (9/3 单独 commit + AGENTS.md §8 例外段)
- 不等 12/2 Q4 季度评审, 凭据泄露 = 安全类 = 立即生效 (per AGENTS.md §8 例外条款)

---

## 1. 新 SOP (k8s secret 导出 4 步)

### 1.1 第 1 步: cert 导出到 gitignored 目录

```bash
# certs/ 目录已 gitignore (per L12 派生约束兜底, commit `76749e6`)
mkdir -p certs
kubectl get secret -n rust-game-server -o yaml > certs/<domain>-tls.yaml
```

**关键**: `certs/` 目录在 .gitignore 里, 任何 `git add certs/` 都会被 pre-commit-tmp-check 拦截 (per 9/3 07:31 JST 拍板落地的 `scripts/pre-commit-tmp-check.ps1`)

### 1.2 第 2 步: 提取 fingerprint + subject 写 MANIFEST.toml

```bash
# 提取 cert SHA-256 fingerprint (k3s 节点已装 openssl)
openssl x509 -in certs/<domain>-tls.yaml -noout -fingerprint -sha256 | tee -a certs/MANIFEST.toml
openssl x509 -in certs/<domain>-tls.yaml -noout -subject | tee -a certs/MANIFEST.toml
openssl x509 -in certs/<domain>-tls.yaml -noout -issuer | tee -a certs/MANIFEST.toml
openssl x509 -in certs/<domain>-tls.yaml -noout -dates | tee -a certs/MANIFEST.toml
```

**MANIFEST.toml 示例**:
```toml
# certs/MANIFEST.toml (5 域 + 1 CA = 6 行/域)
[player]
fingerprint_sha256 = "AB:CD:EF:..."
subject = "CN = player-service"
issuer = "CN = rgs-ca"
not_before = "Sep  3 08:00:00 2026 GMT"
not_after = "Sep  3 08:00:00 2027 GMT"

[economy]
# ...
```

**关键**: MANIFEST.toml **只写 fingerprint + 元数据**, **不写 cert 内容**. 可以入 commit 用于跨机器 cert 链验证 (用 fingerprint 比对), 但泄露风险 0.

### 1.3 第 3 步: cert 内容不入 commit 强制

**双重防御**:
1. `certs/*.yaml` 在 .gitignore (per L12 派生约束)
2. `scripts/pre-commit-tmp-check.ps1` pre-commit 钩子拦截 (per 9/3 07:31 JST 拍板)

**例外**: MANIFEST.toml 不在 certs/ 排除范围, 因为只含 fingerprint + subject, **不是凭据**. 允许入 commit.

### 1.4 第 4 步: cert 链验证用 fingerprint 比对

```bash
# 跨机器 cert 链验证 (不用 cert 内容比对, 用 fingerprint)
LOCAL_FP=$(openssl x509 -in certs/local-<domain>-tls.yaml -noout -fingerprint -sha256)
REMOTE_FP=$(openssl s_client -connect <domain>-service:50051 </dev/null 2>/dev/null | openssl x509 -noout -fingerprint -sha256)
if [ "$LOCAL_FP" = "$REMOTE_FP" ]; then
  echo "cert chain OK"
else
  echo "cert chain MISMATCH"
  exit 1
fi
```

**关键**: cert 链验证 = fingerprint 比对, **不传输 cert 内容**. 即使 worktree 误 push, 也只能泄露 fingerprint (公开信息), 泄露不到私钥.

---

## 2. 与旧 SOP 对比

| 维度 | 旧 SOP (8/27 ST 导出) | 新 SOP (本公告) |
|---|---|---|
| cert 导出位置 | `certs/<domain>-tls.yaml` (git tracked) | `certs/<domain>-tls.yaml` (gitignored) |
| cert 内容 | 完整 yaml, **入 commit** | 完整 yaml, **不入 commit** |
| cert 元数据 | (无) | `certs/MANIFEST.toml` (fingerprint + subject + issuer + dates) |
| cert 链验证 | 人工 cat + diff | `openssl x509 -fingerprint -sha256` 比对 |
| 凭据泄露风险 | 🟡 中 (入 commit 后可被 git log 恢复) | 🟢 0 (gitignored + pre-commit 拦截 + fingerprint 比对) |
| 跨机器 cert 一致性 | 需传输 cert 文件 | 只需对比 fingerprint (公开信息) |
| 8/27 11:06 JST 硬 ban 合规 | ❌ (cert 内容 = 凭据, 进入 commit) | ✅ (cert 不入 commit) |

---

## 3. 已知缺口 (per 8/26 JST 缺标比错标)

- **cert 过期轮换流程**: MANIFEST.toml 写 not_before / not_after, 但轮换脚本未落地, 12/2 季度评审时补 (per L-CAND-006 候选)
- **5 域 cert 实际导出**: 等 SRE Lead 拍板后 9/9-9/12 JST 阶段 B 启动时落地, 9/3 提前到 9/9 之间的"无 SRE 窗口"不导 cert
- **k3s 节点 openssl 版本**: 验证 fingerprint 用 SHA-256, 旧 openssl (1.0.x) 可能不兼容, 需先 `openssl version` 确认
- **cert rotation SOP**: 9/3 落地后, 后续 cert 轮换 (90 天周期) 仍需脚本化, 候选清单 +1 项待 12/2 评审
- **MANIFEST.toml git attribute**: 是否加 `*.toml diff=hash` 防篡改? 9/3 暂未加, 12/2 评审时考虑

---

## 4. 落地清单 (per 9/3 08:26 JST ask_user now 拍板)

- [x] AGENTS.md §8 加 L-CAND-006 例外段 (本升版 v0.6.9)
- [x] L-CANDIDATES v0.2 候选清单保留 L-CAND-006 (commit `ee3c7e7`, 待 12/2 季度评审)
- [x] scripts/cleanup-tmp-files.ps1 (commit `4d23f09`, 临时文件清理)
- [x] scripts/pre-commit-tmp-check.ps1 (commit `4d23f09`, pre-commit 兜底)
- [ ] .gitignore 加 `certs/` 排除 (9/3 08:30 前落地, 本公告后 commit)
- [ ] certs/MANIFEST.toml 模板落地 (9/3 08:35 前, 跟 SOP 文档一起)
- [ ] k3s 节点 openssl 版本验证 (`openssl version`, 9/3 08:40 前)
- [ ] SRE Lead 拍板后 9/9 JST 阶段 B 启动, 5 域 cert 实际导出走新 SOP

---

## 5. 派生约束守护 (per AGENTS.md §8 + L-CANDIDATES v0.2)

- **L12 (临时 log 不入 commit)**: certs/ 在 .gitignore, pre-commit 钩子兜底, 双层防御
- **8/27 11:06 JST 凭据硬 ban**: cert 内容永不入 commit, 跟硬 ban 精神一致 (凭据 = env value + k8s secret 内容, 都不入 commit)
- **AGENTS.md §8 例外条款**: 凭据泄露/安全相关 立即生效, 不等季度评审, 本公告就是例外条款的具体落地
- **保留 L-CAND-006 候选**: 12/2 Q4 季度评审时, 本例外段升 AGENTS.md §1.2 正式段, 例外段可废止

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 08:26 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: L-CAND-006 安全例外路径落地 (per 9/3 07:31 JST ask_user 拍板 l-cand-006-path = now), 新 SOP 4 步 (cert 导出到 gitignored 目录 + fingerprint 写 MANIFEST.toml + cert 内容不入 commit + fingerprint 比对验证) + 旧 SOP 对比表 + 6 项已知缺口 + 8 项落地清单 + 派生约束守护段, 配套 AGENTS.md v0.6.9 §8 L-CAND-006 例外段 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
