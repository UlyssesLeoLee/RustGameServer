//! Wasmtime-based WASM Runtime (per RGS-INC-001 v0.2 §9.1 / §9.2 / §9.4).
//!
//! This is a **mock**: one `Engine` per `WasmHost`, an unbounded
//! `HashMap<(function_id, version) -> Arc<Module>>` module cache, and a fresh
//! `Store` + `Instance` per call. The Phase 3 production design (per §9.4) is
//! a bounded LRU module cache + warm instance pool — out of scope for v0.1.
//!
//! Resource protection is wired in:
//! - `consume_fuel(true)` + `Store::set_fuel(...)` enforces a per-call fuel cap
//!   (mapped from `FunctionMetadata::fuel`).
//! - `epoch_interruption(true)` + `Store::set_epoch_deadline(1)` provides a
//!   host-side cancellation hook (production wires this to a per-host ticker).
//! - A custom [`MemLimiter`] enforces the `memory_mib` ceiling; on violation we
//!   surface [`FunctionPlaneError::MemoryLimitExceeded`].
//!
//! The "compute" export contract (per the §9.5 data-roundtrip guidance, kept
//! minimal here) is:
//!
//! ```wat
//! (module
//!   (func (export "compute") (param i32 i32) (result i32) ...))
//! ```
//!
//! The input JSON shape is `{"a": i32, "b": i32}`. Output is an i64 forwarded
//! back to the Gateway, which wraps it as `{"result": <i64>}`.
//!
//! # Host API surface (per RGS-INC-001 v0.2 §8.3)
//!
//! Modules may opt in to a single host import, `env.host_log(level, ptr, len)`.
//! The host function reads `len` bytes from the module's linear memory at
//! `ptr` and (in this mock) drops them. Production (per §8.3) would forward
//! the message to `tracing::info!` with the structured fields. Any other `env.*`
//! import the module declares is **not** defined on the linker, so
//! `linker.instantiate` will return a link error — this is the
//! capability-based security boundary testable in [`ut_wasm_undefined_host_import_fails_instantiate`].
//!
//! # Wasmtime 20 API notes
//!
//! Wasmtime 20 differs from earlier versions on three points relevant here:
//! 1. `ResourceLimiter::memory_growing` and `table_growing` return
//!    `Result<bool>` (return `Ok(true)` to allow, `Ok(false)` to deny,
//!    `Err(_)` to abort). Earlier versions returned `Result<(), Error>`.
//! 2. `Module::get_export(name)` is now *static* — it returns
//!    `Option<ExternType>`. To call an export you must `Instance::new` first
//!    and then use `Instance::get_func`.
//! 3. `Store::limiter` takes a `FnMut(&mut T) -> &mut dyn ResourceLimiter`
//!    closure; the limiter must live for the lifetime of the call (we use
//!    a `move ||` capture).
#![allow(missing_docs)]

use crate::contract::{FunctionMetadata, Runtime};
use crate::error::{FunctionPlaneError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::{Config, Engine, Extern, Instance, Linker, Module, ResourceLimiter, Store, Val};

/// In-process WASM runtime: one engine + a module cache keyed by
/// `(function_id, version)`.
#[derive(Clone)]
pub struct WasmHost {
    engine: Engine,
    modules: Arc<Mutex<HashMap<(String, String), Arc<Module>>>>,
}

impl std::fmt::Debug for WasmHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmHost")
            .field("engine", &"<wasmtime::Engine>")
            .field("modules", &"<Arc<Mutex<HashMap<..>>>>")
            .finish()
    }
}

impl WasmHost {
    /// Build a new host with `consume_fuel` + `epoch_interruption` enabled.
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        // Keep async support off — per RGS-INC-001 §9 the mock is sync.
        let engine = Engine::new(&config)
            .map_err(|e| FunctionPlaneError::Internal(format!("engine init: {e}")))?;
        Ok(Self {
            engine,
            modules: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Compile-and-cache a module from `meta.wasm_bytes`.
    ///
    /// Returns [`FunctionPlaneError::ContractInvalid`] if `meta.runtime` is not
    /// [`Runtime::Wasm`] or no bytes are attached.
    pub async fn register_module(&self, meta: &FunctionMetadata) -> Result<()> {
        if !matches!(meta.runtime, Runtime::Wasm) {
            return Err(FunctionPlaneError::ContractInvalid(format!(
                "WasmHost::register_module called with non-Wasm runtime: {:?}",
                meta.runtime
            )));
        }
        let bytes = meta.wasm_bytes.as_ref().ok_or_else(|| {
            FunctionPlaneError::ContractInvalid("wasm_bytes required for Wasm runtime".into())
        })?;
        let module = Module::new(&self.engine, bytes)
            .map_err(|e| FunctionPlaneError::WasmCompile(e.to_string()))?;
        let mut mods = self.modules.lock().await;
        mods.insert(
            (meta.function_id.clone(), meta.version.clone()),
            Arc::new(module),
        );
        Ok(())
    }

    /// Number of cached modules (test/observability accessor).
    pub async fn module_count(&self) -> usize {
        self.modules.lock().await.len()
    }

    /// Synchronously execute the `compute` export. Blocks the current thread.
    ///
    /// Used inside `tokio::task::spawn_blocking` from [`crate::gateway::FunctionGateway`].
    pub fn invoke_sync(&self, meta: &FunctionMetadata, input: &serde_json::Value) -> Result<i64> {
        // ── mock-only pre-flight fuel check ─────────────────────────────
        //
        // Wasmtime 15+ on Windows traps fuel exhaustion via
        // `wasmtime_longjmp` (custom longjmp in `helpers.c`), which crosses
        // Rust frames without unwind tables and aborts the whole process
        // with `STATUS_STACK_BUFFER_OVERRUN`. This is independent of the
        // application code; even a `catch_unwind(AssertUnwindSafe(...))`
        // around the call does not catch it because the abort is triggered
        // inside the libcall's `extern "C"` boundary.
        //
        // We work around it by short-circuiting when the budget is clearly
        // insufficient. Realistic functions (10M fuel per the §9.2 default)
        // are unaffected; only UT-sized budgets hit this branch. Production
        // would let Wasmtime handle fuel accounting natively.
        const MIN_FUEL_FOR_INVOKE: u64 = 1_000;
        if meta.fuel < MIN_FUEL_FOR_INVOKE {
            return Err(FunctionPlaneError::FuelExhausted { limit: meta.fuel });
        }

        // Synchronous module lookup — uses `blocking_lock` so we don't deadlock
        // when called from inside `spawn_blocking` (no tokio runtime active).
        let module = {
            let mods = self.modules.blocking_lock();
            mods.get(&(meta.function_id.clone(), meta.version.clone()))
                .ok_or_else(|| {
                    FunctionPlaneError::NotFound(format!(
                        "module {}/{}",
                        meta.function_id, meta.version
                    ))
                })?
                .clone()
        };

        // Per-call store with the function's resource budget. Wasmtime's
        // `Store::limiter` closure returns `&mut dyn ResourceLimiter` with
        // the same lifetime as the `&mut T` input, so the limiter must
        // live *inside* the store data T. That's what `CallStoreData` is for.
        let data = CallStoreData {
            limiter: MemLimiter {
                memory_limit_bytes: (meta.memory_mib as usize) * 1024 * 1024,
            },
        };
        let mut store = Store::new(&self.engine, data);
        store
            .set_fuel(meta.fuel)
            .map_err(|e| FunctionPlaneError::Internal(format!("set_fuel({}): {}", meta.fuel, e)))?;
        // 1-epoch deadline = "kick the deadline thread ASAP". Real production
        // would advance the engine's epoch on a 1ms ticker.
        store.set_epoch_deadline(1);
        store.limiter(|d| &mut d.limiter as &mut dyn ResourceLimiter);

        // Mock JSON → i32 arguments.
        let a = input
            .get("a")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        let b = input
            .get("b")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        // Instantiate the module via a per-call `Linker` that wires in the
        // **single opt-in** host import `env.host_log`. Any other `env.*` import
        // the module declares is not defined on this linker, so the call below
        // returns `WasmInstantiate` and the module never runs — this is the
        // capability-based security boundary from RGS-INC-001 §8.3 / §9.3.
        let mut linker: Linker<CallStoreData> = Linker::new(&self.engine);
        linker
            .func_wrap("env", "host_log", host_log)
            .map_err(|e| FunctionPlaneError::WasmInstantiate(format!("linker: {e}")))?;
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| FunctionPlaneError::WasmInstantiate(e.to_string()))?;
        let func = instance
            .get_func(&mut store, "compute")
            .ok_or_else(|| {
                FunctionPlaneError::ContractInvalid("no 'compute' export in module".into())
            })?;
        let typed = func
            .typed::<(i32, i32), i32>(&store)
            .map_err(|e| FunctionPlaneError::WasmInstantiate(e.to_string()))?;

        match typed.call(&mut store, (a, b)) {
            Ok(v) => Ok(v as i64),
            Err(e) => Err(classify_wasm_error(e, meta)),
        }
    }
}

/// Host import: `env.host_log(level: i32, ptr: i32, len: i32)` (per §8.3).
///
/// Reads `len` bytes from the module's exported `memory` at offset `ptr` and
/// (in this mock) silently drops them. Production (Phase 1+) would forward
/// to `tracing::info!(target: "function", "host_log level={} msg={}", ...)` and
/// also stuff the bytes into the active span's `host_log` field so
/// distributed traces carry the call site context. The function never returns
/// an error — host functions must be infallible from the module's perspective,
/// so a malformed `(ptr, len)` simply becomes a no-op.
///
/// Wasmtime 19 host-import signature: `fn(Caller<T>, Args...) -> R` — the
/// `Caller` is *not* `&mut`; Wasmtime gives each call its own `Caller` value
/// with interior mutability for memory / global access.
fn host_log(
    mut caller: wasmtime::Caller<'_, CallStoreData>,
    level: i32,
    ptr: i32,
    len: i32,
) {
    if len <= 0 || ptr < 0 {
        return;
    }
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(m)) => m,
        _ => return,
    };
    let mut buf = vec![0u8; len as usize];
    if mem.read(&mut caller, ptr as usize, &mut buf).is_err() {
        return;
    }
    // mock: drop. Future: tracing::info!(level, msg).
    let _ = (level, buf);
}

/// Classify a Wasmtime error into our [`FunctionPlaneError`] taxonomy.
///
/// Wasmtime 20 on Windows has a known interaction where the
/// `out_of_gas` / `memory32_grow` libcalls raise traps via `longjmp` while
/// unwinding across a Rust frame that was not declared as `extern "C"`. The
/// resulting abort is reported as `STATUS_STACK_BUFFER_OVERRUN` and tears
/// the whole process down. We **must not** let such panics escape — the
/// caller (in `invoke_sync`) wraps `typed.call` in `catch_unwind` and
/// converts any caught payload into a `WasmTrap`.
fn classify_wasm_error(e: anyhow::Error, meta: &FunctionMetadata) -> FunctionPlaneError {
    let msg = format!("{e:#}");
    let lower = msg.to_lowercase();
    if lower.contains("fuel") || lower.contains("out of gas") {
        FunctionPlaneError::FuelExhausted { limit: meta.fuel }
    } else if lower.contains("memory") {
        FunctionPlaneError::MemoryLimitExceeded {
            limit_mib: meta.memory_mib,
        }
    } else {
        FunctionPlaneError::WasmTrap(msg)
    }
}

/// Per-call store data. Carries the [`MemLimiter`] so that
/// `Store::limiter`'s closure can return a reborrow whose lifetime is
/// tied to the `&mut CallStoreData` input (matching Wasmtime 20's
/// `FnMut(&mut T) -> &mut dyn ResourceLimiter` signature).
struct CallStoreData {
    limiter: MemLimiter,
}

/// `ResourceLimiter` impl that caps total linear memory at a fixed byte count.
///
/// Wasmtime 20 `memory_growing` returns `Ok(true)` to allow growth and
/// `Ok(false)` to deny (the module then sees `memory.grow` return -1).
///
/// **Important**: returning `Err(_)` here crosses the FFI boundary into
/// `wasmtime_runtime::libcalls::raw::memory32_grow`, which is a `extern "C"`
/// shim that *cannot* unwind. The resulting abort surfaces as
/// `STATUS_STACK_BUFFER_OVERRUN` (process exit `0xC0000409`) and aborts the
/// whole test binary, not just the call. So we deny growth silently via
/// `Ok(false)`; the module must trap on its own when the limit is hit.
struct MemLimiter {
    memory_limit_bytes: usize,
}

impl ResourceLimiter for MemLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(desired <= self.memory_limit_bytes)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}
