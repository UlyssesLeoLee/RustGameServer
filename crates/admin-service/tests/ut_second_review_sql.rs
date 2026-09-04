//! UT: second_review + function_registry SQL migration 文本层 schema 校验
//!
//! ## 目的
//! 验证 Phase B admin COC #3 + #4 新增 2 张表的 SQL DDL 文本结构对齐
//! `RGS-INC-001 v0.3 §X.4 (function_registry)` 与 `§X.5 (second_review)` 草案.
//!
//! ## 范围 (5 个测试函数)
//! 1. `ut_second_review_table_exists` — 验证 0007_second_review.sql 含 CREATE TABLE second_review
//! 2. `ut_second_review_columns` — 验证 14 个字段名 (review_id, request_id, actor_id, action,
//!    target_id, coc_decision, coc_reason, coc_module_version, coc_module_hash, coc_params_hash,
//!    original_request, status, reviewer_id, reviewed_at, review_comment, trace_id, created_at)
//! 3. `ut_second_review_status_default` — 验证 status DEFAULT 'pending' + CHECK 三态
//! 4. `ut_second_review_index` — 验证 idx_second_review_status_created 索引
//! 5. `ut_function_registry_table_columns` — 验证 0022_function_registry.sql 含 function_registry +
//!    6 字段名 (function_id, version, module_sha256, status, prev_version, uploaded_by, uploaded_at)
//!
//! ## 风格 (per 任务简报)
//! - **不**真连 DB (Phase C 之前 PG 不可达), 文本层 schema 校验
//! - 用 `std::fs::read_to_string` 读 migration 文件 (编译期不嵌入, 允许 0 字节空文件 fail)
//! - 强约束: schema 变更时本 UT 必须同步更新 (避免静默失同步)
//!
//! ## 已知缺口
//! - 仅校验 DDL 文本, **不**校验 SQL 语法 (PG syntax check 需 PG 可达, per Phase C 介入)
//! - 仅校验必备字段名, **不**校验字段类型 / CHECK 约束 (应用层负责)
//! - 不校验跨表 FK / 触发器 / 分区策略 (per RGS-SPEC-CROSS-005 §2 跨 DB 禁用外键, 本表无跨表 FK)

use std::fs;
use std::path::PathBuf;

/// 读 admin-service 自己的 migration 文件 (per `CARGO_MANIFEST_DIR` 锚定 crate 根)
fn read_admin_migration(file_name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("migrations");
    path.push(file_name);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "无法读 admin-service migration 文件 {}: {} \
             (per RGS-INC-001 v0.3 §X.5 派生约束: second_review migration 必须 commit)",
            path.display(),
            err
        )
    })
}

/// 读兄弟 crate (cluster-ops) 的 migration 文件 (从 admin-service 锚定往上一层)
fn read_cluster_ops_migration(file_name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // admin-service 在 crates/admin-service, 上两级到 D:\RustGameServer 根
    // 但 cluster-ops 与 admin-service 同级, 只需上 1 级
    path.pop(); // 去掉 admin-service 自身
    path.pop(); // 去掉 crates/
    path.push("crates");
    path.push("cluster-ops");
    path.push("migrations");
    path.push(file_name);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "无法读 cluster-ops migration 文件 {}: {} \
             (per RGS-INC-001 v0.3 §X.4 派生约束: function_registry migration 必须 commit)",
            path.display(),
            err
        )
    })
}

/// Test 1: 0007_second_review.sql 含 CREATE TABLE second_review
#[test]
fn ut_second_review_table_exists() {
    let sql = read_admin_migration("0007_second_review.sql");
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS second_review"),
        "0007_second_review.sql 缺 CREATE TABLE second_review \
         (per RGS-INC-001 v0.3 §X.5 schema 草案)"
    );
    // PRIMARY KEY review_id UUID (字段对齐用 split_whitespace 规范化空格)
    let has_pk = sql
        .lines()
        .any(|line| {
            let normalized: Vec<&str> = line.split_whitespace().collect();
            normalized.starts_with(&["review_id", "UUID", "PRIMARY", "KEY"])
        });
    assert!(
        has_pk,
        "0007_second_review.sql 缺 PRIMARY KEY review_id UUID \
         (per RGS-INC-001 v0.3 §X.5 schema 草案)"
    );
}

/// Test 2: 0007_second_review.sql 含 17 字段名 (per §X.5 schema)
#[test]
fn ut_second_review_columns() {
    let sql = read_admin_migration("0007_second_review.sql");
    let required_columns = [
        "review_id",
        "request_id",
        "actor_id",
        "action",
        "target_id",
        "coc_decision",
        "coc_reason",
        "coc_module_version",
        "coc_module_hash",
        "coc_params_hash",
        "original_request",
        "status",
        "reviewer_id",
        "reviewed_at",
        "review_comment",
        "trace_id",
        "created_at",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for col in &required_columns {
        if !sql.contains(col) {
            missing.push(col);
        }
    }
    assert!(
        missing.is_empty(),
        "0007_second_review.sql 缺字段 {} (per RGS-INC-001 v0.3 §X.5 schema 草案, 17 字段必备)",
        missing.join(", ")
    );
}

/// Test 3: status DEFAULT 'pending' + CHECK 三态
#[test]
fn ut_second_review_status_default() {
    let sql = read_admin_migration("0007_second_review.sql");
    let has_default = sql.lines().any(|line| {
        let normalized: Vec<&str> = line.split_whitespace().collect();
        normalized.starts_with(&["status", "TEXT", "NOT", "NULL", "DEFAULT", "'pending'"])
            || normalized.starts_with(&["status", "TEXT", "NOT", "NULL"])
                && line.contains("DEFAULT 'pending'")
    });
    assert!(
        has_default,
        "0007_second_review.sql 缺 status DEFAULT 'pending' \
         (per RGS-INC-001 v0.3 §X.5 schema 草案)"
    );
    assert!(
        sql.contains("CHECK (status IN ('pending', 'approved', 'rejected'))"),
        "0007_second_review.sql 缺 status CHECK 三态 (pending/approved/rejected) \
         (per RGS-INC-001 v0.3 §X.5 schema 草案 + §X.5 状态机)"
    );
}

/// Test 4: idx_second_review_status_created 索引
#[test]
fn ut_second_review_index() {
    let sql = read_admin_migration("0007_second_review.sql");
    assert!(
        sql.contains("idx_second_review_status_created"),
        "0007_second_review.sql 缺 idx_second_review_status_created 索引 \
         (per RGS-INC-001 v0.3 §X.5 schema 草案 + §X.5 SLA 24h 扫表路径)"
    );
    // 索引必须含 (status, created_at) 列序 (per §X.5 原始 schema)
    assert!(
        sql.contains("ON second_review (status, created_at)"),
        "0007_second_review.sql 缺 (status, created_at) 索引列序 \
         (per RGS-INC-001 v0.3 §X.5 schema 草案)"
    );
}

/// Test 5: 0022_function_registry.sql 含 function_registry + 7 字段 + 复合主键 + 索引
#[test]
fn ut_function_registry_table_columns() {
    let sql = read_cluster_ops_migration("0022_function_registry.sql");
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS function_registry"),
        "0022_function_registry.sql 缺 CREATE TABLE function_registry \
         (per RGS-INC-001 v0.3 §X.4 schema 草案)"
    );
    let required_columns = [
        "function_id",
        "version",
        "module_sha256",
        "status",
        "prev_version",
        "uploaded_by",
        "uploaded_at",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for col in &required_columns {
        if !sql.contains(col) {
            missing.push(col);
        }
    }
    assert!(
        missing.is_empty(),
        "0022_function_registry.sql 缺字段 {} \
         (per RGS-INC-001 v0.3 §X.4 schema 草案, 7 字段必备)",
        missing.join(", ")
    );
    // 复合主键 (function_id, version) per §X.4
    assert!(
        sql.contains("PRIMARY KEY (function_id, version)"),
        "0022_function_registry.sql 缺 PRIMARY KEY (function_id, version) 复合主键 \
         (per RGS-INC-001 v0.3 §X.4 schema 草案)"
    );
    // status 索引 per §X.4 原始 schema
    assert!(
        sql.contains("idx_function_registry_status"),
        "0022_function_registry.sql 缺 idx_function_registry_status 索引 \
         (per RGS-INC-001 v0.3 §X.4 schema 草案 + §X.6 第 5 条 status 热路径)"
    );
}
