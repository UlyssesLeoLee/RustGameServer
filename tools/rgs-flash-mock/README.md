# rgs-flash-mock

RGS [游戏A] mock gateway / verification harness (per `RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3`).

> ⚠️ **此目录与 `DEPRECATED.md` 早期标记不同**——DEPRECATED.md 描述的是 2026-09-09
> `rgs-shim-rust` 战略切换前的旧 21 RPC stub mock;自 v0.1 起 (`c5c40062`)
> 已重定位为"verification harness / gap matrix + mTLS 业务级 stub"，
> 与 `rgs-shim-rust` 字节级兼容路径并存。详见 `docs/V0.2-IMPLEMENTATION-REPORT.md`
> / `V0.3-IMPLEMENTATION-REPORT.md`。

## 当前形态 (v0.3)

| 项目 | 说明 |
|---|---|
| HTTP server | actix-web 4, 监听 `0.0.0.0:8791` (env `RGS_GAP_MOCK_BIND` 可覆盖) |
| gRPC clients | tonic 0.12 + rustls/ring, **7 域 mTLS 业务级** (player / economy / match / social / admin + card + leaderboard) |
| RPC stub | 22 RPC 假数据 + 1 RPC (`1201 GetPlayerCollection` v0.3 新增) |
| Gap matrix | `src/gap_matrix.rs` — `RpcStatus` 4 态 / `RpcCategory` 13 类 / 22 RPC stub |
| 测试脚本 | `scripts/ut.sh` / `it.sh` / `st.sh` + 9 个 regression-test-*.sh + `regression-test-all.sh` orchestrator |
| fixture | `mock_data/` 48 个 JSON, W2 12 Partial + W3 30 + 8 域扩展 6 + batch 域 6 + 4 NEW 域 mTLS |

## 派生约束 (per AGENTS.md §2.1 + 8/27 三次强化 + 9/4 17:47)

- L1 `cargo check --tests` 60s 限时, 失败 fail-fast 不重试
- L1.1 `cargo test --lib` 120s 限时, 不 polling
- 凭据永不打印 (8/27 11:06 JST hard ban)
- 代签规则 (8/27 19:39 / 20:56 / 21:59 JST 三次强化)
- 测试脚本+数据归入 mock 项目 (9/4 17:47 JST user 偏好, "以备回归测试")

## 快速跑一遍回归

```bash
cd tools/rgs-flash-mock
CARGO_TARGET_DIR="D:/RustGameServer/target/flash-mock-ulys141" \
  bash scripts/regression-test-all.sh
```

orchestrator 会依次跑 UT → IT (9 个回归脚本) → ST (mock server + RPC 抽样),
日志落到 `tools/rgs-flash-mock/logs/regression-test-all-YYYYMMDD-HHMM.log`。

## 各脚本范围

| 脚本 | 范围 (per RGS-TEST-DESIGN v0.2 §8 + §10) |
|---|---|
| `scripts/ut.sh` | L1 cargo check + L1.1 cargo test --lib + UT 覆盖度审计 (3 mod / 34 test) |
| `scripts/it.sh` | L1 cargo check + 48 fixture JSON valid + cmds 总数 + 9 个 regression 脚本 |
| `scripts/st.sh` | mock server 启动 + /health /ready + 21 RPC 抽样 (需 RGS 域服务可达) |
| `scripts/regression-test-60-all-modules.sh` | 60 module 全覆盖 (~966 用例) |
| `scripts/regression-test-12-partial.sh` | W2 启动 Phase 2 12 Partial mock.json + 9 域 mTLS + batch 6 module |
| `scripts/regression-test-30-new-module.sh` | W3 启动 Phase 3 60 module + 9 域 mTLS + admin-coc §X |
| `scripts/regression-test-9-domain-mtls.sh` | 9 域 mTLS 端到端 11 步客户端模拟器 v3 (per 9/6 d270ab9) |
| `scripts/regression-test-8-domain-extension.sh` | 8 域扩展 (scene / battle / network / account / sub8) 22 RPC |
| `scripts/regression-test-batch-domain.sh` | batch 域 6 module × 15 用例 = 90 用例 |
| `scripts/regression-test-plugin-poc.sh` | PLUGIN-001~007 plugin PoC WASM |
| `scripts/regression-test-admin-coc.sh` | admin-coc §X 集成 3 场景 (1101/1102/1103) + 7 项 admin 域 Lead 真实签字 |
| `scripts/regression-test-all.sh` | UT + IT + ST orchestrator (本目录整体回归) |
| `scripts/smoke-test.sh` | 60 module fixture + 9 域 mTLS + batch 6 + 8 域扩展 + admin-coc §X |

## 已知缺口 (per RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1 §3)

1. **场景文档依赖** — `gap_matrix` 22 RPC stub 缺 E2E 故事链;ST 阶段需 per-case 接入 k3s 真实域
2. **跨域事件链** — saga_abstract 3 module 仅 mock, 缺 e2e cross-domain event verification
3. **plugin 集群** — draw_card_probability PoC 仅 v0.1, 缺 hot-swap E2E 实战验证
4. **app 独立更新** — 缺 4 用例 (APP-DEPLOY-001~004) per 主设计书 v0.2 §7.4
5. **ops UI** — 缺 6 用例 (OPS-UI-001~006) per 主设计书 v0.2 §7.4
6. **9 域 mTLS fixture 100% 覆盖** — 4 NEW 域 (scene/battle/network/account) k8s yaml 9/6 已落档,
   但 mock_data/ 暂未对应 4 个 fixture JSON

## 关联文档

- `docs/V0.2-IMPLEMENTATION-REPORT.md` / `V0.3-IMPLEMENTATION-REPORT.md` — 实施报告
- `docs/12-大类-RPC-清单.md` — 60 module gap matrix (含 §16 W3 + §17 v0.3 升版)
- `docs/W2-PHASE-2-*` / `docs/W3-PHASE-3-*` — Worker 1..5 报告 (W2 2 报告 + 1 handoff + W3 5 报告)
- `../../docs/06-测试与质量保障/RGS-TST-UT-06_*.md` / `RGS-TST-IT-06_*.md` / `RGS-TST-ST-06_*.md`
- `../../docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md` — DDD Review

---

**v0.3 升版**: Mavis 接手 agent (per DEC-008), 代签 Ulysses (8/27 19:39/20:56/21:59 JST 三次强化).