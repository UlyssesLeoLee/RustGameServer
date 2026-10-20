# 新增域 fixture 覆盖缺口闭环 (8 域扩展)

范围: `tools/rgs-flash-mock/` mock 项目的 8 域扩展 fixture 覆盖自检闭环。
本文所有引用均为 file:line, 可独立 `Read` / `grep` 验证。

## 缺口来源

- mock 项目 `README.md:110-111` 已知缺口 #6 — "9 域 mTLS fixture 100% 覆盖":
  4 NEW 域 (scene/battle/network/account) 的 k8s yaml 已落档, 但 `mock_data/`
  缺对应 fixture JSON。
- `scripts/regression-test-8-domain-extension.sh` 原有 22 条
  `echo "    ✅ xxx/RpcName"` 是无断言输出: fixture 缺失 / JSON 非法 / RPC 未覆盖
  都不会让脚本失败 (假绿)。

## 本次补齐 (5 个 fixture / 22 RPC)

| fixture | RPC 数 | rpc_code 段 | 8 域脚本清单行 (单一事实来源) |
|---|---|---|---|
| `mock_data/scene_ext.json` | 4 | 8101-8104 | `scripts/regression-test-8-domain-extension.sh` L51-54 |
| `mock_data/battle_ext.json` | 4 | 8201-8204 | `scripts/regression-test-8-domain-extension.sh` L56-59 |
| `mock_data/network_ext.json` | 3 | 8301-8303 | `scripts/regression-test-8-domain-extension.sh` L61-63 |
| `mock_data/account_ext.json` | 3 | 8401-8403 | `scripts/regression-test-8-domain-extension.sh` L65-67 |
| `mock_data/sub8_ext.json` | 8 | 8501-8508 | `scripts/regression-test-8-domain-extension.sh` L69-76 |

- 期望清单不再写死在测试脚本里: 8 域脚本 L49-77 的清单注释块是唯一事实来源,
  `scripts/regression-test-new-domain-fixture-coverage.sh` 从该块解析。
- 8 域脚本步骤 2 (L79-98) 改为调用该自检脚本, 失败即 `exit 1`; 清单块与
  汇总行里 "22 RPC" 这个数字由 8 域脚本自己断言 (自检输出须含 `22/22 RPC`)。

## rpc_code 为什么是占位

- `src/gap_matrix.rs` 的 22 stub 与本组 8 域扩展 RPC 不是同一集合, gap_matrix 中
  没有 scene_ext / battle_ext / network_ext / account_ext / sub8_ext 条目
  (各 fixture `_module_meta.rpc_code_status` 已写明)。
- 故 8101-8508 段位由本次分配, `rgs_rpc` 一律写 `TBD` — 缺标比错标安全
  (per 仓根 `AGENTS.md:26` §1.1 文档治理), 待 gap_matrix 同步补齐后再定稿。
- 已知缺口: gap_matrix 缺这 5 个条目 → `rgs_rpc` 映射暂时无法实证。

## 怎么验证

```bash
cd tools/rgs-flash-mock
bash scripts/regression-test-new-domain-fixture-coverage.sh
RGS_MOCK_SKIP_CARGO_CHECK=1 bash scripts/regression-test-8-domain-extension.sh
```

- `scripts/regression-test-new-domain-fixture-coverage.sh`: 逐 fixture 校验
  存在 / JSON 合法 (python `json.load`) / `rpcs.rpc_name` 覆盖期望; 末尾输出
  `fixture coverage: 5/5 文件, 22/22 RPC`。
- 退出码语义: `0` = 全部满足; `1` = 有缺失 / JSON 非法 / 未覆盖, 并打印具体缺
  哪个 fixture 或哪个 `rpc_name` 后退出。
- `RGS_MOCK_SKIP_CARGO_CHECK=1` 只跳过 8 域脚本步骤 1 的 cargo check (开关 L23-24),
  默认行为不变 (不设该变量时照常 `cargo check --tests`, L28)。
- 临时文件用 `mktemp` + `trap` 清理, 不落仓库。

## 剩余缺口 (不在本 lane)

- `README.md:105` 已知缺口 #1 — 真实域未接: `gap_matrix` 22 RPC stub 缺 E2E
  故事链, ST 阶段需 per-case 接入 k3s 真实域。本 lane 只落 fixture + 静态覆盖
  断言, 未接真实域。
- `README.md:106-109` 已知缺口 #2/#3/#4/#5 — 跨域事件链 / plugin 集群 hot-swap /
  app 独立更新 / ops UI 用例, 均不在本 lane。
- gap_matrix 5 个新域条目待补 (见上节); 补齐后需重跑本脚本并回填 `rgs_rpc`。
