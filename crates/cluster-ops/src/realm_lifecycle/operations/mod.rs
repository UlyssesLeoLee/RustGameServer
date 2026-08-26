//! 6 阶段操作器（per RGS-DTL-042 §5.2）
//!
//! - [`archive`] —— **归档操作器（本任务 L4 #2074 完整实现）**
//! - 其它 5 个操作器（新服 / 扩缩容 / 分服 / 合服 / 退场）由其它 WBS L4 任务覆盖
//!
//! **本任务范围**：仅实现 [`archive`]，其它保留占位以满足 lib.rs 编译。

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod archive;

// ===== 其它 5 个操作器占位（per RGS-DTL-042 §5.2 表格） =====
pub mod new_realm {
    // 由 WBS L4 #2067 覆盖
}

pub mod scale {
    // 由 WBS L4 #2068 覆盖
}

pub mod split {
    // 由 WBS L4 #2068 覆盖
}

pub mod merge {
    // 由 WBS L4 #2071 覆盖
}

pub mod merge_rollback {
    // 由 WBS L4 #2071 覆盖
}

pub mod retire {
    // 由 WBS L4 #2071 覆盖
}
