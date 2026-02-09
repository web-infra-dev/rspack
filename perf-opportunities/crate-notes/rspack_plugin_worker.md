# rspack_plugin_worker

## Role
Worker plugin for handling worker entry points and chunking.

## Profiling relevance
- Not directly visible in react-10k; hot when worker entries are used.
- Impact increases with many worker entrypoints.

## Perf opportunities
- Cache resolved worker entry modules to avoid repeated resolution.
- Reuse generated worker runtime templates across chunks.
- Avoid string concatenations in per-module worker wrappers.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- Worker entry handling hooks in `src/lib.rs`.

## Suggested experiments
- Profile builds with many worker entries to measure resolution and wrapper overhead.
- Compare cache hit rates when workers are unchanged across rebuilds.

## Code pointers
- `crates/rspack_plugin_worker/Cargo.toml`
- `crates/rspack_plugin_worker/src/lib.rs`
- `crates/rspack_plugin_worker/**`
