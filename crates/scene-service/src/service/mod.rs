//! scene-service 域 service 模块
//!
//! 7 域业务实施 (per 9/5 改进路线图 Phase 2 + 9/4 MD §2 148 RPC)
//! - SceneService trait: 业务层抽象
//! - SceneServiceImpl: 默认实现
//! - grpc_service: gRPC 桥接
//!
//! 设计原则:
//! - ≥20 真实业务逻辑 (per DoD)
//! - 128 stub Unimplemented
//! - InMemory repository 满足 L1 cargo check 0 error

pub mod grpc_service;
pub mod scene_service;

pub use grpc_service::SceneGrpcService;
pub use scene_service::{SceneService, SceneServiceImpl};
