# Plugin Hooks & Bindings

This document analyzes the hook system and JS↔Rust binding overhead that can
affect end‑to‑end build performance.

## Where the Work Happens

- Hook traits + macros: `crates/rspack_hook/src/lib.rs`
- Plugin driver + hook containers: `crates/rspack_core/src/plugin/plugin_driver.rs`
- Binding layer: `crates/rspack_binding_api/**`, `crates/rspack_napi/**`
- Node binding entrypoint: `crates/node_binding/**`

## Optimization Opportunities

### 1) Hook dispatch fast paths

Hook calls happen for many compilation stages. Ensure that hooks short‑circuit
quickly when no taps are registered:

- Add `is_empty()` checks for hook containers before creating vectors.
- Avoid allocating `Vec` for taps on empty hooks.

### 2) Reduce mutex contention on diagnostics

`PluginDriver` stores diagnostics in `Arc<Mutex<Vec<Diagnostic>>>`. On hot
paths, this may introduce lock contention:

- Use a lock‑free channel or per‑thread buffers merged at the end.
- Only lock when a plugin actually emits diagnostics.

### 3) Parser/Generator registration map

`registered_parser_and_generator_builder` uses `FxDashMap`. Ensure hot lookups
avoid repeated hashing:

- Cache resolved parser/generator for module types in `NormalModule`.
- Avoid repeated `ModuleType` conversions by storing pre‑computed keys.

### 4) NAPI boundary batching

Minimize the overhead of JS plugin hooks:

- Batch hook data into single calls instead of per‑module calls.
- Prefer zero‑copy buffers for source/asset payloads.
- Reduce JS object allocations for frequently called hooks.
