# rspack_plugin_remove_empty_chunks

## Role
Remove empty chunks from the chunk graph after optimization.

## Profiling relevance
- Not visible in react-10k; hot when many chunks are created and pruned.
- Cost scales with chunk graph size.

## Perf opportunities
- Track emptiness incrementally to avoid full graph scans.
- Batch removals to reduce chunk graph mutation overhead.
- Avoid recomputation when chunk graph is unchanged.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- Empty chunk removal logic in `src/lib.rs`.

## Suggested experiments
- Measure empty-chunk removal time on large split-chunk configs.
- Compare incremental vs full scan approaches.

## Code pointers
- `crates/rspack_plugin_remove_empty_chunks/Cargo.toml`
- `crates/rspack_plugin_remove_empty_chunks/src/lib.rs`
- `crates/rspack_plugin_remove_empty_chunks/**`
