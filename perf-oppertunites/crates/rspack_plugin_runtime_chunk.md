# rspack_plugin_runtime_chunk

## Role
Runtime chunk extraction and configuration.

## Profiling relevance
- Not visible in react-10k perf list; hot when runtime chunk is split.
- Cost scales with number of entries and runtime variants.

## Perf opportunities
- Avoid repeated runtime chunk decisions across modules.
- Cache runtime chunk names and template outputs.
- Skip runtime chunk work when output uses a single chunk.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- Runtime chunk decision logic in `src/lib.rs` (hook implementations).

## Suggested experiments
- Measure runtime chunk decision time on multi-entry builds.
- Compare cached vs. recomputed runtime chunk naming.

## Code pointers
- `crates/rspack_plugin_runtime_chunk/Cargo.toml`
- `crates/rspack_plugin_runtime_chunk/src/lib.rs`
- `crates/rspack_plugin_runtime_chunk/**`
