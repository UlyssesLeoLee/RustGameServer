//! W37 E2E Phase C marker —— social 域 (per RGS-PHASE-C-PREP-2026-09-02 v0.1 §2.2)
//!
//! ## 目的
//! 在 Phase C SRE 介入前, 5 域 E2E 测试函数 (11 E2E per RGS-TEST-RUN-PLAN v0.1) 还未
//! 跑通 (k3s 部署 + 5 域 binary 起来 + DB 池接通 前置缺失). 本 marker 测试作为
//! 5 域 E2E 函数补齐的占位 + 编译期锚定:
//!
//! 1. **编译期锚定**: 验证 social-service proto + common types 在 tests/ 目录下
//!    仍可引用, 防止 main.rs / proto.rs 重构破坏 IT 编译.
//! 2. **DoD 派生约束 L1 守护**: `cargo check -p social-service --tests` 0 error
//!    (per AGENTS.md §2.1 + §2.6 D3 commit 模板).
//! 3. **Phase C 阶段 C 标记**: 函数名 `ut_e2e_social_phase_c_marker` 在 Phase C
//!    介入后会被替换为真实 E2E 函数 (per RGS-TEST-RUN-PLAN v0.1 §1.2 11 E2E 清单).
//!
//! ## 风格 (per 任务简报)
//! - InMemory mock: 无, 用 proto 字段断言 (无 DB 依赖, 编译即可跑通)
//! - rgs-testkit NoOp: 无 DB 路径, 不需要 (per AGENTS.md §2.3 L3 派生约束)
//! - 临时 log 不入 commit (per L12)
//!
//! ## 已知缺口
//! - 真实 E2E 函数 (e2e_01_*) 需 Phase C 5 域 mTLS 部署完成 + DB 池接通
//!   (per RGS-PHASE-C-PREP-2026-09-02 v0.1 §2.4) 后由 SRE 主导落地

use social_service::common::v1 as common;
use social_service::proto::v1;

/// 5 域 E2E Phase C marker —— social 域.
#[test]
fn ut_e2e_social_phase_c_marker() {
    let entity = common::EntityId {
        id: "guild-e2e-marker-001".to_string(),
    };
    // social.proto v1 Guild 字段 (per DTL-019 §3 + 54.6):
    //   id (EntityId) / status (Status) / created_at (Timestamp) / display_name (string)
    // 容量 50 锚定 OPEN-QA v0.2 Q5 决策 (代码现状 50 为准, 不擅自改 64)
    let guild = v1::Guild {
        id: Some(entity),
        status: common::Status::Ok as i32,
        created_at: Some(common::Timestamp {
            seconds: 1_700_000_000,
            nanos: 0,
        }),
        display_name: "phase_c_marker_guild".to_string(),
    };

    // Phase C marker assertion: proto 字段 + 类型正确, marker 函数已就位
    assert_eq!(guild.id.as_ref().unwrap().id, "guild-e2e-marker-001");
    assert_eq!(guild.status, common::Status::Ok as i32);
    assert_eq!(guild.display_name, "phase_c_marker_guild");
    // 显式 marker 标识, 便于 Phase C 介入后 grep 定位替换
    assert_eq!(
        std::env::consts::ARCH, std::env::consts::ARCH,
        "social 域 Phase C marker 已就位 (per RGS-PHASE-C-PREP v0.1 §2.2)"
    );
}
