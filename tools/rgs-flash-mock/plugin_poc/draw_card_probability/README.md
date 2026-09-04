# draw_card_probability PoC Plugin

> **创建日期**: 2026-09-05 07:10 JST
> **依据**: RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §2.2 + RGS-TEST-DESIGN-2026-09-05 v0.1 §7.2 PLUGIN-001
> **状态**: ⏳ PoC 待 v0.1 sprint 实施

## 0. 目的

PoC 验证 Function 级别 plugin 集群核心流程:
1. WASM module 编译 + 部署到 function-plane registry
2. card-service 在不动代码情况下, 通过 FunctionGateway 调 plugin
3. plugin hot-swap (换 v0.2.0 → SSR 概率从 5% 变 6%) 不重启 card-service
4. plugin 故障时 card-service 走 native fallback (不挂)

## 1. 插件接口契约 (per function-plane/src/contract.rs)

### 1.1 输入 (per function-plane/src/wasm_host.rs L9.5 注释)

```json
{
  "rarity": "SSR",          // string, N | R | SR | SSR | UR
  "pity_count": 73,         // i32, 已连续未出 SSR 的次数
  "banner_id": "limited_001" // string, 卡池 id (限定 / 常驻)
}
```

### 1.2 输出

```json
{
  "result": "hit",        // "hit" | "miss" | "guaranteed"
  "rarity": "SSR",       // 命中的稀有度
  "probability": 0.06,    // f64, 实际概率 (含 pity 提升)
  "pity_bonus": 0.01      // f64, pity 加成
}
```

### 1.3 WASM 导出

per function-plane/src/wasm_host.rs L11-15 注释, compute export:

```wat
(module
  (func (export "compute") (param i32 i32) (result i32) ...))
```

输入 i32 指针 + 长度 (linear memory), 输出 i32 指针 (caller 解码).

## 2. 1.0 实现 (WAT 文本格式, per function-plane/src/wasm_host.rs L9.5)

WAT 是 WebAssembly Text Format, 1.0 实现最简化, 概率 5% 不含 pity.

```wat
;; draw_card_probability v1.0.0
;; 5% SSR 概率, 不含 pity 加成
(module
  (memory (export "memory") 1)
  (func (export "compute") (param $ptr i32) (param $len i32) (result i32)
    ;; 简化的 hash + 阈值判定
    ;; 真实 PoC: input JSON 解析 → 算概率 → 输出 JSON
    ;; 这里用常量 mock: 固定返回概率 0.05
    (i32.const 0)))  ;; stub: 真实现见 v1.1
```

**v1.0 简化说明**: 完整 WAT 实现需要 input/output 解析 (string JSON 解析在 WAT 复杂).
本 PoC 用 stub: compute 返回 0, FunctionGateway 知道这是 v1.0 stub, 用 hardcoded probability 0.05.

## 3. 1.1 实现 (WAT 完整版, 含概率计算)

v1.1 增加 pity 加成: pity_count >= 80 概率 100% (保底), 否则按基础概率 + pity * 0.005 加成.

```wat
;; draw_card_probability v1.1.0
;; 基础 5% + pity 加成, 80 次保底
(module
  (memory (export "memory") 1)

  ;; 简化概率: input = pity_count, output = probability * 10000
  (func (export "compute") (param $ptr i32) (param $len i32) (result i32)
    ;; 实际生产: linear memory 读 JSON, 解析 rarity/pity_count
    ;; v1.1 PoC stub: 概率 = base 5% + pity 0.5% × min(pity, 80)
    ;; 返回 probability × 10000 (i32 范围)
    local.get $ptr      ;; ptr to input (pity_count i32)
    i32.load            ;; 读 pity_count
    local.tee $pity
    i32.const 80
    i32.lt_s
    (if
      (then
        ;; probability = 500 + pity * 50  (×10000)
        i32.const 500
        local.get $pity
        i32.const 50
        i32.mul
        i32.add
      )
      (else
        ;; guarantee = 10000 (×10000 = 100%)
        i32.const 10000))))
```

## 4. 部署 (per RGS-PLUGIN-APP-ARCH §2.2 §3.1)

### 4.1 文件结构

```
tools/rgs-flash-mock/plugin_poc/
├── README.md                # 本文档
├── draw_card_probability_v1.0.0.wat  # 1.0 WAT 源
├── draw_card_probability_v1.0.0.wasm # 1.0 编译后 (wat2wasm)
├── draw_card_probability_v1.1.0.wat
├── draw_card_probability_v1.1.0.wasm
├── metadata.json            # function-plane registry record
└── test-invoke.sh           # 单元 invoke 测试
```

### 4.2 metadata.json (per function-plane/src/contract.rs FunctionMetadata)

```json
{
  "function_id": "card.draw_card_probability",
  "version": "v1.1.0",
  "runtime": "Wasm",
  "trigger_type": "Grpc",
  "status": "Active",
  "fuel": 1000000,
  "memory_mib": 32,
  "code_uri": "file://tools/rgs-flash-mock/plugin_poc/draw_card_probability_v1.1.0.wasm",
  "hash_sha256": "<sha256 of wasm>",
  "owner": "card-service-Lead",
  "config": {
    "base_probability": 0.05,
    "pity_threshold": 80,
    "pity_step": 0.005
  }
}
```

### 4.3 test-invoke.sh (PoC 验证脚本)

```bash
#!/usr/bin/env bash
# PoC plugin invoke test (per RGS-PLUGIN-APP-ARCH §4.2 阶段 1)
set -e

FUNCTION_PLANE_URL="${FUNCTION_PLANE_URL:-http://127.0.0.1:8792}"
FUNCTION_ID="card.draw_card_probability"
VERSION="v1.1.0"

echo "=== PoC plugin invoke test ==="
echo "Function: $FUNCTION_ID $VERSION"

# 1. 健康检查
echo "1. FunctionPlane /health"
curl -sS "$FUNCTION_PLANE_URL/health" | jq .

# 2. 调 invoke 多次, 验证 pity 加成
for pity in 0 50 79 80 100; do
  echo ""
  echo "2.$pity. invoke pity_count=$pity"
  curl -sS -X POST "$FUNCTION_PLANE_URL/invoke/$FUNCTION_ID" \
    -H "Content-Type: application/json" \
    -d "{\"rarity\":\"SSR\",\"pity_count\":$pity,\"banner_id\":\"limited_001\"}" \
    | jq .
done

# 3. 验证 hot-swap 不重启 app
echo ""
echo "3. Hot-swap test (v1.1.0 → v1.2.0 提概率到 6%)"
echo "  3.1 register v1.2.0 Draft"
curl -sS -X POST "$FUNCTION_PLANE_URL/registry/$FUNCTION_ID" \
  -H "Content-Type: application/json" \
  -d @metadata_v1.2.0.json | jq .
echo "  3.2 set_status(Active)"
curl -sS -X POST "$FUNCTION_PLANE_URL/registry/$FUNCTION_ID/v1.2.0/status" \
  -d '{"status":"Active"}' | jq .
echo "  3.3 set_old_status(Archived) — 旧版本下线"
curl -sS -X POST "$FUNCTION_PLANE_URL/registry/$FUNCTION_ID/v1.1.0/status" \
  -d '{"status":"Archived"}' | jq .
echo "  3.4 验证 card-service 进程未重启"
ps -ef | grep card-service | grep -v grep | awk '{print "PID:", $2, "started:", $5}'
```

## 5. 跟 function-plane mock 集成 (per §4.2 阶段 1 落地)

### 5.1 mock_function_plane 路径

function-plane 现状是 mock, 没有 gRPC 端. 阶段 1 落地时:
- `function-plane` 加 gRPC `FunctionRegistryService` (port 8792)
- 加 HTTP front `/invoke/{function_id}` (per test-invoke.sh)
- mock 阶段: 把本 PoC metadata 写到 `tools/rgs-flash-mock/mock_data/functions/`

### 5.2 card-service 集成 (per §4.2 阶段 1)

```rust
// crates/card-service/src/service.rs (PoC 集成示例)
// 1. 构造 FunctionGateway (启动时)
let gateway = FunctionGateway::with_in_memory()?;

// 2. 注册 draw_card_probability v1.1.0
gateway.registry().register(metadata_v1_1_0).await?;

// 3. 抽卡业务调 plugin
pub async fn draw_card(&self, rarity: &str, pity: i32) -> Result<DrawResult> {
    // 尝试 plugin
    let result = gateway.invoke("draw_card_probability",
        json!({"rarity": rarity, "pity_count": pity, "banner_id": "limited_001"})
    ).await;

    match result {
        Ok(r) => Ok(DrawResult::from_plugin(r)),
        Err(_) => {
            // native fallback (per 8/27 55.26 fail-closed 精神)
            tracing::warn!("draw_card_probability plugin failed, fallback to native");
            self.native_draw_card(rarity, pity).await
        }
    }
}

async fn native_draw_card(&self, rarity: &str, pity: i32) -> Result<DrawResult> {
    // 5% 基础 + pity 加成 (与 v1.1.0 plugin 同步)
    let base = 0.05;
    let pity_bonus = (pity as f64 * 0.005).min(0.95);
    let probability = base + pity_bonus;
    // rand 判定 hit / miss
    Ok(DrawResult { ... })
}
```

## 6. 测试用例 (per RGS-TEST-CASES-2026-09-05 v0.1 §3.2)

| 用例 ID | 名称 | 期望 |
|---|---|---|
| PLUGIN-001 | PoC 抽卡 hot-swap | card-service 进程不重启, 概率生效 |
| PLUGIN-002 | WASM 资源超 fuel cap | plugin 失败, 走 native fallback |
| PLUGIN-003 | WASM 内存超 memory_mib cap | plugin 失败, 走 native fallback |
| PLUGIN-004 | plugin Paused 时 | gateway 拒绝, 走 native fallback |
| PLUGIN-005 | 跨 app plugin 双 registry 一致性 | player + card app 都拿同一 v1.1.0 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 07:10 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review 二审 | 起草, PoC WAT 模板 + 部署 yaml + invoke 测试脚本 |
