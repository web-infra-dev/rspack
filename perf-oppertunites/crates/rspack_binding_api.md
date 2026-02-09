# rspack_binding_api

## Role
Shared binding API surface for JS↔Rust integration.

## Profiling relevance
- Indirectly affects performance in JS plugin-heavy builds.
- Costs scale with number of binding calls and payload size.

## Perf opportunities
- Batch binding calls to reduce NAPI overhead.
- Prefer zero-copy buffers for assets and sources.
- Avoid deep cloning when transferring large objects.

## Key functions/structs to inspect
- `Compiler` wrapper + `CompilerState::enter` (compiler.rs).
- `Compilation` module bindings (compilation/*).
- JS hook bridge in `plugins/js_hooks_plugin.rs` and `plugins/interceptor.rs`.
- Raw options parsing in `raw_options/*` (conversion overhead).

## Suggested experiments
- Measure per-hook binding overhead using a plugin-heavy workload.
- Compare zero-copy buffer paths vs cloned buffers.

## Code pointers
- `crates/rspack_binding_api/Cargo.toml`
- `crates/rspack_binding_api/src/lib.rs`
- `crates/rspack_binding_api/src/compiler.rs`
- `crates/rspack_binding_api/src/compilation/mod.rs`
- `crates/rspack_binding_api/src/plugins/mod.rs`
- `crates/rspack_binding_api/src/raw_options/mod.rs`
- `crates/rspack_binding_api/src/resolver.rs`
- `crates/rspack_binding_api/src/module_graph.rs`
- `crates/rspack_binding_api/**`
