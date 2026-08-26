//! 鏈嶅姟鍣ㄥ叏鐢熷懡鍛ㄦ湡绠＄悊锛圠CM锛夆€?realm_lifecycle 瀛愭ā鍧楀叆鍙?//!
//! 瑙勮寖锛歊GS-SPEC-DTL-042 搂2 + ARC-051 Feature 鎵╁睍
//! 鍏ュ彛缁熶竴缁忕敱 `AdminService` 杞彂锛團R-LCM-004 纭害鏉燂級
//! 闃舵鍙樻洿浣滀负 `realm_lifecycle::*` Feature 瀛愮被璧?ClusterOpsService PFAU 缂栨帓
//!
//! ## 瀛愭ā鍧?//!
//! - [`plans`]锛? 寮?Plan 琛?entity + PgRepository 楠ㄦ灦锛坧er M-2068.7锛?//! - 鍚庣画 L4 浠诲姟锛圵F-1-2066 / WF-1-2067锛夊皢琛ュ厖锛?//!   - `operates/`锛? 闃舵鎿嶄綔鍣紙NewRealm / Scale / Split / Merge / Retire / Archive锛?//!   - `saga.rs`锛歋agaOrchestrator
//!   - `drill.rs`锛欴rillExecutor锛堟矙绠?PG + K8s 瀹㈡埛绔級
//!   - `feature_adapter.rs`锛欳lusterOpsService PFAU 闆嗘垚
//!   - `olu_reporter.rs`锛歄LU 棰勭畻涓婃姤锛坧er NFR-LCM-007锛?//!   - `metrics.rs`锛?0 椤?`rgs_lcm_*` 鎸囨爣
//!
//! ## 纭害鏉燂紙缁ф壙鑷?RGS-SPEC-DTL-042 搂3锛?//!
//! - **FR-LCM-001**锛? 寮犺〃鍏ㄩ儴鍦?admin_db锛涙湰瀛愭ā鍧椾笉鏂板缓鐙珛鏁版嵁搴?//! - **FR-LCM-003**锛欴rillExecutor **浠?*鍦ㄦ矙绠?PG 姹?+ 娌欑 K8s 瀹㈡埛绔窇
//! - **FR-LCM-004**锛氬叆鍙ｇ粺涓€缁忕敱 AdminService 杞彂锛涗笉鏆撮湶鐙珛鎺ュ彛
//! - **NFR-LCM-007**锛歄LU 棰勭畻涓婃姤**蹇呴』**缁?rgs-arc-olu 鏃㈠畾鏈嶅姟
//! - **NFR-SE-010**锛欸DPR 鍒犻櫎閫氳矾 admin_db.audit_log 鍙屽眰瀹¤

pub mod error;
pub mod plans;

// 鍚庣画 L4 浠诲姟浼氭墿灞曚互涓嬪瓙妯″潡锛坧er RGS-SPEC-DTL-042 搂2 瀹炵幇鍗曞厓锛夛細
// pub mod operates;
// pub mod saga;
// pub mod drill;
// pub mod feature_adapter;
// pub mod olu_reporter;
// pub mod metrics;

// 鍦?realm_lifecycle 鍛藉悕绌洪棿涓嬮噸鏂板鍑?plans 妯″潡鐨勫叕鍏遍」锛?// 鏂逛究璋冪敤鏂瑰啓 `realm_lifecycle::RealmLifecycleRun` 鑰岄潪 `realm_lifecycle::plans::RealmLifecycleRun`銆?pub use plans::*;
