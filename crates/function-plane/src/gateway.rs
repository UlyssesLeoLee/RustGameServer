//! High-level `FunctionGateway` facade (per RGS-INC-001 v0.2 §8.1 / §8.4).
//!
//! The Gateway is the only call-site a domain service should know about.
//! It composes:
//!
//! 1. a [`FunctionRegistry`] for metadata + status lookups,
//! 2. a [`WasmHost`] for in-process WASM execution,
//! 3. a `tokio::task::spawn_blocking` bridge for the sync Wasmtime call so
//!    the executor thread is never blocked on module instantiation.
//!
//! Phase 1+ work (out of scope for the mock): capability manager, NATS
//! subscription bridge, gRPC/HTTP front, gRPC outbound, retry/backoff, real
//! fuel accounting, deadline-driven epoch interruption.
#![allow(missing_docs)]

use crate::contract::{
    FunctionMetadata, FunctionStatus, InvocationRequest, InvocationResult, Runtime,
};
use crate::error::{FunctionPlaneError, Result};
use crate::registry::{FunctionRegistry, InMemoryRegistry};
use crate::wasm_host::WasmHost;
use std::sync::Arc;
use std::time::Instant;

/// High-level facade for the Function Plane.
#[derive(Clone)]
pub struct FunctionGateway {
    registry: Arc<dyn FunctionRegistry>,
    host: Arc<WasmHost>,
}

impl std::fmt::Debug for FunctionGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionGateway")
            .field("registry", &"<dyn FunctionRegistry>")
            .field("host", &"<WasmHost>")
            .finish()
    }
}

impl FunctionGateway {
    /// Build a gateway from a (registry, host) pair.
    pub fn new(registry: Arc<dyn FunctionRegistry>, host: Arc<WasmHost>) -> Self {
        Self { registry, host }
    }

    /// Convenience constructor: pair an [`InMemoryRegistry`] with a fresh
    /// [`WasmHost`]. Used by tests, the PoC CLI, and the development binary.
    pub fn with_in_memory() -> Result<Self> {
        let registry = Arc::new(InMemoryRegistry::new());
        let host = Arc::new(WasmHost::new()?);
        Ok(Self { registry, host })
    }

    /// Register a function. If the metadata says `status == Active` and the
    /// runtime is WASM, the bytes are eagerly compiled and cached. Paused /
    /// Draft versions are metadata-only and will be hot-loaded later.
    pub async fn register(&self, meta: FunctionMetadata) -> Result<()> {
        let needs_compile = matches!(meta.runtime, Runtime::Wasm)
            && meta.status == FunctionStatus::Active;
        if needs_compile {
            self.host.register_module(&meta).await?;
        }
        tracing::debug!(
            function_id = %meta.function_id,
            version = %meta.version,
            runtime = ?meta.runtime,
            status = ?meta.status,
            "function-plane: register"
        );
        self.registry.register(meta).await
    }

    /// Invoke a function by `(function_id, version=None)`.
    ///
    /// Returns:
    /// - `Ok(InvocationResult { success: true, .. })` on a normal return.
    /// - `Ok(InvocationResult { success: false, .. })` if the function
    ///   completed but the runtime could not capture structured output (mock).
    /// - `Err(FunctionPlaneError::*)` for any pre-execution failure
    ///   (NotFound, NotActive, ContractInvalid, ...).
    pub async fn invoke(&self, req: InvocationRequest) -> Result<InvocationResult> {
        let started = Instant::now();
        let meta = self
            .registry
            .get(&req.function_id, req.version.as_deref())
            .await?;
        if meta.status != FunctionStatus::Active {
            return Err(FunctionPlaneError::NotActive(format!("{:?}", meta.status)));
        }

        // Phase 0 mock: only WASM runtime is wired. Container = explicit error.
        if !matches!(meta.runtime, Runtime::Wasm) {
            return Err(FunctionPlaneError::ContractInvalid(format!(
                "runtime {:?} not supported by mock gateway",
                meta.runtime
            )));
        }

        // Bridge to spawn_blocking so the sync Wasmtime call never blocks
        // a tokio worker thread.
        let host = self.host.clone();
        let meta_clone = meta.clone();
        let input = req.input.clone();
        let output_int = tokio::task::spawn_blocking(move || host.invoke_sync(&meta_clone, &input))
            .await
            .map_err(|e| FunctionPlaneError::Internal(format!("join error: {e}")))??;

        let duration_ms = started.elapsed().as_millis() as u64;
        // mock: assume configured fuel fully consumed on success.
        Ok(InvocationResult {
            function_id: meta.function_id.clone(),
            version: meta.version.clone(),
            output: Some(serde_json::json!({ "result": output_int })),
            fuel_consumed: Some(meta.fuel),
            duration_ms,
            success: true,
        })
    }

    /// Direct access to the underlying registry (read-only, used by tests).
    pub fn registry(&self) -> &Arc<dyn FunctionRegistry> {
        &self.registry
    }
}
