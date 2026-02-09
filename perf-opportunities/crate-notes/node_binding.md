# node_binding (rspack_node)

## Role
Node.js binding entrypoint for Rspack.

## Profiling relevance
- Indirectly visible in builds with heavy JS plugin usage.
- Overhead scales with number of JS↔Rust calls and payload size.

## Perf opportunities
- Batch JS↔Rust calls to reduce NAPI overhead.
- Prefer zero-copy buffers for sources/assets.
- Avoid cloning large structures when marshaling to JS.

## Key functions/structs to inspect
- Thin re-export crate; key entrypoints are in `rspack_binding_api`.
- Build/packaging scripts in `scripts/build.js` and `scripts/move-binding.js`.

## Suggested experiments
- Measure NAPI call counts per build and identify top hooks.
- Compare zero-copy vs clone paths for large sources.

## Code pointers
- `crates/node_binding/Cargo.toml`
- `crates/node_binding/src/lib.rs`
- `crates/node_binding/**`
