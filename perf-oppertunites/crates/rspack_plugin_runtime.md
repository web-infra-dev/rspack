# rspack_plugin_runtime

## Role
Runtime module generation and runtime requirements computation.

## Profiling relevance
- Not explicitly visible in flat samples; runtime codegen is part of chunk/asset passes.
- Costs scale with number of runtime variants and chunks.

## Perf opportunities
- Cache runtime module outputs keyed by runtime + feature flags.
- Avoid repeated template string concatenation; preallocate buffers.
- Skip runtime module regeneration when module hashes are unchanged.

## Key functions/structs to inspect
- `enable_chunk_loading_plugin` (lib.rs) — selects runtime loading plugins.
- `RuntimePlugin::apply` (runtime_plugin.rs) — hooks registration.
- Runtime module implementations in `runtime_module/*` (e.g. `load_script`, `jsonp_chunk_loading`).

## Suggested experiments
- Measure runtime module generation time on multi-runtime builds.
- Compare cached runtime outputs across incremental builds.

## Code pointers
- `crates/rspack_plugin_runtime/Cargo.toml`
- `crates/rspack_plugin_runtime/src/lib.rs`
- `crates/rspack_plugin_runtime/src/runtime_module/mod.rs`
- `crates/rspack_plugin_runtime/src/runtime_module/jsonp_chunk_loading.rs`
- `crates/rspack_plugin_runtime/src/runtime_module/load_script.rs`
- `crates/rspack_plugin_runtime/**`
