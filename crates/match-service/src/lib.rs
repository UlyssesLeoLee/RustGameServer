//! match-service —— 5 域匹配域微服务
//!
//! 域职责：房间匹配 / 对战撮合 / Match Slot Reservation / 不可逆比赛结算
//! 规范：RGS-REQ-016 / RGS-BAS-016 / RGS-DTL-016 / RGS-SPEC-DTL-016
//!        RGS-DTL-102 §6 比赛已结束 Manual Intervention
//! DB：独立 match_db（per ARC-008）
//! gRPC API：match/v1/match.proto
//!
//! 53.2 占位。