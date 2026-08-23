//! Function Plane mock (per RGS-INC-001 v0.2 §8 / §9 / §15).
//!
//! This crate is a **mock**, not a production implementation. It exists to:
//!
//! 1. Prove that a real Wasmtime engine can be embedded in the workspace
//!    (Apache-2.0, satisfying ADR-0020's reserved WASM upgrade path).
//! 2. Exercise the §15 Function Registry contract (register / get / version
//!    selection) against an in-memory backend.
//! 3. Offer a [`FunctionGateway`] facade so domain services and the PoC CLI
//!    can wire up invocations without a real gRPC front, NATS bridge, or
//!    capability manager.
//!
//! ## Module map
//!
//! | Module             | §8 / §9 / §15 anchors                                |
//! |--------------------|------------------------------------------------------|
//! | [`contract`]       | §15.2 schema (simplified)                            |
//! | [`error`]          | §9.2 / §15.3 failure model                           |
//! | [`registry`]       | §15.1 / §15.3 / §15.4 (InMemoryRegistry)             |
//! | [`wasm_host`]      | §9.1 / §9.2 / §9.4 / §9.5 (Wasmtime + resource cap)  |
//! | [`gateway`]        | §8.1 / §8.4 (Function Gateway facade)                |
//!
//! See `README.md` for the full list of 20+ UTs and the Phase 1+ backlog
//! (gRPC, KEDA, PG registry, capability manager, AI Function Pool, ...).
//!
//! ## License
//!
//! Apache-2.0 (per the workspace `license.workspace = true` declaration).

#![deny(missing_docs)]
// `wat` is a test-only dep (used by `tests/ut.rs` to compile WAT fixtures).
// It still shows up in the lib's "test build" cargo graph, so we silence
// the warning here rather than adding a dead `use wat as _;` to the lib.
#![allow(unused_crate_dependencies)]

pub mod contract;
pub mod error;
pub mod gateway;
pub mod registry;
pub mod wasm_host;

pub use contract::{
    FunctionContext, FunctionMetadata, FunctionStatus, InvocationRequest, InvocationResult,
    Runtime, TriggerType,
};
pub use error::{FunctionPlaneError, Result};
pub use gateway::FunctionGateway;
pub use registry::{FunctionRegistry, InMemoryRegistry};
pub use wasm_host::WasmHost;

/// Crate version (taken from `Cargo.toml` at compile time).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
