//! Error types for the Function Plane.
//!
//! Mirrors the §8.3 / §9.2 / §15 failure model from RGS-INC-001 v0.2.
//! All errors flow up as `FunctionPlaneError`; the Gateway maps them to
//! `InvocationResult { success: false, ... }` when appropriate.
#![allow(missing_docs)]

use thiserror::Error;

/// All errors that can be produced by the Function Plane mock.
#[derive(Debug, Error)]
pub enum FunctionPlaneError {
    /// Function ID is not registered in the Registry.
    #[error("function not found: {0}")]
    NotFound(String),

    /// Function is registered, but the requested version does not exist.
    #[error("version not found: function={function_id} version={version}")]
    VersionNotFound {
        /// Logical function id.
        function_id: String,
        /// Requested semver-ish version string.
        version: String,
    },

    /// Generic Registry backend failure (lock poisoning, internal inconsistency, ...).
    #[error("registry error: {0}")]
    Registry(String),

    /// WASM bytes failed validation / compilation.
    #[error("wasm compile error: {0}")]
    WasmCompile(String),

    /// WASM module could not be instantiated (missing export, bad signature, ...).
    #[error("wasm instantiate error: {0}")]
    WasmInstantiate(String),

    /// WASM trap raised during execution (e.g. div-by-zero, unreachable).
    #[error("wasm trap: {0}")]
    WasmTrap(String),

    /// Execution exceeded the configured wall-clock timeout.
    #[error("execution timeout after {0}ms")]
    Timeout(u64),

    /// Wasmtime fuel ran out before the function returned.
    #[error("fuel exhausted (limit={limit})")]
    FuelExhausted {
        /// The fuel limit that was configured for this call.
        limit: u64,
    },

    /// Function tried to grow memory past the configured limit.
    #[error("memory limit exceeded: {limit_mib} MiB")]
    MemoryLimitExceeded {
        /// Memory limit in MiB at the time of the violation.
        limit_mib: u32,
    },

    /// Function is registered but its `status` is not `Active`.
    #[error("function not active: status={0}")]
    NotActive(String),

    /// Function contract is structurally invalid (missing WASM bytes for `Wasm`,
    /// invalid semver string, malformed JSON Schema, ...).
    #[error("contract invalid: {0}")]
    ContractInvalid(String),

    /// Catch-all for unexpected internal failures.
    #[error("internal: {0}")]
    Internal(String),
}

/// Convenient `Result` alias for the Function Plane.
pub type Result<T> = std::result::Result<T, FunctionPlaneError>;
