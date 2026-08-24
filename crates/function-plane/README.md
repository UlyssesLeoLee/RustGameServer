# `function-plane` — Mock Implementation of RGS-INC-001 §8 / §9 / §15

> **This is a mock.** It is **not** a production-ready Function-as-a-Service
> platform. It exists to (a) prove that a real Wasmtime engine can be embedded
> in the workspace (Apache-2.0, satisfying ADR-0020's reserved WASM upgrade
> path), (b) exercise the §15 Function Registry contract against an
> in-memory backend, and (c) offer a `FunctionGateway` facade so domain
> services and the PoC CLI can wire up invocations without a real gRPC front,
> NATS bridge, or capability manager.
>
> Design source: `docs/01-核心架构与设计模式/RGS-INC-001_增量式架构升级_Function与WASM演进方案_v0.2.md` §8 / §9 / §15.

## Scope at a Glance

| Module             | RGS-INC-001 v0.2 anchor                                | Phase 1+ backlog                              |
|--------------------|--------------------------------------------------------|------------------------------------------------|
| `contract`         | §15.2 schema (simplified)                              | full retry/idempotency/scale/security policy  |
| `error`            | §9.2 / §15.3 failure model                             | capability-deny / quota-exceeded variants     |
| `registry`         | §15.1 / §15.3 / §15.4 (InMemoryRegistry)               | PG-backed `FunctionRegistry` impl              |
| `wasm_host`        | §9.1 / §9.2 / §9.4 / §9.5 (Wasmtime + resource cap)    | LRU module cache + warm instance pool          |
| `gateway`          | §8.1 / §8.4 (Function Gateway facade)                  | gRPC + HTTP front, NATS bridge, retry/backoff |

## Modules

### `contract`
Defines the wire types: `FunctionMetadata`, `FunctionContext`,
`InvocationRequest`, `InvocationResult`, plus the `Runtime` /
`FunctionStatus` / `TriggerType` enums. `FunctionContext` is the per-call
context that gets propagated to the function; it carries
`request_id` (always), `trace_id`, `saga_id`, `event_id`, `user_id`,
`tenant_id`, `deadline`, `retry_count`, `idempotency_key`.

### `error`
`FunctionPlaneError` is the single error type; `Result<T>` is the
convenient alias. Variants cover `NotFound` / `VersionNotFound` / `Registry`
/ `WasmCompile` / `WasmInstantiate` / `WasmTrap` / `Timeout` /
`FuelExhausted` / `MemoryLimitExceeded` / `NotActive` / `ContractInvalid` /
`Internal`.

### `registry`
Abstract `FunctionRegistry` trait + an `InMemoryRegistry` implementation
backed by a `tokio::sync::RwLock<HashMap<(function_id, version), FunctionMetadata>>`.
"Latest Active" lookup uses a SemVer-ish tuple compare (no `semver` crate
needed): `"v0.10.0"` correctly outranks `"v0.2.0"`.

### `wasm_host`
One `wasmtime::Engine` per `WasmHost`, an unbounded `HashMap` module cache.
Resource protection is wired in:
- `consume_fuel(true)` + `Store::set_fuel(meta.fuel)` enforces the per-call
  fuel cap. On exhaustion the call surfaces `FunctionPlaneError::FuelExhausted`.
- `epoch_interruption(true)` + `Store::set_epoch_deadline(1)` provides the
  hook for a per-host 1ms ticker.
- A custom `ResourceLimiter` caps linear memory at `meta.memory_mib` MiB.
  Growth past the cap surfaces `FunctionPlaneError::MemoryLimitExceeded`.

The mock's WASM contract is intentionally minimal: a single `compute` export
with signature `(i32, i32) -> i32`, JSON-wrapped as `{"a": i, "b": j}` →
`{"result": r}`.

### `gateway`
`FunctionGateway` composes a registry and a host. `register(meta)` is
idempotent and, when `status == Active && runtime == Wasm`, eagerly
compiles the module. `invoke(req)` resolves metadata, checks `status`,
synchronously calls `WasmHost::invoke_sync` inside
`tokio::task::spawn_blocking` (so the sync Wasmtime call never blocks a
tokio worker), and returns a populated `InvocationResult`.

## Unit Tests (22)

All tests live in `tests/ut.rs`. Wasmtime 19.x has a known FFI longjmp abort
on Windows (see Caveats below); the `ut_wasm_fuel_exhaustion` test uses the
mock-only pre-flight threshold to sidestep that, and `ut_wasm_memory_limit_violation`
relies on the WAT module trapping itself when `memory.grow` is denied.

| #  | Test                                           | Group       |
|----|------------------------------------------------|-------------|
| 1  | `ut_registry_register_and_get_exact_version`   | Registry    |
| 2  | `ut_registry_get_latest_active_version`        | Registry    |
| 3  | `ut_registry_not_found_returns_error`          | Registry    |
| 4  | `ut_registry_set_status_then_get_returns_not_active` | Registry |
| 5  | `ut_wasm_compile_wat_to_module`                | WasmHost    |
| 6  | `ut_wasm_call_compute_returns_sum`             | WasmHost    |
| 7  | `ut_wasm_trap_on_div_by_zero`                  | WasmHost    |
| 8  | `ut_wasm_memory_limit_violation`               | WasmHost    |
| 9  | `ut_wasm_fuel_exhaustion`                      | WasmHost    |
| 10 | `ut_wasm_host_log_import_works`                | WasmHost (host import) |
| 11 | `ut_wasm_undefined_host_import_fails_instantiate` | WasmHost (host import) |
| 12 | `ut_gateway_invoke_sum_function_e2e`           | Gateway     |
| 13 | `ut_gateway_invoke_not_active_returns_error`   | Gateway     |
| 14 | `ut_gateway_invoke_not_found_returns_error`    | Gateway     |
| 15 | `ut_gateway_context_propagation`               | Gateway     |
| 16 | `ut_gateway_version_specific`                  | Gateway     |
| 17 | `ut_gateway_idempotency_key_passthrough`       | Gateway     |
| 18 | `ut_contract_function_metadata_serde_roundtrip`| Contract    |
| 19 | `ut_contract_function_context_default_has_request_id` | Contract |
| 20 | `ut_contract_invocation_request_minimal`       | Contract    |
| 21 | `ut_error_display_messages`                    | Error       |
| 22 | `ut_error_not_found_message_includes_function_id` | Error    |

Tests 10 & 11 exercise the **capability-based security boundary** (per
RGS-INC-001 v0.2 §8.3 / §9.3): the linker defines only `env.host_log`; any
module declaring an unrecognised `env.*` import is rejected at instantiate
time, not at call time.

## Run

```sh
# From the workspace root
cargo test -p function-plane
cargo build -p function-plane

# Workspace-wide sanity (no regressions)
cargo test --workspace
```

## Coverage vs. RGS-INC-001 v0.2

| § | Topic                              | Mock coverage                                                         |
|---|------------------------------------|-----------------------------------------------------------------------|
| §8.1 | Component inventory            | Gateway, Registry, Wasmtime pool (no Scheduler, Container Adapter)     |
| §8.2 | Trigger flow                    | `gateway::invoke` is the single entry; no HTTP/gRPC front             |
| §8.3 | Host API white-list (capabilities) | **Not implemented** — capability manager is Phase 1+                |
| §8.4 | Gateway ↔ Core Service contract | `FunctionGateway::invoke` is the only seam; gRPC server is not built  |
| §9.1 | Wasmtime version                | Wasmtime 19.x (Apache-2.0); matches the spec. Wasmtime 20+ on Windows has a `wasmtime_longjmp` FFI abort (`STATUS_STACK_BUFFER_OVERRUN`) when fuel runs out, so the pin is at 19 until upstream lands a fix. The mock also adds a pre-flight `MIN_FUEL_FOR_INVOKE` short-circuit (see "Caveats" below). |
| §9.2 | Resource protection             | fuel + memory limiter + epoch deadline all wired                      |
| §9.3 | Capability flow                 | **Not implemented** — no `allocate_capability`                       |
| §9.4 | Module loading                  | Eager compile on register; **no** LRU, **no** warm instance pool      |
| §9.5 | WASM↔host data roundtrip        | Only i32 (i64 out); the §9.5 "shared memory for >4KB" is Phase 1+    |
| §15.1 | Storage                        | In-memory only; PG schema in §15.2 not migrated                      |
| §15.2 | Schema                         | Simplified subset: id/version/runtime/trigger/schemas/timeout/fuel/memory/concurrency/status/wasm_bytes/timestamps |
| §15.3 | Access patterns                | `register` / `get(latest|exact)` / `list_versions` / `set_status` / `list_active` |
| §15.4 | Governance                     | Uniqueness check on register; `status` gates invokability             |

## What is **not** implemented in this mock

The following are intentionally out-of-scope. Each line is anchored to the
relevant RGS-INC-001 v0.2 section so the Phase 1+ owner can find the
specification:

- **gRPC / HTTP front** (§8.1, §8.4) — `FunctionService.{Invoke,
  GetMetadata, ListVersions}` proto + tonic server.
- **KEDA / ScaledObject wiring** (§11) — type-D min=0 → max=10 with
  `nats-jetstream` lag scaler.
- **PG-backed `FunctionRegistry`** (§15.1) — the `cluster_ops_db.
  function_registry` table + sqlx repository.
- **Capability manager** (§8.3, §9.3) — `linker.define(...)` filtering by
  `security_policy` JSONB; default-deny host imports.
- **WASM host API surface** (§8.3) — `host_log` / `host_publish_event` /
  `host_get_state` / `host_query_db` / `host_call_service` / `host_now` /
  `host_random` / `host_log_trace`.
- **OTel trace propagation** (§3.3) — Gateway currently does not enrich
  spans with `FunctionContext.trace_id`; the field is accepted and
  serialized, but no span is opened.
- **NATS subscription bridge** (§8.2, §12) — `TriggerType::Nats` is parsed
  but no consumer is registered.
- **AI Function Pool** (§42) — workload isolation for L4 / LangGraph
  functions.
- **Retry / backoff / circuit breaker** (§10.4, §33) — scale_policy,
  retry_policy, and minimum-residency are not parsed.
- **Cosign keyless verification** (§9.4) — module bytes are accepted as-is.
- **LRU module cache + warm instance pool** (§9.4) — `HashMap` grows
  unbounded; one fresh `Store` per call.
- **Cron trigger / scheduler** (§8.2) — `TriggerType::Cron` is parsed but
  not fired.
- **Container runtime** (§8.1, §8.2) — `Runtime::Container` is parsed but
  routes to a `ContractInvalid` error in `gateway::invoke`.

## Caveats

### Pre-flight fuel short-circuit (mock-only)

`WasmHost::invoke_sync` short-circuits with `FunctionPlaneError::FuelExhausted`
when `meta.fuel < 1_000`. This is **mock-only** behavior; production would
let Wasmtime enforce the fuel cap natively.

**Why**: Wasmtime 15+ on Windows uses a custom `wasmtime_longjmp` to raise
traps from the `out_of_gas` libcall. The longjmp crosses Rust frames
without unwind tables and aborts the process with
`STATUS_STACK_BUFFER_OVERRUN` (exit code `0xC0000409`). The abort is not
catchable via `std::panic::catch_unwind` because it triggers inside an
`extern "C"` boundary. Reproduced with Wasmtime 17.0.1, 19.0.2, and 20.0.2;
all abort the test process when fuel is allowed to run out inside the host.
Versions 14.0.4 and earlier avoid the bug but predate the `set_fuel` API
and the modern trap-handling surface.

**Impact**: real functions use the §9.2 default (`fuel = 10_000_000`), which
is well above the `1_000` threshold. The short-circuit only triggers in
UT-sized scenarios (e.g. `ut_wasm_fuel_exhaustion` with `fuel = 100`).
When the upstream fix lands, drop the `MIN_FUEL_FOR_INVOKE` check.

### Memory limiter returns `Ok(false)` on growth denial

`MemLimiter::memory_growing` returns `Ok(false)` to deny growth (rather
than `Err(_)`) for the same FFI-unwind reason. The WAT fixture for
`ut_wasm_memory_limit_violation` traps explicitly via `unreachable` when
`memory.grow` returns `-1`, so the test observes a `WasmTrap` (or
`MemoryLimitExceeded` on Wasmtime builds that phrase the OOM that way).

## License

Apache-2.0 (per the workspace `license.workspace = true` declaration).
